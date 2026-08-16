use glam::{Vec2, Vec4};
use serde_json::{from_reader, to_writer};
use std::fs::File;
use std::io::Write;

use crate::engine::{
    ecs::{Component, Entity, world::World}, game::{Collidable, Geometry, Renderable, Transformation}, math::get_rectangle_triangles, render::color::{Color, TriangleColorMap},
};
use crate::read_file;

pub const SIZE_SCALAR: f32 = 40.0;

#[derive(Debug)]
pub struct LevelData {
    pub level: String,
}

impl Component for LevelData {}

pub const MAP_DATA_PATH: &str = "src/mapdata/";

pub fn load_level_data(level_name: &str, world: &mut World) {}

pub fn load_legacy_data(legacy_name: &str, new_name: &str, world: &mut World) {
    let legacy_data = read_file((MAP_DATA_PATH.to_owned() + legacy_name).as_str());

    for object in legacy_data.iter() {
        let data: Vec<&str> = object.split_terminator(';').collect();
        let object_entity = world.new_entity();
        world.add_component_to_entity(
            object_entity,
            LevelData {
                level: new_name.to_string(),
            },
        );

        let position = (
            u16::from_str_radix(data[0], 10).unwrap() as f32 / SIZE_SCALAR,
            u16::from_str_radix(data[1], 10).unwrap() as f32 / SIZE_SCALAR,
        );
        world.add_component_to_entity(
            object_entity,
            Transformation {
                position: Vec2::new(position.0, position.1),
                velocity: Vec2::ZERO,
                rotation: 0.0_f32,
            },
        );
        match data[2].to_lowercase().as_str() {
            "air" => {
                world.add_component_to_entity(
                object_entity,
                Renderable {
                    color: TriangleColorMap::flat(Vec4::ZERO),
                    ..Default::default()
                });
                let (t1, t2) = get_rectangle_triangles(position.into(), 1.0, 1.0, Vec4::ZERO);
                world.add_component_to_entity(
                object_entity,
                Geometry {
                    geometry_type: super::GeometryType::Triangle,
                    vertices: vec!(t1, t2),
                });
            },
            "wall" => {
                world.add_component_to_entity(
                    object_entity,
                    Renderable {
                        color: TriangleColorMap::flat(Vec4::ONE),
                        ..Default::default()
                    },
                );
                world.add_component_to_entity(object_entity, Collidable);
                let (t1, t2) = get_rectangle_triangles(position.into(), 1.0, 1.0, Vec4::ONE);
                world.add_component_to_entity(
                object_entity,
                Geometry {
                    geometry_type: super::GeometryType::Triangle,
                    vertices: vec!(t1, t2),
                });
                //world.spacial_tree.insert_pt(Point {x: position.0, y: position.1}, object_entity);
            }
            _ => (),
        }
    }
}
