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