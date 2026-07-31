use crate::engine::render::{Triangle, Vertex};
use glam::{Vec2, Vec4, vec2};
use std::f32::consts::PI;
use uuid::Uuid;

pub trait Asset {
    /// Unpack turns an object into a vector of triangles for addition to the object store.
    fn unpack(&self) -> Vec<Triangle>;
}

///High level structure for the AssetManager/
/// Stores the UUID and underlying data for our object in an easy to manage data structure for manipulation by the AssetManager.
pub struct Object {
    pub uuid: Uuid,
    pub data: Box<dyn Asset>,
}

impl Object {
    pub fn new(item: impl Asset + 'static) -> Object {
        return Object {
            uuid: Uuid::new_v4(),
            data: Box::new(item),
        };
    }
}

pub struct AssetManager {
    pub asset_count: usize,
    pub pool: Vec<Object>,
}

impl AssetManager {
    pub fn empty() -> AssetManager {
        AssetManager {
            pool: Vec::new(),
            asset_count: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Rectangle {
    pub position: Vec2,
    pub width: f32,
    pub height: f32,
    pub color: Vec4,
}

impl Asset for Rectangle {
    fn unpack(&self) -> Vec<Triangle> {
        let v1 = Vertex::from_vec2_c(self.position, self.color);
        let v2 = Vertex::from_vec2_c(
            vec2(self.position.x + self.width, self.position.y),
            self.color,
        );
        let v3 = Vertex::from_vec2_c(
            vec2(self.position.x, self.position.y + self.height),
            self.color,
        );
        let v4 = Vertex::from_vec2_c(
            vec2(self.position.x + self.width, self.position.y + self.height),
            self.color,
        );

        let t1 = Triangle::new(v1, v3, v2);
        let t2 = Triangle::new(v2, v3, v4);

        return vec![t1, t2];
    }
}

#[derive(Copy, Clone)]
pub struct Line {
    pub start: Vec2,
    pub end: Vec2,
    pub color: Vec4,
    pub size: Vec2,
}

impl Asset for Line {
    fn unpack(&self) -> Vec<Triangle> {
        let (dy, dx) = (self.end.y - self.start.y, self.end.x - self.start.x);
        let length: f32 = (dx * dx + dy * dy).sqrt();
        let normalized: Vec2 = vec2(-dy / length, dx / length);

        let v1 = Vertex::from_vec2_c(self.start + (normalized * self.size), self.color);
        let v2 = Vertex::from_vec2_c(self.start - (normalized * self.size), self.color);

        let v3 = Vertex::from_vec2_c(self.end + (normalized * self.size), self.color);
        let v4 = Vertex::from_vec2_c(self.end - (normalized * self.size), self.color);

        let t1 = Triangle::new(v1, v3, v2);
        let t2 = Triangle::new(v2, v3, v4);
        return vec![t1, t2];
    }
}

#[derive(Copy, Clone)]
pub struct Circle {
    pub position: Vec2,
    pub radius: f32,
    pub color: Vec4,
}

impl Asset for Circle {
    fn unpack(&self) -> Vec<Triangle> {
        let mut tris: Vec<Triangle> = Vec::new();

        //DICTATES THE NUMBER OF TRIANGLES USED TO BUILD A CIRCLE
        const TRI_COUNT: usize = 30;
        const COUNT_F32: f32 = TRI_COUNT as f32;

        const SCALAR: f32 = 2_f32 * PI / COUNT_F32;

        for index in 0..TRI_COUNT {
            tris.push(Triangle::new(
                Vertex::from_vec2_c(
                    vec2(
                        self.position.x + (self.radius * (index as f32 * SCALAR).cos()),
                        self.position.y + (self.radius * (index as f32 * SCALAR).sin()),
                    ),
                    self.color,
                ),
                Vertex::from_vec2_c(
                    vec2(
                        self.position.x + (self.radius * ((index + 1) as f32 * SCALAR).cos()),
                        self.position.y + (self.radius * ((index + 1) as f32 * SCALAR).sin()),
                    ),
                    self.color,
                ),
                Vertex::from_vec2_c(self.position, self.color),
            ))
        }

        return tris;
    }
}
