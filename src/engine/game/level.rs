use std::io::Write;
use std::fs::File;
use serde_json::{from_reader, to_writer};
use glam::Vec2;

use crate::engine::{ecs::{Component, Entity, world::World}, game::{Collidable, Renderable, Transformation}, render::color::Color};
use crate::read_file;

#[derive(Debug)]
pub struct LevelData {
    pub level: String,
}

impl Component for LevelData {}

pub const MAP_DATA_PATH: &str = "src/mapdata/";

pub fn load_level_data(level_name: &str, world: &mut World) {
    
}

pub fn load_legacy_data(legacy_name: &str, new_name: &str, world: &mut World) {
    let legacy_data = read_file((MAP_DATA_PATH.to_owned() + legacy_name).as_str());

    for object in legacy_data.iter() {
        let data: Vec<&str> = object.split_terminator(';').collect();
        let object_entity = world.new_entity();
        world.add_component_to_entity(object_entity, LevelData {level: new_name.to_string()});
        
        let position = (u16::from_str_radix(data[0], 10).unwrap(), u16::from_str_radix(data[1], 10).unwrap());
        world.add_component_to_entity(object_entity, Transformation {
            position: Vec2::new(position.0 as f32, position.1 as f32),
            velocity: Vec2::ZERO,
        });
        match data[2].to_lowercase().as_str() {
            "air" => world.add_component_to_entity(object_entity, Renderable {
                color: Color::BLACK,
                ..Default::default()
            }),
            "wall" => {
                world.add_component_to_entity(object_entity, Renderable {
                    color: Color::WHITE,
                    ..Default::default()
                });
                world.add_component_to_entity(object_entity, Collidable);
                //world.spacial_tree.insert_pt(Point {x: position.0, y: position.1}, object_entity);
            },
            _ => (),
        }
    }
}