use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use crate::game::{Collidable, GRID_CELL_SIZE, Geometry, GeometryType, Renderable, Transformation};
use crate::{
    engine::{
        ecs::{Component, world::World},
        math::get_rectangle_triangles,
        render::color::{Color, TriangleColorMap},
    },
    game::{Interactable, WorldPosition},
};
use glam::{Vec3, Vec4, vec2};
use serde::{Deserialize, Serialize};

const OBJECT_Z: i32 = 5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ObjectType {
    Wall,
    Door,
}

#[derive(Debug)]
pub struct LevelData {
    pub level: String,
    pub otype: ObjectType,
}
impl Component for LevelData {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ObjectData {
    position: (i32, i32),
    otype: ObjectType,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct LevelFile {
    objects: Vec<ObjectData>,
}

pub const MAP_DATA_PATH: &str = "mapdata/";

fn level_path(name: &str) -> String {
    format!("{MAP_DATA_PATH}{name}.json")
}

pub fn list_levels() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(MAP_DATA_PATH)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter_map(|path| path.file_stem()?.to_str().map(String::from))
        .collect();
    names.sort();
    names
}

fn spawn_object(world: &mut World, level: &str, object: ObjectData) {
    let world_position = WorldPosition::new(object.position.0, object.position.1, OBJECT_Z);
    let render_position = world_position.to_render();
    let corner_position = vec2(
        render_position.x - GRID_CELL_SIZE / 2.0,
        render_position.y - GRID_CELL_SIZE / 2.0,
    );

    //Every object gets
    let object_entity = world.new_entity();
    world.add_component_to_entity(
        object_entity,
        Transformation {
            position: render_position,
            velocity: Vec3::ZERO,
            rotation: 0.0_f32,
        },
    );
    world.add_component_to_entity(object_entity, world_position);

    //Get color based on ObjectType
    let color: Vec4 = match object.otype {
        ObjectType::Wall => Vec4::ONE,
        ObjectType::Door => Color::BROWN.as_vec4(),
    };
    let color_map = TriangleColorMap::flat(color);
    let (t1, t2) =
        get_rectangle_triangles(corner_position, GRID_CELL_SIZE, GRID_CELL_SIZE, color_map);

    //Add additional components based on ObjectType
    match object.otype {
        ObjectType::Door => {
            world.add_component_to_entity(object_entity, Interactable::DOOR);
        }
        _ => {}
    }

    //Add remaning common components
    world.add_component_to_entity(
        object_entity,
        LevelData {
            level: level.to_string(),
            otype: object.otype,
        },
    );
    world.add_component_to_entity(object_entity, world_position);
    world.add_component_to_entity(
        object_entity,
        Transformation {
            position: render_position,
            ..Default::default()
        },
    );
    world.add_component_to_entity(
        object_entity,
        Renderable {
            color: color_map,
            ..Default::default()
        },
    );
    world.add_component_to_entity(
        object_entity,
        Geometry {
            geometry_type: GeometryType::Cell(t1, t2),
        },
    );
    //All entities get collidable by default, can be changed to only include certain entities later
    world.add_component_to_entity(object_entity, Collidable::default());
}

pub fn load_level(name: &str, world: &mut World) -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string(level_path(name))?;
    let file: LevelFile = serde_json::from_str(&text)?;

    for object in &file.objects {
        spawn_object(world, name, *object);
    }

    if !file.objects.is_empty() {
        world.create_collision_grid();
    }

    Ok(())
}

pub fn save_level(name: &str, world: &World) -> Result<(), Box<dyn Error>> {
    let mut objects: Vec<ObjectData> = if world.get_component_set::<LevelData>().is_some() {
        world
            .query::<(&LevelData, &WorldPosition)>()
            .filter(|(data, _)| data.level.eq_ignore_ascii_case(name))
            .map(|(data, position)| ObjectData {
                position: position.to_tuple2d(),
                otype: data.otype,
            })
            .collect()
    } else {
        Vec::new()
    };
    objects.sort_by_key(|o| (o.position.1, o.position.0));

    let json = serde_json::to_string_pretty(&LevelFile { objects })?;
    fs::write(level_path(name), json)?;

    Ok(())
}

pub fn convert_legacy_levels() -> io::Result<()> {
    for entry in fs::read_dir(MAP_DATA_PATH)? {
        let path = entry?.path();
        if path.extension().is_none_or(|ext| ext != "txt") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if Path::new(&level_path(name)).exists() {
            continue;
        }

        let file = parse_legacy(&path)?;
        fs::write(level_path(name), serde_json::to_string_pretty(&file)?)?;
        println!(
            "Converted legacy level '{name}' ({} objects)",
            file.objects.len()
        );
    }
    Ok(())
}

fn parse_legacy(path: &Path) -> io::Result<LevelFile> {
    let text = fs::read_to_string(path)?;
    let mut objects = Vec::new();

    for line in text.lines().skip(1) {
        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() < 3 {
            continue;
        }
        let (Ok(x), Ok(y)) = (
            fields[0].trim().parse::<i32>(),
            fields[1].trim().parse::<i32>(),
        ) else {
            continue;
        };
        let otype = match fields[2].trim().to_lowercase().as_str() {
            "wall" => ObjectType::Wall,
            "door" => ObjectType::Door,
            _ => continue, // air is just an empty cell; other legacy types aren't supported yet
        };
        objects.push(ObjectData {
            position: (x, y),
            otype,
        });
    }
    Ok(LevelFile { objects })
}
