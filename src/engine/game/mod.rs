use glam::Vec2;
use crate::engine::ecs::Component;
use crate::engine::render::color::Color;

pub mod level;

//Determines what shape an object has for rendering
//stores triangle or circle data within enum
#[derive(Debug, Clone)]
pub enum Geometry {
    Triangle,
    Circle,
}

#[derive(Debug, Clone)]
pub struct Transformation {
    pub position: Vec2,
    pub velocity: Vec2,
    //Rotation?
}
impl Component for Transformation {}

impl Default for Transformation {
    fn default() -> Self {
        Transformation { position: Vec2::ZERO, velocity: Vec2::ZERO }
    }
}

#[derive(Debug)]
pub struct Renderable {
    pub color: Color,
    pub geometry: Geometry,
    pub visible: bool,
}
impl Component for Renderable {}

impl Default for Renderable {
    fn default() -> Self {
        Renderable {
            color: Color::WHITE,
            geometry: Geometry::Triangle,
            visible: true,
        }
    } 
}

#[derive(Debug)]
pub struct Collidable;
impl Component for Collidable {}