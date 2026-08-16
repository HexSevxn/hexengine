use std::f32::consts::PI;
use std::sync::Arc;

use glam::{Vec2, Vec3, Vec4, vec2};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::engine::ecs::world::World;
use crate::engine::game::{Camera, Geometry, GeometryType, Renderable, Transformation};
use crate::engine::math::get_rectangle_triangles;
use crate::engine::render::color::Color;
use crate::engine::render::wgpu_ctx::WgpuCtx;
use crate::engine::render::{Triangle, TriangleColorMap};

#[derive(Default, Debug)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    pub display_size: Vec2,
    pub world: World,
}

impl<'window> App<'window> {
    //RENDER SYSTEM UPDATE
    pub fn render_step(&mut self) {
        let ctx = self
            .wgpu_ctx
            .as_mut()
            .expect("Render step error getting WGPU_CTX");
        let render_query = self
            .world
            .query::<(&Renderable, &Geometry, &Transformation)>();

        ctx.tri_object_store.clear();

        for (render_data, geometry, transform) in render_query {
            if !render_data.visible {
                continue;
            }

            for triangle in &geometry.vertices {
                ctx.tri_object_store.push(Triangle {
                    position: transform.position.into(),
                    rotation: transform.rotation,
                    v0: triangle.v0,
                    v1: triangle.v1,
                    v2: triangle.v2,
                    c0: triangle.c0,
                    c1: triangle.c1,
                    c2: triangle.c2,
                    ..Default::default()
                });
            }
        }
        ctx.sync_tri_instances();
        ctx.draw();
    }

    // Draws a rectangle at position with given width, height, and color. position is pixel based and converted to proper proportion
    pub fn draw_rectangle(&mut self, position: Vec2, width: f32, height: f32, color: Vec4) {
        let display_size = self.display_size.clone();

        let scaled_position = scale_to_screen(display_size, position);
        let scaled_components = scale_pixel(display_size, vec2(width, height));
        let midpoint = vec2(position.x + (width / 2.0), position.y + (height / 2.0));

        let (t1, t2) = get_rectangle_triangles(scaled_position, scaled_components.x, scaled_components.y, color);

        let rectangle = self.world.new_entity();
        let transform = Transformation {
            position: midpoint,
            velocity: Vec2::ZERO,
            rotation: 0.0_f32,
        };
        self.world.add_component_to_entity(
            rectangle,
            Geometry {
                geometry_type: GeometryType::Triangle,
                vertices: vec![t1, t2],
            },
        );
        self.world.add_component_to_entity(rectangle, transform);
        self.world.add_component_to_entity(
            rectangle,
            Renderable {
                color: TriangleColorMap::flat(color),
                visible: true,
            },
        );
    }

    //Draws a line between two given points, with a pixel width and color
    pub fn draw_line(&mut self, start: Vec2, end: Vec2, width: f32, color: Vec4) {
        let screen_size = self.display_size.clone();
        let midpoint = start.midpoint(end);
        let color_map = TriangleColorMap::flat(color);

        let size = vec2(width, width);
        let half_size = vec2(size.x / screen_size.x, size.y / screen_size.y) / 2.0;

        let (dy, dx) = (end.y - start.y, end.x - start.x);
        let length: f32 = (dx * dx + dy * dy).sqrt();
        let normalized: Vec2 = vec2(-dy / length, dx / length);

        let v0 = (start + (normalized * half_size)) - midpoint;
        let v1 = (start - (normalized * half_size)) - midpoint;

        let v2 = (end + (normalized * half_size)) - midpoint;
        let v3 = (end - (normalized * half_size)) - midpoint;

        let t1 = Triangle::new(v0, v2, v1, color_map);
        let t2 = Triangle::new(v1, v2, v3, color_map);

        let line = self.world.new_entity();
        let transform = Transformation {
            position: midpoint,
            velocity: Vec2::ZERO,
            rotation: 0.0_f32,
        };
        self.world.add_component_to_entity(
            line,
            Geometry {
                geometry_type: GeometryType::Triangle,
                vertices: vec![t1, t2],
            },
        );
        self.world.add_component_to_entity(line, transform);
        self.world.add_component_to_entity(
            line,
            Renderable {
                color: TriangleColorMap::flat(color),
                visible: true,
            },
        );
    }

    pub fn draw_circle(&mut self, position: Vec2, radius: f32, color: Vec4) {
        let mut triangles: Vec<Triangle> = Vec::new();
        let color_map = TriangleColorMap::flat(color);
        //DICTATES THE NUMBER OF TRIANGLES USED TO BUILD A CIRCLE
        const CIRCLE_TRI_COUNT: usize = 30;

        const SCALAR: f32 = 2_f32 * PI / CIRCLE_TRI_COUNT as f32;

        for index in 0..CIRCLE_TRI_COUNT {
            triangles.push(Triangle::new(
                vec2(
                    position.x + (radius * (index as f32 * SCALAR).cos()),
                    position.y + (radius * (index as f32 * SCALAR).sin()),
                ) - position,
                vec2(
                    position.x + (radius * ((index + 1) as f32 * SCALAR).cos()),
                    position.y + (radius * ((index + 1) as f32 * SCALAR).sin()),
                ) - position,
                Vec2::ZERO,
                color_map,
            ))
        }

        let circle = self.world.new_entity();
        let transform = Transformation {
            position,
            velocity: Vec2::ZERO,
            rotation: 0.0_f32,
        };
        self.world.add_component_to_entity(
            circle,
            Geometry {
                geometry_type: GeometryType::Circle,
                vertices: triangles,
            },
        );
        self.world.add_component_to_entity(circle, transform);
        self.world.add_component_to_entity(
            circle,
            Renderable {
                color: TriangleColorMap::flat(color),
                visible: true,
            },
        );
    }

