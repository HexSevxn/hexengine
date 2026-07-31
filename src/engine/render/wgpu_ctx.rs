use std::borrow::Cow;
use std::sync::Arc;
use wgpu::MemoryHints::Performance;
use wgpu::util::DeviceExt;
use wgpu::{CurrentSurfaceTexture, ShaderSource, Trace};
use winit::window::Window;

use super::{Triangle, Vertex};

pub const TRIANGLE_SHADER_SOURCE: ShaderSource = ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/triangle.wgsl")));
pub const CIRCLE_SHADER_SOURCE: ShaderSource = ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/circle.wgsl")));

const SURFACE_BACKGROUND_COLOR: wgpu::Color = wgpu::Color::BLACK;

#[allow(dead_code)]
pub struct WgpuCtx<'window> {
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tri_render_pipeline: wgpu::RenderPipeline,
    pub tri_object_store: Vec<Triangle>,
    tri_vertex_buffer: TriangleVertexBuffer,
}

#[repr(C)]
#[derive(Clone)]
pub struct TriangleVertexBuffer {
    pub buffer: Option<wgpu::Buffer>,
    pub num_vertices: u32,
}

impl TriangleVertexBuffer {
    pub fn new(buffer: wgpu::Buffer, num_vertices: u32) -> TriangleVertexBuffer {
        return TriangleVertexBuffer {
            buffer: Some(buffer),
            num_vertices,
        };
    }
    pub fn empty() -> TriangleVertexBuffer {
        return TriangleVertexBuffer {
            buffer: None,
            num_vertices: 0,
        };
    }
}

impl<'window> WgpuCtx<'window> {
    pub async fn new_async(window: Arc<Window>) -> WgpuCtx<'window> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");
        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Nvidia GPU"),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: Performance,
                trace: Trace::Off,
            })
            .await
            .expect("Failed to create device");

        // Get the internal physical pixel dimensions of the window (without the title bar)
        let size = window.inner_size();
        // At least (w = 1, h = 1), otherwise WGPU will panic
        let width = size.width.max(1);
        let height = size.height.max(1);
        // Get a default configuration
        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        // Complete initial configuration
        surface.configure(&device, &surface_config);

        let tri_object_store: Vec<Triangle> = Vec::new();
        let (tri_render_pipeline, tri_vertex_buffer) =
            setup_tri_pipeline(&device, &surface_config.format, &tri_object_store);

        WgpuCtx {
            surface,
            surface_config,
            adapter,
            device,
            queue,
            tri_render_pipeline,
            tri_object_store,
            tri_vertex_buffer,
        }
    }

    pub fn new(window: Arc<Window>) -> WgpuCtx<'window> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    //Recreate the render pipeline with current object store
    pub fn update_tri_pipeline(&mut self) {
        (self.tri_render_pipeline, self.tri_vertex_buffer) = setup_tri_pipeline(
            &self.device,
            &self.surface_config.format,
            &self.tri_object_store,
        )
    }

    pub fn draw(&mut self) {
        //Attempts to fetch the current surface texture, multiple possible states must be handled, and some require reconfiguration of the surface.
        let surface_texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture) | CurrentSurfaceTexture::Suboptimal(texture) => texture,
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => return,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.surface_config);
                return //Lost could require recreation of device, uneccessary error to handle
            },
            CurrentSurfaceTexture::Validation => return,
        };
        
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            // Create the render pass
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(SURFACE_BACKGROUND_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Draw TRIANGLES by setting the active pipeline and vertex buffer to the triangle one.
            rpass.set_pipeline(&self.tri_render_pipeline);
            rpass.set_vertex_buffer(
                0,
                self.tri_vertex_buffer
                    .buffer
                    .clone()
                    .expect("No vertex buffer found!")
                    .slice(..),
            );
            rpass.draw(0..(self.tri_vertex_buffer.num_vertices), 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(surface_texture)
    }
}

fn setup_tri_pipeline(
    device: &wgpu::Device,
    swap_chain_format: &wgpu::TextureFormat,
    object_store: &Vec<Triangle>,
) -> (wgpu::RenderPipeline, TriangleVertexBuffer) {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(object_store),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("triangle_shader"),
        source: TRIANGLE_SHADER_SOURCE,
    });

    (
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle_pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(swap_chain_format.clone().into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                // strip_index_format: None,
                // front_face: wgpu::FrontFace::Ccw,
                // cull_mode: Some(wgpu::Face::Back),
                // // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // // or Features::POLYGON_MODE_POINT
                // polygon_mode: wgpu::PolygonMode::Fill,
                // // Requires Features::DEPTH_CLIP_CONTROL
                // unclipped_depth: false,
                // // Requires Features::CONSERVATIVE_RASTERIZATION
                // conservative: false,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        }),
        TriangleVertexBuffer::new(vertex_buffer, object_store.len() as u32 * 3),
    )
}
