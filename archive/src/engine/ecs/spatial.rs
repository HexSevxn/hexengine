use crate::engine::math::{Rec, Circle, Vec2};

pub trait SpatialShape {
    fn aabb(&self) -> Rect;
    fn contains(&self, point: Vec2) -> bool;
}

impl SpatialShape for Vec2 {
    fn aabb(&self) -> Rect {
        return Rect::new(*self, *self);
    }
    fn contains(&self, point: Vec2) -> bool {
        return *self == point;
    }
}

impl SpatialShape for Rec {
    fn aabb(&self) -> Rect {
        return *self; 
    }
    
    fn contains(&self, point: Vec2) -> bool {
        return point.x >= self.aa.x && point.y >= self.aa.y && point.x <= self.bb.x && point.y <= self.bb.y;
    }
}

impl SpatialShape for Circle {
    
}