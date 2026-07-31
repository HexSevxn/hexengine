use std::sync::Arc;

use glam::{Vec2, Vec4, vec2};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use crate::engine::asset::{Asset, AssetManager, Circle, Line, Object, Rectangle};
use crate::engine::render::wgpu_ctx::WgpuCtx;
use crate::engine::render::{Triangle, Vertex};

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    pub display_size: Vec2,
    pub asset_manager: Option<AssetManager>,
}

impl<'window> App<'window> {
    // Draws a rectangle at position with given width, height, and color. position is pixel based and converted to proper proportion
    pub fn draw_rectangle(&mut self, position: Vec2, width: f32, height: f32, color: Vec4) {
        let display_size = self.display_size.clone();

        let scaled_position = scale_to_screen(display_size, position);
        let scaled_components = scale_pixel(display_size, vec2(width, height));

        self.push_asset(Rectangle {
            position: scaled_position,
            width: scaled_components.x,
            height: scaled_components.y,
            color,
        });
    }

    //Draws a line between two given points, with a pixel width and color
    pub fn draw_line(&mut self, p1: Vec2, p2: Vec2, width: f32, color: Vec4) {
        let screen_size = self.display_size.clone();

        let size = vec2(width, width);
        let half_size = vec2(size.x / screen_size.x, size.y / screen_size.y) / 2.0;

        self.push_asset(Line {
            start: p1,
            end: p2,
            color: color,
            size: half_size,
        });
    }

    pub fn draw_triangle(&mut self, v1: Vec2, v2: Vec2, v3: Vec2, color: Vec4) {
        let (v1, v2, v3): (Vertex, Vertex, Vertex) = (
            Vertex::from_vec2_c(v1, color),
            Vertex::from_vec2_c(v2, color),
            Vertex::from_vec2_c(v3, color),
        );

        self.push_asset(Triangle::new(v1, v2, v3));
    }

    pub fn draw_circle(&mut self, position: Vec2, radius: f32, color: Vec4) {
        self.push_asset(Circle {
            position,
            radius,
            color,
        });

        //UNIMPLEMENTED
    }
    pub fn draw_triangle_raw(&mut self, v1: Vertex, v2: Vertex, v3: Vertex) {
        self.push_asset(Triangle::new(v1, v2, v3));
    }

    pub fn push_asset(&mut self, item: impl Asset + 'static) {
        let triangles = item.unpack();
        let object = Object::new(item);
        self.asset_manager.as_mut().unwrap().pool.push(object);

        for tri in triangles {
            self.wgpu_ctx.as_mut().unwrap().tri_object_store.push(tri);
        }
    }

    pub fn update_pipeline(&mut self) {
        self.wgpu_ctx.as_mut().unwrap().update_tri_pipeline();
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("wgpu winit example").with_inner_size(LogicalSize::new(800., 800.));
            // use Arc.
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            self.window = Some(window.clone());
            self.display_size = [
                window.inner_size().width as f32,
                window.inner_size().height as f32,
            ]
            .into();
            let wgpu_ctx = WgpuCtx::new(window.clone());
            self.wgpu_ctx = Some(wgpu_ctx);
            self.asset_manager = Some(AssetManager::empty());
            crate::setup_graphics(self);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(wgpu_ctx), Some(window)) =
                    (self.wgpu_ctx.as_mut(), self.window.as_ref())
                {
                    wgpu_ctx.resize((new_size.width, new_size.height));
                    self.display_size = [new_size.width as f32, new_size.height as f32].into();
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.draw();
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    logical_key, text, location, repeat, ..},
                ..
            } => (), //Handle any keypresses here!!!
            WindowEvent::MouseInput {
                 device_id: _device_id, state, button 
            } => (), //Handle mouse input here!
            _ => (),
        }
    }
}

//Takes a coordinate in pixels and converts it to a screenspace coordinate based on the display size
pub fn scale_to_screen(display_size: Vec2, position: Vec2) -> Vec2 {
    let scaled = vec2(
        ((position.x / display_size.x) * 2.0) - 1.0,
        ((position.y / display_size.y) * 2.0) - 1.0,
    );
    scaled
}

pub fn scale_pixel(display_size: Vec2, pixel: Vec2) -> Vec2 {
    (pixel / display_size) * 2.0
}
