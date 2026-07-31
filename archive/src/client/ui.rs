use std::io::{Write, stdout};
use crossterm::{execute, queue, style::{self, Stylize}, terminal, cursor};
use crate::engine::{ecs::{world::World, Entity}, game::{Renderable, Transformation}, math::Vec2};

pub const SCREEN_SIZE_X: u16 = 41;
pub const SCREEN_SIZE_Y: u16 = 21;

pub const DEBUG_PRINT: bool = false;

#[derive(Debug)]
pub struct TextBox {
    contents: String,
    size: (u16, u16),
    paading: (u16, u16),
}

pub struct Window {
    size: (u16, u16),
    position: (u16, u16),
    contents: Vec<Option<TextBox>>,
    visible: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub position: (u16, u16),
    pub layer: usize,
    pub fg_color: style::Color,
    pub bg_color: style::Color,
    pub content: char,
}

impl Pixel {
    pub fn from_char(character: char) -> Pixel {
        Pixel {
            content: character,
            ..Default::default()
        }
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            position: (0, 0),
            layer: 0,
            content: ' ',
            fg_color: style::Color::White,
            bg_color: style::Color::Black,
        }
    }
}

#[derive(Debug)]
pub struct Screen {
    screen_buffer: [Pixel; (SCREEN_SIZE_X * SCREEN_SIZE_Y) as usize],
    draw_queue: Vec<Pixel>,
    //swap_buffer: [Pixel; (SCREEN_SIZE_X * SCREEN_SIZE_Y) as usize],
}

impl Screen {
    pub fn new() -> Screen {
        queue!(stdout(), terminal::Clear(terminal::ClearType::Purge), terminal::SetSize(SCREEN_SIZE_X, SCREEN_SIZE_Y), cursor::MoveTo(0,0), cursor::Hide).unwrap();
        stdout().flush().unwrap();
        if !DEBUG_PRINT {
            terminal::enable_raw_mode().unwrap();
        }
        
        Screen {
            screen_buffer: [Pixel::default(); (SCREEN_SIZE_X * SCREEN_SIZE_Y) as usize],
            draw_queue: Vec::new(),
        }
    }

    //During render pass, queue order is determined by transformation "layer".
    // Higher layers are drawn on top, and sorted into lower indexes of the vec
    pub fn render_pass(&mut self, world: &mut World, target: Entity) {
        let local_center = world.get_entity_component::<Transformation>(target).expect("No transformation on render target.").clone();
        let renderable = world.query::<(&Transformation, &Renderable)>();
        for (transform, render_data) in renderable {
            let relative_position = self.get_relative_transform(&local_center, transform);
            let screen_position = &Vec2::new((SCREEN_SIZE_X / 2) as i32, (SCREEN_SIZE_Y / 2) as i32) + &relative_position.position;
            if (screen_position.x > 0 && screen_position.x < SCREEN_SIZE_X as i32) && (screen_position.y > 0 && screen_position.y < SCREEN_SIZE_Y as i32) {
                let pixel: Pixel = Pixel {
                    position: screen_position.into(),
                    layer: transform.layer,
                    fg_color: render_data.fg_color,
                    bg_color: render_data.bg_color,
                    content: render_data.character,
                };

                self.draw_at(pixel);
            }
        }
        self.draw_queue.sort_by(|pixel1, pixel2| {
            pixel2.layer.cmp(&pixel1.layer)
        });
        
        if !DEBUG_PRINT {
            self.clear_buffer();
        } else {
            println!("{:#?}", self.draw_queue);
            self.draw_queue = Vec::new();
        }
        while self.draw_queue.len() > 0 && !DEBUG_PRINT {
            if let Some(pixel) = self.draw_queue.pop() {
                queue!(stdout(), cursor::MoveTo(pixel.position.0, pixel.position.1), style::PrintStyledContent(pixel.content.with(pixel.fg_color).on(pixel.bg_color))).unwrap();
            }
        }
        stdout().flush().unwrap();
    }
    
    pub fn draw_buffer(&mut self) {
        for pixel in self.screen_buffer.iter() {
            self.draw_queue.push(pixel.clone());
        }
    }
    /*
    pub fn swap(&mut self) {
        let _ = std::mem::replace(&mut self.screen_buffer, self.swap_buffer);
    }*/

    pub fn clear(&self) {
        execute!(stdout(), cursor::MoveTo(0, 0), terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0),).unwrap();
        stdout().flush().unwrap();
    }

    pub fn clear_buffer(&mut self) {
        self.screen_buffer = [Pixel::default(); (SCREEN_SIZE_X * SCREEN_SIZE_Y) as usize];
    }

    pub fn get_index(&self, position: (u16, u16)) -> usize {
        return ((position.1 * SCREEN_SIZE_X) + position.0) as usize;
    }

    pub fn get_position(&self, index: usize) -> (u16, u16) {
        return (index as u16 % SCREEN_SIZE_X, index as u16 / SCREEN_SIZE_X);
    }

    pub fn get_relative_transform(&self, local_space: &Transformation, world_space: &Transformation) -> Transformation {
        Transformation {
            position: &world_space.position - &local_space.position,
            layer: world_space.layer - local_space.layer,
            velocity: &world_space.velocity - &local_space.velocity,
        }
    }

    pub fn draw_at(&mut self, pixel: Pixel) {
        self.screen_buffer[self.get_index((pixel.position.0, pixel.position.1))] = pixel;
        self.draw_queue.push(pixel);
    }
}