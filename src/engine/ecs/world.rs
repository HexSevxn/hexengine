use glam::vec2;

use crate::engine::ecs::{
    Component, Entity, SparseSet,
    query::{Query, QueryMut},
    spatial::{AABB, Quadtree},
};
use crate::game::{Collidable, Geometry, WorldPosition};
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

//Type for easy additions in future (generations?)

#[derive(Debug)]
pub struct World {
    entity_count: usize,
    component_storage: HashMap<TypeId, Box<dyn Any>>,
    pub collision_grid: HashMap<(i32, i32), Vec<Entity>>,
    spacial_tree: Quadtree,
    //pub spacial_tree: Quadtree<u16, Entity>,
}

impl World {
    pub fn new() -> World {
        World {
            entity_count: 0,
            component_storage: HashMap::new(),
            collision_grid: HashMap::new(),
            spacial_tree: Quadtree::new(AABB::new(vec2(0.0, 0.0), vec2(100.0, 100.0))),
        }
    }

    pub fn query<'a, Q>(&'a self) -> Q::Iter
    where
        Q: Query<'a>,
    {
        Q::fetch(self)
    }

    pub fn query_mut<'a, Q>(&'a mut self) -> Q::Iter
    where
        Q: QueryMut<'a>,
    {
        Q::fetch(self)
    }

    fn register_component<T: Component + 'static>(&mut self) {
        let sparse_set: SparseSet<T> = SparseSet::new();
        self.component_storage
            .insert(TypeId::of::<T>(), Box::new(sparse_set));
    }

    pub fn get_component_set<T: Component + 'static>(&self) -> Option<&SparseSet<T>> {
        self.component_storage
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref())
    }

    pub fn get_component_set_mut<T: Component + 'static>(&mut self) -> Option<&mut SparseSet<T>> {
        self.component_storage
            .get_mut(&TypeId::of::<T>())
            .and_then(|s| s.downcast_mut())
    }

    pub fn get_entity_component<T: Component + 'static>(
        &mut self,
        entity: Entity,
    ) -> Option<&mut T> {
        self.get_component_set_mut::<T>()
            .expect("Attempted to fetch unregistered component")
            .get_mut(entity)
    }

    pub fn new_entity(&mut self) -> Entity {
        let id = self.entity_count;
        self.entity_count += 1;
        Entity(id)
    }

    pub fn add_component_to_entity<T: Component + 'static>(
        &mut self,
        entity: Entity,
        component: T,
    ) {
        let type_id = TypeId::of::<T>();

        if !self.component_storage.contains_key(&type_id) {
            self.register_component::<T>();
        }

        let storage = self.get_component_set_mut::<T>().unwrap();
        storage.add(entity, component);
    }

    pub fn create_collision_grid(&mut self) {
        let entities = self
            .query::<(Entity, &Collidable, &WorldPosition)>()
            .map(|(entity, _, position)| (entity, position.to_tuple2d()))
            .collect::<Vec<(Entity, (i32, i32))>>();

        self.collision_grid.clear();

        for (entity, position) in entities {
            self.collision_grid
                .entry(position)
                .or_default()
                .push(entity);
        }
    }
    fn is_solid(&self, entity: Entity) -> bool {
        match self
            .get_component_set::<Collidable>()
            .and_then(|s| s.get(entity))
        {
            Some(collidable) => !collidable.overwrite,
            None => true,
        }
    }

    pub fn check_grid_collision(&self, position: (i32, i32)) -> bool {
        self.collision_grid
            .get(&position)
            .is_some_and(|entities| entities.iter().any(|&e| self.is_solid(e)))
    }

    pub fn check_collisions(&mut self, entity: Entity) -> bool {
        if !self.get_entity_component::<Collidable>(entity).is_some() {
            return false;
        }
        let geometry = self
            .get_entity_component::<Geometry>(entity)
            .expect("Checking collision on entity without geometry!")
            .get_bounds();

        let mut query_results: HashSet<Entity> = HashSet::new();
        self.spacial_tree.query(geometry, &mut query_results);
        if query_results.contains(&entity) {
            return true;
        }

        return false;
    }
}

impl Default for World {
    fn default() -> Self {
        World {
            entity_count: 0,
            component_storage: HashMap::new(),
            collision_grid: HashMap::new(),
            spacial_tree: Quadtree::new(AABB::new(vec2(0.0, 0.0), vec2(100.0, 100.0))),
        }
    }
}
