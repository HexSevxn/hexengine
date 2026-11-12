#![allow(non_snake_case, unused_parens)]
use crate::engine::ecs::{Component, Entity, SparseSet, world::World};
use std::marker::PhantomData;

pub trait Query<'a> {
    type Iter: Iterator + 'a;
    fn fetch(world: &'a World) -> Self::Iter;
}

pub trait QueryMut<'a> {
    type Iter: Iterator + 'a;
    fn fetch(world: &'a mut World) -> Self::Iter;
}

pub trait QueryParam<'a> {
    type Item;

    fn entities(world: &'a World) -> Option<&'a [Entity]>;
    unsafe fn get(world: *const World, entity: Entity) -> Option<Self::Item>;
}

impl<'a, T: Component + 'static> QueryParam<'a> for &'a T {
    type Item = &'a T;

    fn entities(world: &'a World) -> Option<&'a [Entity]> {
        Some(&world.get_component_set::<T>()?.entities)
    }

    unsafe fn get(world: *const World, entity: Entity) -> Option<Self::Item> {
        // SAFETY: the query owns an immutable borrow of the World for 'a.
        let set = unsafe { &*world }.get_component_set::<T>()?;
        let component = set.get(entity)? as *const T;
        // SAFETY: `component` points into the World borrowed for 'a.
        Some(unsafe { &*component })
    }
}

impl<'a> QueryParam<'a> for Entity {
    type Item = Entity;

    fn entities(_world: &'a World) -> Option<&'a [Entity]> {
        None
    }

    unsafe fn get(_world: *const World, entity: Entity) -> Option<Self::Item> {
        Some(entity)
    }
}

pub trait QueryMutParam<'a> {
    type Item;

    fn entities(world: &'a World) -> Option<&'a [Entity]>;
    unsafe fn get(world: *mut World, entity: Entity) -> Option<Self::Item>;
}

impl<'a, T: Component + 'static> QueryMutParam<'a> for &'a mut T {
    type Item = &'a mut T;

    fn entities(world: &'a World) -> Option<&'a [Entity]> {
        Some(&world.get_component_set::<T>()?.entities)
    }

    unsafe fn get(world: *mut World, entity: Entity) -> Option<Self::Item> {
        // SAFETY: `QueryMut::fetch` has exclusive access to the World for 'a.
        // The query contract requires that the same mutable component type is
        // not requested more than once in a tuple.
        let set = unsafe { &mut *world }.get_component_set_mut::<T>()?;
        let component = set.get_mut(entity)? as *mut T;
        // SAFETY: the component belongs to the exclusively borrowed World.
        Some(unsafe { &mut *component })
    }
}

impl<'a> QueryMutParam<'a> for Entity {
    type Item = Entity;

    fn entities(_world: &'a World) -> Option<&'a [Entity]> {
        None
    }

    unsafe fn get(_world: *mut World, entity: Entity) -> Option<Self::Item> {
        Some(entity)
    }
}

//Creates a Query trait implementation for queries with the given number of input types
macro_rules! impl_query {
    ($name:ident, $($param:ident),+) => {
        #[derive(Debug)]
        pub struct $name<'a, $($param),+> {
            world: *const World,
            entities: &'a [Entity],
            index: usize,
            _marker: PhantomData<($($param,)+)>,
        }

        impl<'a, $($param: QueryParam<'a>),+> Iterator for $name<'a, $($param),+> {
            type Item = ($(<$param as QueryParam<'a>>::Item,)+);

            fn next(&mut self) -> Option<Self::Item> {
                while self.index < self.entities.len() {
                    let entity = self.entities[self.index];
                    self.index += 1;

                    // All QueryParam::get calls use the same entity. A missing
                    // required component causes this entity to be skipped.
                    let result = unsafe {
                        (
                            $(<$param as QueryParam<'a>>::get(self.world, entity),)+
                        )
                    };

                    if let ($(Some($param),)+) = result {
                        return Some(($($param,)+));
                    }
                }

                None
            }
        }

        impl<'a, $($param: QueryParam<'a> + 'a),+> Query<'a> for ($($param,)+) {
            type Iter = $name<'a, $($param),+>;

            fn fetch(world: &'a World) -> Self::Iter {
                let mut driver: Option<&'a [Entity]> = None;

                $(
                    if let Some(entities) = <$param as QueryParam<'a>>::entities(world) {
                        if driver.map_or(true, |current| entities.len() < current.len()) {
                            driver = Some(entities);
                        }
                    }
                )+

                let entities = driver.expect(
                    "Query must contain at least one component-backed parameter"
                );

                Self::Iter {
                    world: world as *const World,
                    entities,
                    index: 0,
                    _marker: PhantomData,
                }
            }
        }
    };
}

impl_query!(QueryIter1, A);
impl_query!(QueryIter2, A, B);
impl_query!(QueryIter3, A, B, C);
impl_query!(QueryIter4, A, B, C, D);

macro_rules! impl_query_mut {
    ($name:ident, $($param:ident),+) => {
        #[derive(Debug)]
        pub struct $name<'a, $($param),+> {
            world: *mut World,
            entities: &'a [Entity],
            index: usize,
            _marker: PhantomData<($($param,)+)>,
            _world: PhantomData<&'a mut World>,
        }

        impl<'a, $($param: QueryMutParam<'a>),+> Iterator
            for $name<'a, $($param),+>
        {
            type Item = ($(<$param as QueryMutParam<'a>>::Item,)+);

            fn next(&mut self) -> Option<Self::Item> {
                while self.index < self.entities.len() {
                    let entity = self.entities[self.index];
                    self.index += 1;

                    // SAFETY: `QueryMut::fetch` has exclusive access to the
                    // World for 'a. Each requested mutable component type is
                    // expected to occur at most once in the query tuple.
                    let result = unsafe {
                        (
                            $(<$param as QueryMutParam<'a>>::get(self.world, entity),)+
                        )
                    };

                    if let ($(Some($param),)+) = result {
                        return Some(($($param,)+));
                    }
                }

                None
            }
        }

        impl<'a, $($param: QueryMutParam<'a> + 'a),+> QueryMut<'a>
            for ($($param,)+)
        {
            type Iter = $name<'a, $($param),+>;

            fn fetch(world: &'a mut World) -> Self::Iter {
                // We keep the World exclusively borrowed for the entire
                // iterator lifetime. Raw access is only used to work around
                // the HashMap's dynamic component storage.
                let world_ptr = world as *mut World;
                let world_shared = world as *const World;

                let mut driver: Option<&'a [Entity]> = None;

                $(
                    if let Some(entities) = unsafe {
                        <$param as QueryMutParam<'a>>::entities(&*world_shared)
                    } {
                        if driver.map_or(true, |current| entities.len() < current.len()) {
                            driver = Some(entities);
                        }
                    }
                )+

                let entities = driver.expect(
                    "Mutable query must contain at least one component-backed parameter"
                );

                Self::Iter {
                    world: world_ptr,
                    entities,
                    index: 0,
                    _marker: PhantomData,
                    _world: PhantomData,
                }
            }
        }
    };
}

impl_query_mut!(QueryIterMut1, A);
impl_query_mut!(QueryIterMut2, A, B);
impl_query_mut!(QueryIterMut3, A, B, C);
impl_query_mut!(QueryIterMut4, A, B, C, D);
