use crossterm::style;
use crate::{client::ui::Pixel, engine::{math::Vec2, ecs::Component}};

pub mod level;

#[derive(Debug, Clone)]
pub struct Transformation {
    pub position: Vec2,
    pub layer: usize,
    pub velocity: Vec2,
    //Rotation?
}
impl Component for Transformation {}

impl Default for Transformation {
    fn default() -> Self {
        Transformation { position: Vec2::zero(), layer: 0, velocity: Vec2::zero() }
    }
}

#[derive(Debug)]
pub struct Renderable {
    pub character: char,
    pub fg_color: style::Color,
    pub bg_color: style::Color,
    pub visible: bool,
}
impl Component for Renderable {}

impl From<&Renderable> for Pixel {
    fn from(value: &Renderable) -> Self {
        Pixel {
            content: value.character,
            fg_color: value.fg_color,
            bg_color: value.bg_color,
            ..Default::default()
        }
    }
}

impl Default for Renderable {
    fn default() -> Self {
        Renderable {
            character: ' ',
            fg_color: style::Color::White,
            bg_color: style::Color::Black,
            visible: true,
        }
    } 
}

#[derive(Debug)]
pub struct Collidable;
impl Component for Collidable {}