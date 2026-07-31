pub mod editor;
pub mod engine;

use engine::render::app::{App, scale_to_screen};
use glam::{vec2, vec4};

pub fn setup_graphics(app: &mut App) {
    let display_size = app.display_size.clone();
    let grid_size: f32 = 64.0;

    let x_offset = display_size.x / grid_size;
    let y_offset = display_size.y / grid_size;

    let grid_color = vec4(1.0, 1.0, 1.0, 1.0);
    let space_color = vec4(1.0, 0.0, 0.0, 1.0);

    for y in 0..grid_size as usize {
        let start = scale_to_screen(display_size, vec2(0.0, y as f32 * y_offset));
        let end = scale_to_screen(display_size, vec2(display_size.x, y as f32 * y_offset));
        app.draw_line(start, end, 2.0, grid_color);
    }
    for x in 0..grid_size as usize {
        let start = vec2(x as f32 * x_offset, 0.0);
        let start = scale_to_screen(display_size, start);
        let end = scale_to_screen(display_size, vec2(x as f32 * x_offset, display_size.y));
        app.draw_line(start, end, 2.0, grid_color);
    }

    for y in 0..grid_size as usize {
        for x in 0..grid_size as usize {
            if x % 2 == 0 {
                let position = vec2(x_offset * x as f32, y_offset * y as f32);
                app.draw_rectangle(position, x_offset, y_offset, space_color);
            }
        }
    }

    app.draw_circle(vec2(0.0, 0.0), 0.5, vec4(1.0, 1.0, 1.0, 1.0));

    app.update_pipeline();
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