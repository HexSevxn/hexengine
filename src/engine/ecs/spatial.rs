use glam::Vec2;

use crate::engine::asset::Rectangle;

pub trait SpatialShape {
    fn aabb(&self) -> Rectangle;
    fn contains(&self, point: Vec2) -> bool;
}

/* 
TODO!!
impl SpatialShape for Vec2 {
    fn aabb(&self) -> Rectangle {
        
    }
    fn contains(&self, point: Vec2) -> bool {
        return *self == point;
    }
}

impl SpatialShape for Rectangle {
    fn aabb(&self) -> Rectangle {
        return *self; 
    }
    
    fn contains(&self, point: Vec2) -> bool {
        return point.x >= self.aa.x && point.y >= self.aa.y && point.x <= self.bb.x && point.y <= self.bb.y;
    }
}

impl SpatialShape for Circle {
    fn aabb(&self) -> Rectangle {
        
    }
    fn contains(&self, point: Vec2) -> bool {
        
    }
}

    */