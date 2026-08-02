pub mod query;
pub mod spatial;
pub mod world;

pub type Entity = usize;
pub trait Component: std::fmt::Debug {}

#[derive(Debug)]
pub struct SparseSet<T> {
    dense: Vec<T>,         //Data storage. Compact storage of component data
    entities: Vec<Entity>, //Parallel with dense, each index contains the EID of the entity that has the parallel component data
    sparse: Vec<usize>,    //Non-parallel, index with EID to find index in dense
}

impl<T> SparseSet<T> {
    pub fn new() -> SparseSet<T> {
        SparseSet {
            dense: Vec::new(),
            entities: Vec::new(),
            sparse: Vec::new(),
        }
    }

    //Add new data mapped to the given entity (EID)
    pub fn add(&mut self, entity: Entity, component: T) {
        let index = self.dense.len();

        if entity >= self.sparse.len() {
            self.sparse.resize(entity + 1, usize::MAX);
        }

        self.dense.push(component);
        self.entities.push(entity);
        self.sparse[entity] = index;
    }

    //Check if the sparse set contains component data for the given entity (EID)
    pub fn contains(&self, entity: Entity) -> bool {
        if let Some(&index) = self.sparse.get(entity) {
            index != usize::MAX && self.entities[index] == entity
        } else {
            false
        }
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        if self.contains(entity) {
            let index = self.sparse[entity];
            Some(&self.dense[index])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if self.contains(entity) {
            let index = self.sparse[entity];
            Some(&mut self.dense[index])
        } else {
            None
        }
    }

    //swap-removes entity data from set, returning removed data
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        if !self.contains(entity) {
            return None;
        }

        let entity_index = self.sparse[entity];
        let last_index = self.dense.len() - 1;

        self.dense.swap(entity_index, last_index);
        self.entities.swap(entity_index, last_index);

        let moved_entity = self.entities[entity_index];
        self.sparse[moved_entity] = entity_index;

        self.sparse[entity] = usize::MAX;

        self.entities.pop();
        self.dense.pop()
    }
    pub fn iter(&self) -> SparseSetIter<T> {
        SparseSetIter {
            storage: self,
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> SparseSetIterMut<T> {
        SparseSetIterMut {
            inner: self.dense.iter_mut(),
        }
    }
}

pub struct SparseSetIter<'a, T> {
    storage: &'a SparseSet<T>,
    index: usize,
}

impl<'a, T> Iterator for SparseSetIter<'a, T> {
    type Item = (&'a Entity, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.storage.dense.len() {
            return None;
        }

        let item = (
            &self.storage.entities[self.index],
            &self.storage.dense[self.index],
        );
        self.index += 1;
        return Some(item);
    }
}

pub struct SparseSetIterMut<'a, T> {
    inner: std::slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for SparseSetIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
