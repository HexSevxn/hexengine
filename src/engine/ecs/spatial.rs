use std::collections::HashSet;

use glam::Vec2;

use crate::engine::ecs::Entity;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    min: Vec2,
    max: Vec2,
}

impl AABB {
    pub fn new(center: Vec2, half_extents: Vec2) -> AABB {
        AABB {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn new_raw(min: Vec2, max: Vec2) -> AABB {
        AABB { min, max }
    }

    pub const ZERO: AABB = AABB {
        min: Vec2::ZERO,
        max: Vec2::ZERO,
    };
}

const MAX_ENTRIES_PER_NODE: usize = 8;
const MAX_DEPTH: u32 = 6;

#[derive(Debug)]
pub struct Quadtree {
    bounds: AABB,
    depth: u32,
    entries: Vec<(Entity, AABB)>,
    children: Option<Box<[Quadtree; 4]>>,
}

impl Quadtree {
    //Creates a new quadtree with a depth of 0
    pub fn new(bounds: AABB) -> Quadtree {
        Quadtree {
            bounds,
            depth: 0,
            entries: Vec::new(),
            children: None,
        }
    }

    //Creates a new quadtree at the given depth
    fn new_at_depth(bounds: AABB, depth: u32) -> Quadtree {
        Quadtree {
            bounds,
            depth,
            entries: Vec::new(),
            children: None,
        }
    }

    //Resets the quadtree to a cleared state, without children or entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.children = None;
    }

    //Inserts data into a quadtree, given the data entry and bounds
    pub fn insert(&mut self, entity: Entity, aabb: AABB) {
        if !aabb.intersects(&self.bounds) {
            return;
        }

        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.insert(entity, aabb);
            }
            return;
        }

        self.entries.push((entity, aabb));

        if self.entries.len() > MAX_ENTRIES_PER_NODE && self.depth < MAX_DEPTH {
            self.subdivide();
        }
    }

    //Subdivies the quadtree into 4 child quadtrees
    fn subdivide(&mut self) {
        let center = (self.bounds.min + self.bounds.max) / 2.0;

        let mut children = Box::new([
            Quadtree::new_at_depth(
                AABB {
                    min: self.bounds.min,
                    max: center,
                },
                self.depth + 1,
            ),
            Quadtree::new_at_depth(
                AABB {
                    min: Vec2::new(center.x, self.bounds.min.x),
                    max: Vec2::new(self.bounds.max.x, center.y),
                },
                self.depth + 1,
            ),
            Quadtree::new_at_depth(
                AABB {
                    min: Vec2::new(self.bounds.min.x, center.y),
                    max: Vec2::new(center.x, self.bounds.max.y),
                },
                self.depth + 1,
            ),
            Quadtree::new_at_depth(
                AABB {
                    min: center,
                    max: self.bounds.max,
                },
                self.depth + 1,
            ),
        ]);

        for (entity, aabb) in self.entries.drain(..) {
            for child in children.iter_mut() {
                child.insert(entity, aabb);
            }
        }

        self.children = Some(children)
    }

    //Returns a hashset of entities within the given region. Uses a hashset to prevent duplicates
    pub fn query(&self, region: AABB, out: &mut HashSet<Entity>) {
        if !self.bounds.intersects(&region) {
            return;
        }

        for (entity, aabb) in &self.entries {
            if aabb.intersects(&region) {
                out.insert(*entity);
            }
        }

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query(region, out);
            }
        }
    }
}
