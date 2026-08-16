use crate::engine::render::Triangle;
use crate::engine::render::color::Color;
use crate::engine::{ecs::Component, render::color::TriangleColorMap};
use glam::{Vec2, vec2, vec4};

pub mod level;

//Determines what shape an object has for rendering
//stores triangle or circle data within enum
#[derive(Debug, Clone)]
pub enum GeometryType {
    Triangle,
    Circle, //UNIMPLEMENTED
    Mesh,   //UNIMPLEMENTED TYPE
}
#[derive(Debug, Clone)]
pub struct Geometry {
    pub geometry_type: GeometryType,
    pub vertices: Vec<Triangle>,
}
impl Component for Geometry {}

impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            geometry_type: GeometryType::Triangle,
            vertices: vec![Triangle::new(
                vec2(0.0, 0.0),
                vec2(0.0, 0.0),
                vec2(0.0, 0.0),
                TriangleColorMap::flat(vec4(1.0, 1.0, 1.0, 1.0)),
            )],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transformation {
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
}
impl Component for Transformation {}

impl Default for Transformation {
    fn default() -> Self {
        Transformation {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
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
pub struct Camera {}
impl Component for Camera {}

#[derive(Debug)]
pub struct Collidable;
impl Component for Collidable {}