    pub fn draw_triangle(&mut self, v0: Vec2, v1: Vec2, v2: Vec2, color: Vec4) {
        let color_map = TriangleColorMap::flat(color);
        let triangle = self.world.new_entity();
        let transform = Transformation {
            position: v0.midpoint(v1).midpoint(v2),
            velocity: Vec2::ZERO,
            rotation: 0.0_f32,
        };
        self.world.add_component_to_entity(
            triangle,
            Geometry {
                geometry_type: GeometryType::Triangle,
                vertices: vec![Triangle::new(
                    v0 - transform.position,
                    v1 - transform.position,
                    v2 - transform.position,
                    color_map,
                )],
            },
        );
        self.world.add_component_to_entity(triangle, transform);
        self.world.add_component_to_entity(
            triangle,
            Renderable {
                color: TriangleColorMap::flat(color),
                visible: true,
            },
        );
    }

    pub fn draw_triangle_raw(
        &mut self,
        position: Vec2,
        rotation: f32,
        v0: Vec2,
        v1: Vec2,
        v2: Vec2,
        c0: Vec4,
        c1: Vec4,
        c2: Vec4,
    ) {
        let triangle = self.world.new_entity();
        let transform = Transformation {
            position: vec2(
                ((v0.x + v1.x) / 2.0 + v2.x) / 2.0,
                ((v0.y + v1.y) / 2.0 + v2.y) / 2.0,
            ),
            velocity: Vec2::ZERO,
            rotation,
        };
        let (v0, v1, v2) = (
            vec2(v0.x - transform.position.x, v0.y - transform.position.y),
            vec2(v1.x - transform.position.x, v1.y - transform.position.y),
            vec2(v2.x - transform.position.x, v2.y - transform.position.y),
        );
        self.world.add_component_to_entity(
            triangle,
            Geometry {
                geometry_type: GeometryType::Triangle,
                vertices: vec![Triangle {
                    position: position.into(),
                    rotation: rotation,
                    v0: v0.into(),
                    v1: v1.into(),
                    v2: v2.into(),
                    c0: c0.into(),
                    c1: c1.into(),
                    c2: c2.into(),
                    ..Default::default()
                }],
            },
        );
        self.world.add_component_to_entity(triangle, transform);
        self.world.add_component_to_entity(
            triangle,
            Renderable {
                color: TriangleColorMap {
                    c0: Color::from_vec4(c0),
                    c1: Color::from_vec4(c1),
                    c2: Color::from_vec4(c2),
                },
                visible: true,
            },
        );
    }

    pub fn update_camera_position(&mut self, delta: Vec2) {
        let mut camera_query = self.world.query_mut::<(&mut Camera, &mut Transformation)>();
        let (_camera, transform) = camera_query.next().expect("No camera created."); // SHOULD only be one camera, but we get the first one regardless.
        transform.position += delta;
        self.wgpu_ctx.as_mut().unwrap().update_camera_position(transform.position);

        self.render_step();
    }

    pub fn update_pipeline(&mut self) {
        self.wgpu_ctx.as_mut().unwrap().sync_tri_instances();
    }

    pub fn create_camera(&mut self) {
        let camera = self.world.new_entity();
        self.world.add_component_to_entity(camera, Camera {});
        self.world.add_component_to_entity(camera, Transformation {
            position: vec2(0.5, 0.5),
            ..Default::default()
        });
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes()
                .with_title("wgpu winit example")
                .with_inner_size(LogicalSize::new(800., 800.));
            // use Arc. --I forgot why this note is here, thanks past me.
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
            self.world = World::default();
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
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        logical_key,
                        text,
                        location,
                        repeat,
                        ..
                    },
                ..
            } => {
                if !key_state.is_pressed() {return}
                match code {
                    KeyCode::KeyW => self.update_camera_position(vec2(0.0, -0.05)),
                    KeyCode::KeyS => self.update_camera_position(vec2(0.0, 0.05)),
                    KeyCode::KeyA => self.update_camera_position(vec2(0.05, 0.0)),
                    KeyCode::KeyD => self.update_camera_position(vec2(-0.05, 0.0)),

                    _ => (),
                }
            }, //Handle any keypresses here!!!
            WindowEvent::MouseInput {
                device_id: _device_id,
                state,
                button,
            } => (), //Handle mouse input here!
            _ => (),
        }
    }
}

//Takes a coordinate in pixels and converts it to a screen-scaled vector2 based on the display size
pub fn scale_to_screen(display_size: Vec2, position: Vec2) -> Vec2 {
    let scaled = vec2(
        ((position.x / display_size.x) * 2.0) - 1.0,
        ((position.y / display_size.y) * 2.0) - 1.0,
    );
    scaled
}

//Takes a full pixel object and returns the screen-scaled vector2
pub fn scale_pixel(display_size: Vec2, pixel: Vec2) -> Vec2 {
    (pixel / display_size) * 2.0
}
