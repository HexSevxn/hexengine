use std::time::{Duration, Instant};
use std::thread::sleep;
use crossterm::style;
use crossterm::event::{self, KeyCode, Event};

pub mod editor;
pub mod engine;
pub mod client;

use client::{Control, ui::Screen};
use engine::ecs::world::World;
use engine::math::Vec2;
use engine::game::{Renderable, Transformation, Collidable, level};

pub fn blocked_loop(screen: &mut Screen, world: &mut World) {
    screen.clear();

    let player = world.new_entity();
    world.add_component_to_entity(player, Control);
    world.add_component_to_entity(player, Renderable {
        character: 'P',
        fg_color: style::Color::Red,
        bg_color: style::Color::Black,
        visible: true,
    });
    world.add_component_to_entity(player, Transformation {
        position: Vec2::new(4, 8),
        ..Default::default()
    });

    level::load_legacy_data("spawn.txt", "spawn.json", world);

    screen.render_pass(world, player);
    loop {
        match event::read().unwrap() {
            Event::Key(key_event) => {
                input_step(world, key_event.code);
            }
            _ => (),
        }

        game_step(world);
        screen.render_pass(world, player);
    }
}

pub const FPS_TARGET: f64 = 15.0;
pub fn frame_loop(screen: &mut Screen, world: &mut World) {
    screen.clear();
    let frame_duration: f64 = 1.0 / FPS_TARGET;
    let mut last_frame = Instant::now();
    let mut accumulated_time: f64 = 0.0;

    let player = world.new_entity();
    world.add_component_to_entity(player, Control);
    world.add_component_to_entity(player, Renderable {
        character: 'P',
        fg_color: style::Color::Red,
        bg_color: style::Color::Black,
        visible: true,
    });
    world.add_component_to_entity(player, Transformation {
        position: Vec2::new(4, 8),
        ..Default::default()
    });

    level::load_legacy_data("spawn.txt", "spawn.json", world);
    
    loop {
        let elapsed = last_frame.elapsed().as_secs_f64();
        accumulated_time += elapsed;
        last_frame = Instant::now();
        
        while accumulated_time >= frame_duration {
            //get input
            if event::poll(Duration::from_secs_f64(frame_duration)).unwrap() {
                match event::read().unwrap() {
                    Event::Key(key_event) => {
                        input_step(world, key_event.code);
                    }
                    _ => (),
                }
            }
            //update game state
            game_step(world);
            accumulated_time -= frame_duration;
        }
        //render pass
        screen.render_pass(world, player);

        let sleep_time = frame_duration - elapsed;
        if sleep_time > 0.0 {
            sleep(Duration::from_secs_f64(sleep_time));
        }      
    }
}

fn input_step(world: &mut World, key: KeyCode) {
    let controllables = world.query_mut::<(&mut Control, &mut Transformation)>();
    let mut update: Vec2 = Vec2::zero();
    match key {
        KeyCode::Char('w') => update.y -= 1,
        KeyCode::Char('s') => update.y += 1,
        KeyCode::Char('a') => update.x -= 1,
        KeyCode::Char('d') => update.x += 1,
        _ => (),
    }
    for (_control, transformation) in controllables {
        transformation.velocity += update.clone();
    }
}

fn game_step(world: &mut World) {
    let velocity_query = world.query_mut::<&mut Transformation>();
    for transform in velocity_query {
        transform.position += transform.velocity.normal();
        transform.velocity -= transform.velocity.normal();
    }
    
    /*
    let collidable_query = world.query::<(&Transformation, &Collidable)>();
    let velocity_query = world.query_mut::<(&mut Transformation, &mut Collidable)>();
    for (transform, _collide) in velocity_query {
        
    }
    */
}

pub fn read_file(path: &str) -> Vec<String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file = File::open(path).expect(&format!("File {} cannot be found.", path));
    let reader = BufReader::new(file);
    reader
        .lines()
        .into_iter()
        .map(|x| x.expect(&format!("Error reading lines of file: {}", path)))
        .collect::<Vec<String>>()
}