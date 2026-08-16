use glam::{Vec2, Vec4, vec2};
use crate::engine::render::{Triangle, color::TriangleColorMap};

pub fn get_rectangle_triangles(position: Vec2, width: f32, height: f32, color: Vec4) -> (Triangle, Triangle) {
    let color_map = TriangleColorMap::flat(color);
    
    let midpoint = vec2(position.x + (width / 2.0), position.y + (height / 2.0));
    let v0 = position - midpoint;
    let v1 = vec2(position.x + width, position.y) - midpoint;
    let v2 = vec2(position.x, position.y + height) - midpoint;
    let v3 = vec2(
         position.x + width,
        position.y + height,
    ) - midpoint;

    let t1 = Triangle::new(v0, v2, v1, color_map);
    let t2 = Triangle::new(v1, v2, v3, color_map);

    return (t1, t2);
}