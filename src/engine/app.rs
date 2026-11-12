use std::f32::consts::PI;
use std::sync::Arc;

use glam::{Vec2, Vec3, Vec3Swizzles, Vec4, vec2};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::engine::ecs::Entity;
use crate::engine::ecs::world::World;
use crate::engine::math::get_rectangle_triangles;
use crate::engine::render::color::Color;
use crate::engine::render::wgpu_ctx::WgpuCtx;
use crate::engine::render::{Triangle, color::TriangleColorMap};
use crate::engine::ui::{Gui, MainMenuAction};
use crate::game::level::{convert_legacy_levels, load_level};
use crate::game::{Camera, Geometry, Renderable, Transformation, player::PlayerControl};
use crate::game::{Collidable, GRID_CELL_SIZE, GameState, Interactable, InteractionType};
use crate::game::{GeometryType, WorldPosition};

#[derive(Default, Debug)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    gui: Option<Gui>,
    pub display_size: Vec2,
    pub world: World,
    pub state: GameState,
}

//NON-GAME SPECIFIC IMPLEMENTATIONS
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

            match &geometry.geometry_type {
                GeometryType::Triangle(tri) => ctx.tri_object_store.push(Triangle {
                    position: transform.position.into(),
                    rotation: transform.rotation,
                    v0: tri.v0,
                    v1: tri.v1,
                    v2: tri.v2,
                    c0: tri.c0,
                    c1: tri.c1,
                    c2: tri.c2,
                    ..Default::default()
                }),
                GeometryType::Cell(tri1, tri2) => {
                    ctx.tri_object_store.push(Triangle {
                        position: transform.position.into(),
                        rotation: transform.rotation,
                        v0: tri1.v0,
                        v1: tri1.v1,
                        v2: tri1.v2,
                        c0: tri1.c0,
                        c1: tri1.c1,
                        c2: tri1.c2,
                        ..Default::default()
                    });
                    ctx.tri_object_store.push(Triangle {
                        position: transform.position.into(),
                        rotation: transform.rotation,
                        v0: tri2.v0,
                        v1: tri2.v1,
                        v2: tri2.v2,
                        c0: tri2.c0,
                        c1: tri2.c1,
                        c2: tri2.c2,
                        ..Default::default()
                    });
                }
                GeometryType::Circle(triangles) => {
                    for tri in triangles {
                        ctx.tri_object_store.push(Triangle {
                            position: transform.position.into(),
                            rotation: transform.rotation,
                            v0: tri.v0,
                            v1: tri.v1,
                            v2: tri.v2,
                            c0: tri.c0,
                            c1: tri.c1,
                            c2: tri.c2,
                            ..Default::default()
                        })
                    }
                }
                GeometryType::Mesh(triangles) => {
                    for tri in triangles {
                        ctx.tri_object_store.push(Triangle {
                            position: transform.position.into(),
                            rotation: transform.rotation,
                            v0: tri.v0,
                            v1: tri.v1,
                            v2: tri.v2,
                            c0: tri.c0,
                            c1: tri.c1,
                            c2: tri.c2,
                            ..Default::default()
                        })
                    }
                }
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

        let (t1, t2) = get_rectangle_triangles(
            scaled_position,
            scaled_components.x,
            scaled_components.y,
            TriangleColorMap::flat(color),
        );

        let rectangle = self.world.new_entity();
        let transform = Transformation {
            position: midpoint.extend(0.0),
            velocity: Vec3::ZERO,
            rotation: 0.0_f32,
        };
        self.world.add_component_to_entity(
            rectangle,
            Geometry {
                geometry_type: GeometryType::Mesh(vec![t1, t2]),
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
            position: midpoint.extend(0.0),
            velocity: Vec3::ZERO,
            rotation: 0.0_f32,
        };
        self.world.add_component_to_entity(
            line,
            Geometry {
                geometry_type: GeometryType::Mesh(vec![t1, t2]),
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
            position: position.extend(0.0),
            velocity: Vec3::ZERO,
            rotation: 0.0_f32,
        };
        self.world.add_component_to_entity(
            circle,
            Geometry {
                geometry_type: GeometryType::Circle(triangles),
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
            position: v0.midpoint(v1).midpoint(v2).extend(0.0),
            velocity: Vec3::ZERO,
            rotation: 0.0_f32,
        };
        self.world.add_component_to_entity(
            triangle,
            Geometry {
                geometry_type: GeometryType::Triangle(Triangle::new(
                    v0 - transform.position.xy(),
                    v1 - transform.position.xy(),
                    v2 - transform.position.xy(),
                    color_map,
                )),
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
            )
            .extend(0.0),
            velocity: Vec3::ZERO,
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
                geometry_type: GeometryType::Triangle(Triangle {
                    position: position.extend(0.0).into(),
                    rotation: rotation,
                    v0: v0.into(),
                    v1: v1.into(),
                    v2: v2.into(),
                    c0: c0.into(),
                    c1: c1.into(),
                    c2: c2.into(),
                    ..Default::default()
                }),
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

    pub fn create_camera(&mut self) {
        let camera = self.world.new_entity();
        let grid_position = WorldPosition::new(0, 0, 0);
        let midpoint = grid_position.to_render();

        self.world
            .add_component_to_entity(camera, Camera::default());
        self.world.add_component_to_entity(camera, grid_position);
        self.world.add_component_to_entity(
            camera,
            Transformation {
                position: midpoint,
                ..Default::default()
            },
        );
    }

    pub fn update_camera_position(&mut self, delta_x: i32, delta_y: i32) {
        let mut camera_query = self
            .world
            .query_mut::<(&mut Camera, &mut WorldPosition, &mut Transformation)>();
        let (_camera, world_position, transform) = camera_query.next().expect("No camera created."); // SHOULD only be one camera, but we get the first one regardless.

        world_position.x += delta_x;
        world_position.y += delta_y;

        transform.position = world_position.to_render().with_z(transform.position.z);
        self.wgpu_ctx
            .as_mut()
            .unwrap()
            .update_camera_position(transform.position);

        self.render_step();
    }

    pub fn start_level(&mut self, name: &str) {
        self.world = World::default();
        self.create_camera();
        self.create_player(5, 5, 1);
        if let Err(error) = load_level(name, &mut self.world) {
            eprintln!("Failed to load level '{name}': {error}");
            self.world = World::default();
            return;
        }

        if let Some(ctx) = self.wgpu_ctx.as_mut() {
            ctx.update_camera_position(WorldPosition::default().to_render());
        }
        self.state = GameState::Playing;
        self.render_step();
    }

    pub fn return_to_menu(&mut self) {
        self.world = World::default();
        self.state = GameState::MainMenu;
        if let Some(ctx) = self.wgpu_ctx.as_mut() {
            ctx.tri_object_store.clear();
            ctx.sync_tri_instances();
        }
        if let Some(gui) = self.gui.as_mut() {
            gui.refresh_levels();
        }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn draw_menu(&mut self) -> MainMenuAction {
        let (Some(ctx), Some(gui), Some(window)) = (
            self.wgpu_ctx.as_mut(),
            self.gui.as_mut(),
            self.window.as_ref(),
        ) else {
            return MainMenuAction::None;
        };
        let mut action = MainMenuAction::None;
        let mut repaint = false;
        ctx.draw_with(|target| {
            repaint = gui.render_main_menu(window, target, &mut action);
        });
        if repaint {
            window.request_redraw();
        }
        action
    }

    //Updates the pipline and sorts the instance buffer by depth
    pub fn update_pipeline(&mut self) {
        self.wgpu_ctx.as_mut().unwrap().sync_tri_instances();
    }
}

//GAME-SPECICI IMPLEMENTATIONS
impl<'window> App<'window> {
    pub fn sync_camera_position(&mut self) {
        let (player_position, player_transform) = {
            let mut player_query = self
                .world
                .query::<(&PlayerControl, &WorldPosition, &Transformation)>();
            let (_player, player_position, player_transform) =
                player_query.next().expect("No player found.");

            (player_position.clone(), player_transform.clone())
        };

        let mut camera_query = self
            .world
            .query_mut::<(&mut Camera, &mut WorldPosition, &mut Transformation)>();
        let (camera, camera_position, camera_transform) =
            camera_query.next().expect("No camera created."); // SHOULD only be one camera, but we get the first one regardless.
        camera.synced = !camera.synced;

        *camera_position = player_position;
        *camera_transform = player_transform;
        self.wgpu_ctx
            .as_mut()
            .unwrap()
            .update_camera_position(camera_transform.position);

        self.render_step();
    }

    pub fn create_player(&mut self, pos_x: i32, pos_y: i32, z_index: i32) {
        let player_color = TriangleColorMap::flat(Color::GREEN.as_vec4());
        let player_size = GRID_CELL_SIZE;
        let world_position = WorldPosition::new(pos_x, pos_y, z_index);
        let midpoint = world_position.to_render();

        let corner = vec2(
            midpoint.x - player_size / 2.0,
            midpoint.y - player_size / 2.0,
        );

        let player = self.world.new_entity();
        self.world.add_component_to_entity(player, PlayerControl);
        self.world.add_component_to_entity(player, world_position);
        self.world.add_component_to_entity(
            player,
            Transformation {
                position: midpoint,
                velocity: Vec3::ZERO,
                rotation: 0.0,
            },
        );
        self.world.add_component_to_entity(
            player,
            Renderable {
                color: player_color,
                visible: true,
            },
        );
        let (t1, t2) = get_rectangle_triangles(corner, player_size, player_size, player_color);
        self.world.add_component_to_entity(
            player,
            Geometry {
                geometry_type: GeometryType::Cell(t1, t2),
            },
        );
    }

    pub fn update_player_position(&mut self, delta_x: i32, delta_y: i32) -> bool {
        let mut new_position = {
            let mut player_query = self
                .world
                .query::<(&PlayerControl, &WorldPosition, &Transformation)>();
            let (_player, world_position, _transform) =
                player_query.next().expect("No player found.");

            world_position.clone()
        };
        new_position.x += delta_x;
        new_position.y += delta_y;

        if self.world.check_grid_collision(new_position.to_tuple2d()) {
            return false;
        }

        let mut player_query =
            self.world
                .query_mut::<(&mut PlayerControl, &mut WorldPosition, &mut Transformation)>();
        let (_player, world_position, transform) = player_query.next().expect("No player found.");
        transform.position = new_position.to_render().with_z(transform.position.z);
        *world_position = new_position;

        //PLAYER STILL UNABLE TO MOVE, TRANSFORM POSITION NOT UPDATING PROPERLY?

        self.render_step();
        true
    }

    pub fn handle_player_movement(&mut self, delta_x: i32, delta_y: i32, camera_movement: bool) {
        let mut camera_query = self
            .world
            .query::<(&Camera, &WorldPosition, &Transformation)>();
        let (camera, _world_position, _transform) =
            camera_query.next().expect("No camera created."); // SHOULD only be one camera, but we get the first one regardless.
        if camera.synced {
            if !self.update_player_position(delta_x, delta_y) {
                return;
            };
            self.update_camera_position(delta_x, delta_y);
            return;
        }
        if camera_movement {
            self.update_camera_position(delta_x, delta_y);
        } else {
            self.update_player_position(delta_x, delta_y);
        }
    }

    pub fn handle_interaction(&mut self) {
        //get player and find cardinally neighboring cells, query for entities at those positions with Interactable
        //  from queried entities, match interaction type and do logic based on the interaction type
        let player_position = {
            let mut player_query = self
                .world
                .query::<(&PlayerControl, &WorldPosition, &Transformation)>();
            let (_player, world_position, _transform) =
                player_query.next().expect("No player found.");

            world_position.clone()
        };

        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let entities: Vec<Entity> = self
                .world
                .collision_grid
                .get(&(player_position.x + dx, player_position.y + dy))
                .cloned()
                .unwrap_or_default();

            for entity in entities {
                if let Some(interactable) = self
                    .world
                    .get_entity_component::<Interactable>(entity)
                    .cloned()
                {
                    match interactable.interaction_type {
                        InteractionType::Door => {
                            let collidable = self
                                .world
                                .get_entity_component::<Collidable>(entity)
                                .unwrap();
                            collidable.overwrite = !collidable.overwrite;
                        }
                        _ => println!("Unhandled interaction!"),
                    }
                }
            }
        }
        self.render_step();
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
            self.gui = Some(Gui::new(&window, wgpu_ctx.device(), wgpu_ctx.surface_format()));
            self.wgpu_ctx = Some(wgpu_ctx);
            self.world = World::default();
            
            if let Some(gui) = self.gui.as_mut() {
                gui.refresh_levels();
            }
            self.state = GameState::MainMenu;
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let in_menu = self.state == GameState::MainMenu;
        if let (Some(gui), Some(window)) = (self.gui.as_mut(), self.window.as_ref()) {
            let response = gui.on_window_event(window, &event);
            if response.repaint && in_menu {
                window.request_redraw();
            }
            if response.consumed {
                return;
            }
        }
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
            WindowEvent::RedrawRequested => match self.state {
                GameState::MainMenu => match self.draw_menu() {
                    MainMenuAction::Load(name) => self.start_level(&name),
                    MainMenuAction::Quit => event_loop.exit(),
                    MainMenuAction::None => {}
                },
                GameState::Playing => {
                    if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                        wgpu_ctx.draw();
                    }
                }
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        logical_key: _,
                        text: _,
                        location: _,
                        repeat: _,
                        ..
                    },
                ..
            } => {
                if !key_state.is_pressed() || self.state != GameState::Playing {
                    return;
                }
                match code {
                    KeyCode::Escape => return self.return_to_menu(),
                    KeyCode::ArrowUp => self.handle_player_movement(0, 1, true),
                    KeyCode::ArrowDown => self.handle_player_movement(0, -1, true),
                    KeyCode::ArrowLeft => self.handle_player_movement(-1, 0, true),
                    KeyCode::ArrowRight => self.handle_player_movement(1, 0, true),
                    KeyCode::KeyW => self.handle_player_movement(0, 1, false),
                    KeyCode::KeyS => self.handle_player_movement(0, -1, false),
                    KeyCode::KeyA => self.handle_player_movement(-1, 0, false),
                    KeyCode::KeyD => self.handle_player_movement(1, 0, false),
                    KeyCode::KeyE => self.handle_interaction(),
                    KeyCode::KeyL => self.sync_camera_position(),
                    _ => (),
                }
            } //Handle any keypresses here!!!
            WindowEvent::MouseInput {
                device_id: _device_id,
                state: _,
                button: _,
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
