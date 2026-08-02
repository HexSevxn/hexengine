#![allow(non_snake_case, unused_parens)]
use crate::engine::ecs::{Component, SparseSet, world::World};
use std::marker::PhantomData;

pub trait Query<'a> {
    type Iter: Iterator + 'a;
    fn fetch(world: &'a World) -> Self::Iter;
}

pub trait QueryMut<'a> {
    type Iter: Iterator + 'a;
    fn fetch(world: &'a mut World) -> Self::Iter;
}

//Creates a Query trait implementation for queries with the given number of input types
macro_rules! impl_query {
    ($name:ident, $($set_type:ident),+) => {
        #[derive(Debug)]
        pub struct $name<'a, $($set_type),+> {
            $(
                $set_type: &'a SparseSet<$set_type>,
            )+
            index: usize,
            driver: usize,
        }

        impl<'a, $($set_type: Component + 'static),+>
            $name<'a, $($set_type),+>
        {
            fn new(
                $($set_type: &'a SparseSet<$set_type>),+,
                driver: usize,
            ) -> Self {
                Self {
                    $($set_type),+,
                    index: 0,
                    driver,
                }
            }
        }

        impl<'a, $($set_type: Component + 'static),+>
            Iterator for $name<'a, $($set_type),+>
        {
            type Item = ($(&'a $set_type),+);

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let mut current_idx = 0;
                    let mut entity = None;

                    $(
                        if self.driver == current_idx {
                            if self.index < self.$set_type.entities.len() {
                                entity = Some(self.$set_type.entities[self.index]);
                            }
                        }
                        current_idx += 1;
                    )+
                    let _ = current_idx;

                    let entity = match entity {
                        Some(e) => e,
                        None => return None,
                    };

                    self.index += 1;

                    //O(1) time btw
                    let contains_all = true $(&& self.$set_type.contains(entity))+;
                    if contains_all == true {
                        return Some(
                            ($(self.$set_type.get(entity).unwrap()),+)
                        );
                    }
                }
            }
        }

        impl<'a, $($set_type: Component + 'static),+>
            Query<'a> for ($(&'a $set_type,)+)
        {
            type Iter = $name<'a, $($set_type),+>;

            //Finds the smallest set before creating iterator
            fn fetch(world: &'a World) -> Self::Iter {
                $(let $set_type = world.get_component_set::<$set_type>().unwrap();)+

                let mut driver = 0;
                let mut min_len = usize::MAX;
                let mut current_idx = 0;
                $(
                    if $set_type.entities.len() < min_len {
                        min_len = $set_type.entities.len();
                        driver = current_idx;
                    }
                    current_idx += 1;
                )+
                let (_, _) = (min_len, current_idx);

                $name::new($($set_type),+, driver)
            }
        }
    };
}

impl_query!(QueryIter1, A);
impl_query!(QueryIter2, A, B);
impl_query!(QueryIter3, A, B, C);
impl_query!(QueryIter4, A, B, C, D);

macro_rules! impl_query_mut {
    ($name:ident, $($set_type:ident),+) => {
        #[derive(Debug)]
        pub struct $name<'a, $($set_type),+> {
            // use raw pointers instead of references to bypass the HashMap double-borrow and the Iterator lifetime issues.
            $(
                $set_type: *mut SparseSet<$set_type>,
            )+
            index: usize,
            driver: usize,
            // PhantomData tells the compiler this struct conceptually holds mutable references tied to lifetime 'a, ensuring safety outside the macro.
            _marker: PhantomData<&'a mut ()>,
        }

        impl<'a, $($set_type: Component + 'static),+>
            Iterator for $name<'a, $($set_type),+>
        {
            type Item = ($(&'a mut $set_type),+);

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let mut current_idx = 0;
                    let mut entity = None;

                    $(
                        if self.driver == current_idx {
                            // SAFETY: have exclusive access to these sets for lifetime 'a
                            let driver_set = unsafe { &*self.$set_type };
                            if self.index < driver_set.entities.len() {
                                entity = Some(driver_set.entities[self.index]);
                            }
                        }
                        current_idx += 1;
                    )+

                    let _ = current_idx;

                    let entity = match entity {
                        Some(e) => e,
                        None => return None,
                    };

                    self.index += 1;

                    let mut contains_all = true;

                    $(
                        let set = unsafe { &*self.$set_type };
                        contains_all = contains_all && set.contains(entity);
                    )+

                    if contains_all {
                        return Some((
                            $(
                                // SAFETY: Types are distinct (enforced by how the query is used).
                                // only yield each entity once, preventing mutable aliasing.
                                unsafe { &mut *self.$set_type }.get_mut(entity).unwrap()
                            ),+
                        ));
                    }
                }
            }
        }

        impl<'a, $($set_type: Component + 'static),+>
            QueryMut<'a> for ($(&'a mut $set_type),+)
        {
            type Iter = $name<'a, $($set_type),+>;

            fn fetch(world: &'a mut World) -> Self::Iter {
                // SAFETY: temporarily cast the world to a raw pointer.
                // This stops the compiler from panicking when extracting multiple
                // mutable sets from the same HashMap in the macro loop below.
                let world_ptr = world as *mut World;

                $(
                    let $set_type = unsafe { &mut *world_ptr }
                        .get_component_set_mut::<$set_type>()
                        .expect("Tried to mutably query a component that is not registered")
                        as *mut SparseSet<$set_type>;
                )+

                let mut driver = 0;
                let mut min_len = usize::MAX;
                let mut current_idx = 0;

                $(
                    let set_len = unsafe { &*$set_type }.entities.len();
                    if set_len < min_len {
                        min_len = set_len;
                        driver = current_idx;
                    }
                    current_idx += 1;
                )+

                let _ = current_idx;
                let _ = min_len;

                $name {
                    $($set_type,)+
                    index: 0,
                    driver,
                    _marker: PhantomData,
                }
            }
        }
    };
}

impl_query_mut!(QueryIterMut1, A);
impl_query_mut!(QueryIterMut2, A, B);
impl_query_mut!(QueryIterMut3, A, B, C);
impl_query_mut!(QueryIterMut4, A, B, C, D);
