use std::borrow::Cow;
use std::mem::size_of;
use std::sync::Arc;
use wgpu::MemoryHints::Performance;
use wgpu::{CurrentSurfaceTexture, ShaderSource, Trace};
use winit::window::Window;

use super::Triangle;

pub const TRIANGLE_SHADER_SOURCE: ShaderSource =
    ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/triangle.wgsl")));
pub const CIRCLE_SHADER_SOURCE: ShaderSource =
    ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/circle.wgsl")));

const SURFACE_BACKGROUND_COLOR: wgpu::Color = wgpu::Color::BLACK;

#[allow(dead_code)]
#[derive(Debug)]
pub struct WgpuCtx<'window> {
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tri_render_pipeline: wgpu::RenderPipeline,
    pub tri_object_store: Vec<Triangle>,
    tri_instance_buffer: TriangleInstanceBuffer,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct TriangleInstanceBuffer {
    pub buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub num_instances: u32,
    pub capacity_instances: u32,
}

impl TriangleInstanceBuffer {
    pub fn new(
        buffer: wgpu::Buffer,
        bind_group: wgpu::BindGroup,
        num_instances: u32,
        capacity_instances: u32,
    ) -> TriangleInstanceBuffer {
        return TriangleInstanceBuffer {
            buffer: Some(buffer),
            bind_group: Some(bind_group),
            num_instances,
            capacity_instances,
        };
    }
    pub fn empty() -> TriangleInstanceBuffer {
        return TriangleInstanceBuffer {
            buffer: None,
            bind_group: None,
            num_instances: 0,
            capacity_instances: 16,
        };
    }
}

pub fn create_tri_storage_buffer(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    capacity_instances: u32,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let size = (capacity_instances.max(1) as u64) * (size_of::<Triangle>() as u64);

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Triangle Instance Buffer"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("triangle_bind_group"),
        layout: bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    (buffer, bind_group)
}

fn setup_tri_pipeline(
    device: &wgpu::Device,
    swap_chain_format: &wgpu::TextureFormat,
    object_store: &Vec<Triangle>,
) -> (wgpu::RenderPipeline, TriangleInstanceBuffer) {
    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("triangle_shader"),
        source: TRIANGLE_SHADER_SOURCE,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("triangle_pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
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
    });
    let bind_group_layout = pipeline.get_bind_group_layout(0);
    let initial_capacity = 16u32;
    let (storage_buffer, bind_group) =
        create_tri_storage_buffer(device, &bind_group_layout, initial_capacity);

    (
        pipeline,
        TriangleInstanceBuffer::new(
            storage_buffer,
            bind_group,
            object_store.len() as u32,
            initial_capacity,
        ),
    )
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
                required_limits: wgpu::Limits {
                    max_storage_buffers_per_shader_stage: 1,
                    ..wgpu::Limits::downlevel_defaults() // or ::default() for full WebGPU limits
                }
                .using_resolution(adapter.limits()),
                memory_hints: Performance,
                trace: Trace::Off,
            })
            .await
            .expect("Failed to create device");

        // Get the internal physical pixel dimensions of the window (without the title bar)
        let size = window.inner_size();
        // Width and Height must be at least 1
        let width = size.width.max(1);
        let height = size.height.max(1);
        // Get a default configuration
        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        // Complete initial configuration
        surface.configure(&device, &surface_config);

        let tri_object_store: Vec<Triangle> = Vec::new();
        let (tri_render_pipeline, tri_instance_buffer) =
            setup_tri_pipeline(&device, &surface_config.format, &tri_object_store);

        WgpuCtx {
            surface,
            surface_config,
            adapter,
            device,
            queue,
            tri_render_pipeline,
            tri_object_store,
            tri_instance_buffer,
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

    //updates the pipeline with the current object store
    pub fn sync_tri_instances(&mut self) {
        let needed = self.tri_object_store.len() as u32;

        if needed > self.tri_instance_buffer.capacity_instances {
            // Grow with some slack so we're not reallocating every time the count creeps up by one.
            let new_capacity = needed.max(self.tri_instance_buffer.capacity_instances * 2);
            let bind_group_layout = self.tri_render_pipeline.get_bind_group_layout(0);
            let (buffer, bind_group) =
                create_tri_storage_buffer(&self.device, &bind_group_layout, new_capacity);
            self.tri_instance_buffer.buffer = Some(buffer);
            self.tri_instance_buffer.bind_group = Some(bind_group);
            self.tri_instance_buffer.capacity_instances = new_capacity;
        }

        if needed > 0 {
            self.queue.write_buffer(
                self.tri_instance_buffer.buffer.as_ref().expect("no buffer"),
                0,
                bytemuck::cast_slice(&self.tri_object_store),
            );
        }

        self.tri_instance_buffer.num_instances = needed;
    }

    pub fn draw(&mut self) {
        //Attempts to fetch the current surface texture, multiple possible states must be handled, and some require reconfiguration of the surface.
        let surface_texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture)
            | CurrentSurfaceTexture::Suboptimal(texture) => texture,
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => return,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.surface_config);
                return; //Lost could require recreation of device, uneccessary error to handle
            }
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
            rpass.set_bind_group(
                0,
                self.tri_instance_buffer
                    .bind_group
                    .as_ref()
                    .expect("No vertex buffer found!"),
                &[],
            );
            rpass.draw(0..3, 0..self.tri_instance_buffer.num_instances);
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(surface_texture)
    }
}