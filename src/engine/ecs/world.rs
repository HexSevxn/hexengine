use crate::engine::ecs::{
    Component, Entity, SparseSet,
    query::{Query, QueryMut},
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

//Type for easy additions in future (generations?)

#[derive(Debug)]
pub struct World {
    entity_count: usize,
    component_storage: HashMap<TypeId, Box<dyn Any>>,
    //pub spacial_tree: Quadtree<u16, Entity>,
}

impl World {
    pub fn new() -> World {
        World {
            entity_count: 0,
            component_storage: HashMap::new(),
            //spacial_tree: Quadtree::<u16, Entity>::new(8),
        }
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
        id
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
}

impl Default for World {
    fn default() -> Self {
        World {
            entity_count: 0,
            component_storage: HashMap::new(),
        }
    }
}
