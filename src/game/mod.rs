use crate::engine::ecs::spatial::AABB;
use crate::engine::render::Triangle;
use crate::engine::render::color::Color;
use crate::engine::{ecs::Component, render::color::TriangleColorMap};
use glam::{Vec2, Vec3, vec2, vec3, vec4};

pub mod level;
pub mod player;

pub const SIZE_SCALAR: f32 = 40.0;
pub const GRID_CELL_SIZE: f32 = 1.0 / SIZE_SCALAR;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
}

//Determines what shape an object has for rendering
//stores triangle or circle data within enum
#[derive(Debug, Clone)]
pub enum GeometryType {
    Triangle(Triangle),
    Cell(Triangle, Triangle),
    Circle(Vec<Triangle>), //UNIMPLEMENTED
    Mesh(Vec<Triangle>),   //UNIMPLEMENTED TYPE
}
#[derive(Debug, Clone)]
pub struct Geometry {
    pub geometry_type: GeometryType,
}
impl Component for Geometry {}

impl Geometry {
    pub fn get_bounds(&self) -> AABB {
        match self.geometry_type {
            GeometryType::Cell(t1, t2) => {
                let verts = [t1.v0, t1.v1, t1.v2, t2.v0, t2.v1, t2.v2].map(Vec2::from);

                let mut min = Vec2::splat(f32::MAX);
                let mut max = Vec2::splat(f32::MIN);
                for v in verts {
                    min = min.min(v);
                    max = max.max(v);
                }

                AABB::new_raw(min, max)
            }

            _ => AABB::ZERO,
        }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            geometry_type: GeometryType::Triangle(Triangle::new(
                vec2(0.0, 0.0),
                vec2(0.0, 0.0),
                vec2(0.0, 0.0),
                TriangleColorMap::flat(vec4(1.0, 1.0, 1.0, 1.0)),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transformation {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: f32,
}
impl Component for Transformation {}

impl Default for Transformation {
    fn default() -> Self {
        Transformation {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            rotation: 0.0_f32,
        }
    }
}

#[derive(Debug)]
pub struct Renderable {
    pub color: TriangleColorMap, //A colormap that describes the color of each vertex in the geometry
    pub visible: bool,
}
impl Component for Renderable {}

impl Default for Renderable {
    fn default() -> Self {
        Renderable {
            color: TriangleColorMap::flat(Color::WHITE.as_vec4()),
            visible: true,
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    pub synced: bool,
}
impl Component for Camera {}

impl Default for Camera {
    fn default() -> Self {
        Camera { synced: true }
    }
}

#[derive(Debug)]
pub struct Collidable {
    pub overwrite: bool,
}
impl Component for Collidable {}

impl Default for Collidable {
    fn default() -> Self {
        Collidable { overwrite: false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Component for WorldPosition {}

impl WorldPosition {
    pub fn new(x: i32, y: i32, z: i32) -> WorldPosition {
        WorldPosition { x, y, z }
    }

    pub fn to_tuple2d(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn to_render(&self) -> Vec3 {
        vec3(
            self.x as f32 * GRID_CELL_SIZE + GRID_CELL_SIZE / 2.0,
            self.y as f32 * GRID_CELL_SIZE + GRID_CELL_SIZE / 2.0,
            self.z as f32,
        )
    }

    pub fn from_render(p: Vec3) -> Self {
        Self {
            x: (p.x / GRID_CELL_SIZE).floor() as i32,
            y: (p.y / GRID_CELL_SIZE).floor() as i32,
            z: p.z.round() as i32,
        }
    }
}

impl Default for WorldPosition {
    fn default() -> Self {
        WorldPosition { x: 0, y: 0, z: 0 }
    }
}

impl From<(u16, u16)> for WorldPosition {
    fn from(value: (u16, u16)) -> Self {
        WorldPosition {
            x: value.0 as i32,
            y: value.1 as i32,
            z: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InteractionType {
    Door,
    Null,
}

#[derive(Debug, Clone)]
pub struct Interactable {
    pub interaction_type: InteractionType,
}
impl Component for Interactable {}

impl Interactable {
    pub const DOOR: Interactable = Interactable {
        interaction_type: InteractionType::Door,
    };
}

impl Default for Interactable {
    fn default() -> Self {
        Interactable {
            interaction_type: InteractionType::Null,
        }
    }
}
