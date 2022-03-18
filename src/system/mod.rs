//! Tools for controlling behavior in an ECS application.
//!
//! Systems define how an ECS based application behaves. They have to be registered to a
//! [`SystemStage`](crate::schedule::SystemStage) to be able to run. A system is usually
//! written as a normal function that will be automatically converted into a system.
//!
//! System functions can have parameters, through which one can query and mutate Bevy ECS state.
//! Only types that implement [`SystemParam`] can be used, automatically fetching data from
//! the [`World`](crate::world::World).
//!
//! System functions often look like this:
//!
//! ```
//! # use bevy_ecs::prelude::*;
//! #
//! # #[derive(Component)]
//! # struct Player { alive: bool }
//! # #[derive(Component)]
//! # struct Score(u32);
//! # struct Round(u32);
//! #
//! fn update_score_system(
//!     mut query: Query<(&Player, &mut Score)>,
//!     mut round: ResMut<Round>,
//! ) {
//!     for (player, mut score) in query.iter_mut() {
//!         if player.alive {
//!             score.0 += round.0;
//!         }
//!     }
//!     round.0 += 1;
//! }
//! # update_score_system.system();
//! ```
//!
//! # System ordering
//!
//! While the execution of systems is usually parallel and not deterministic, there are two
//! ways to determine a certain degree of execution order:
//!
//! - **System Stages:** They determine hard execution synchronization boundaries inside of
//!   which systems run in parallel by default.
//! - **Labeling:** First, systems are labeled upon creation by calling `.label()`. Then,
//!   methods such as `.before()` and `.after()` are appended to systems to determine
//!   execution order in respect to other systems.
//!
//! # System parameter list
//! Following is the complete list of accepted types as system parameters:
//!
//! - [`Query`]
//! - [`Res`] and `Option<Res>`
//! - [`ResMut`] and `Option<ResMut>`
//! - [`Commands`]
//! - [`Local`]
//! - [`EventReader`](crate::event::EventReader)
//! - [`EventWriter`](crate::event::EventWriter)
//! - [`NonSend`] and `Option<NonSend>`
//! - [`NonSendMut`] and `Option<NonSendMut>`
//! - [`RemovedComponents`]
//! - [`SystemChangeTick`]
//! - [`Archetypes`](crate::archetype::Archetypes) (Provides Archetype metadata)
//! - [`Bundles`](crate::bundle::Bundles) (Provides Bundles metadata)
//! - [`Components`](crate::component::Components) (Provides Components metadata)
//! - [`Entities`](crate::entity::Entities) (Provides Entities metadata)
//! - All tuples between 1 to 16 elements where each element implements [`SystemParam`]
//! - [`()` (unit primitive type)](https://doc.rust-lang.org/stable/std/primitive.unit.html)

mod commands;
mod exclusive_system;
mod function_system;
mod query;
#[allow(clippy::module_inception)]
mod system;
mod system_chaining;
mod system_param;

pub use commands::*;
pub use exclusive_system::*;
pub use function_system::*;
pub use query::*;
pub use system::*;
pub use system_chaining::*;
pub use system_param::*;