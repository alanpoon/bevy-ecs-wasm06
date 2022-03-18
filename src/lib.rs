#![doc = include_str!("../README.md")]

pub mod archetype;
pub mod bundle;
pub mod change_detection;
pub mod component;
pub mod entity;
pub mod event;
pub mod query;
pub mod schedule;
pub mod storage;
pub mod system;
pub mod world;

/// Most commonly used re-exported types.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        bundle::Bundle,
        change_detection::DetectChanges,
        component::Component,
        entity::Entity,
        event::{EventReader, EventWriter},
        query::{Added, ChangeTrackers, Changed, Or, QueryState, With, Without},
        schedule::{
            AmbiguitySetLabel, ExclusiveSystemDescriptorCoercion, ParallelSystemDescriptorCoercion,
            RunCriteria, RunCriteriaDescriptorCoercion, RunCriteriaLabel, RunCriteriaPiping,
            Schedule, Stage, StageLabel, State, SystemLabel, SystemSet, SystemStage,
        },
        system::{
            Commands, ConfigurableSystem, In, IntoChainSystem, IntoExclusiveSystem, IntoSystem,
            Local, NonSend, NonSendMut, Query, QuerySet, RemovedComponents, Res, ResMut, System,
        },
        world::{FromWorld, Mut, World},
    };
}

pub use bevy_ecs_macros::all_tuples;

#[cfg(test)]
mod tests {
    use crate as bevy_ecs;
    use crate::{
        bundle::Bundle,
        component::{Component, ComponentId},
        entity::Entity,
        query::{
            Added, ChangeTrackers, Changed, FilterFetch, FilteredAccess, With, Without, WorldQuery,
        },
        world::{Mut, World},
    };
    use std::any::TypeId;
    #[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
    struct A(usize);
    #[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
    struct B(usize);
    #[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
    struct C;

    #[derive(Component, Clone, Debug)]
    struct DropCk(Arc<AtomicUsize>);
    impl DropCk {
        fn new_pair() -> (Self, Arc<AtomicUsize>) {
            let atomic = Arc::new(AtomicUsize::new(0));
            (DropCk(atomic.clone()), atomic)
        }
    }

    impl Drop for DropCk {
        fn drop(&mut self) {
            self.0.as_ref().fetch_add(1, Ordering::Relaxed);
        }
    }

    #[derive(Component, Clone, Debug)]
    #[component(storage = "SparseSet")]
    struct DropCkSparse(DropCk);

    #[derive(Component, Copy, Clone, PartialEq, Eq, Debug)]
    #[component(storage = "Table")]
    struct TableStored(&'static str);
    #[derive(Component, Copy, Clone, PartialEq, Eq, Debug)]
    #[component(storage = "SparseSet")]
    struct SparseStored(u32);
}
