use std::io::Write;

use crate::{engine::{
    ecs::{Component, Entity, world::World},
    math::get_rectangle_triangles,
    render::color::{Color, TriangleColorMap},
}, game::{Interactable, WorldPosition}};
use crate::game::{Collidable, GRID_CELL_SIZE, Geometry, Renderable, SIZE_SCALAR, Transformation, level};
use crate::read_file;
use glam::{Vec2, Vec3, Vec3Swizzles, Vec4, vec2};
use serde::{Deserialize, Serialize, Serializer};


#[derive(Debug, Serialize, Deserialize)]
pub struct LegacyObjectData {
    position: (i32, i32),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LegacyObjectType {
    Wall,
    Door,
    Air,
}

#[derive(Debug)]
pub struct LevelData {
    pub level: String,
    pub otype: LegacyObjectType,
}
impl Component for LevelData {}

pub const MAP_DATA_PATH: &str = "src/mapdata/";

pub fn load_level_data(level_name: &str, world: &mut World) {}

pub fn load_legacy_data(legacy_name: &str, new_name: &str, world: &mut World) {
    let legacy_data = read_file((MAP_DATA_PATH.to_owned() + legacy_name).as_str());

    for object in legacy_data.iter() {
        let data: Vec<&str> = object.split_terminator(';').collect();
        let object_entity = world.new_entity();

        let world_position = WorldPosition::from((
            u16::from_str_radix(data[0], 10).unwrap(), 
            u16::from_str_radix(data[1], 10).unwrap()
        ));
        let render_pos = world_position.to_render().with_z(5.0);
        let corner = vec2(
            render_pos.x - GRID_CELL_SIZE / 2.0,
            render_pos.y - GRID_CELL_SIZE / 2.0,
        );

        world.add_component_to_entity(
            object_entity,
            Transformation {
                position: render_pos,
                velocity: Vec3::ZERO,
                rotation: 0.0_f32,
            },
        );
        world.add_component_to_entity(
            object_entity, 
            world_position,
        );
        match data[2].to_lowercase().as_str() {
            "air" => {
                
                world.add_component_to_entity(
                    object_entity,
                    LevelData {
                        level: new_name.to_string(),
                        otype: LegacyObjectType::Air,
                    },
                );
                world.add_component_to_entity(
                    object_entity,
                    Renderable {
                        color: TriangleColorMap::flat(Vec4::ZERO),
                        ..Default::default()
                    },
                );
                let (t1, t2) = get_rectangle_triangles(
                    corner.into(),
                    GRID_CELL_SIZE,
                    GRID_CELL_SIZE,
                    TriangleColorMap::flat(Vec4::ZERO),
                );
                world.add_component_to_entity(
                    object_entity,
                    Geometry {
                        geometry_type: super::GeometryType::Cell(t1, t2),
                    },
                );
            }
            "wall" => {
                world.add_component_to_entity(
                    object_entity,
                    LevelData {
                        level: new_name.to_string(),
                        otype: LegacyObjectType::Wall,
                    },
                );
                world.add_component_to_entity(
                    object_entity,
                    Renderable {
                        color: TriangleColorMap::flat(Vec4::ONE),
                        ..Default::default()
                    },
                );
                world.add_component_to_entity(object_entity, Collidable::default());
                let (t1, t2) = get_rectangle_triangles(
                    corner.into(),
                    GRID_CELL_SIZE,
                    GRID_CELL_SIZE,
                    TriangleColorMap::flat(Vec4::ONE),
                );
                world.add_component_to_entity(
                    object_entity,
                    Geometry {
                        geometry_type: super::GeometryType::Cell(t1, t2),
                    },
                );
                world.add_component_to_entity(object_entity, Collidable::default());
                //world.spacial_tree.insert_pt(Point {x: position.0, y: position.1}, object_entity);
            }
            "door" => {
                world.add_component_to_entity(
                    object_entity,
                    LevelData {
                        level: new_name.to_string(),
                        otype: LegacyObjectType::Wall,
                    },
                );
                world.add_component_to_entity(
                    object_entity,
                    Renderable {
                        color: TriangleColorMap::flat(Color::BROWN.as_vec4()),
                        ..Default::default()
                    },
                );
                world.add_component_to_entity(object_entity, Collidable::default());
                world.add_component_to_entity(object_entity, Interactable::DOOR);
                let (t1, t2) = get_rectangle_triangles(
                    corner.into(),
                    GRID_CELL_SIZE,
                    GRID_CELL_SIZE,
                    TriangleColorMap::flat(Color::BROWN.as_vec4()),
                );
                world.add_component_to_entity(
                    object_entity,
                    Geometry {
                        geometry_type: super::GeometryType::Cell(t1, t2),
                    },
                );
            }
            _ => (),
        }
    }
    world.create_collision_grid();
}


//Writes 
//Drops z_index when saving data
pub fn save_world_data(name: &str, world: &World) {
    let level_data = world.query::<(&LevelData, &WorldPosition)>();

    let mut target_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&format!("src/mapdata/{name}.json"))
        .expect("Error opening map data file.");
    let mut object_collection: Vec<LegacyObjectData> = Vec::new();
   
    for (level_data, world_position) in level_data {
        if level_data.level.to_lowercase() == name.to_lowercase() {
            match level_data.otype {
                LegacyObjectType::Air => continue,
                LegacyObjectType::Wall => object_collection.push(LegacyObjectData {
                    position: world_position.to_tuple2d(),
                }),
                LegacyObjectType::Door => object_collection.push(LegacyObjectData {
                    position: world_position.to_tuple2d(),
                }),
            };
        };
    }
    let output_stream = serde_json::to_string_pretty(&object_collection).unwrap();
    target_file
        .write(output_stream.as_bytes())
        .expect("Error saving level data to file.");
}
