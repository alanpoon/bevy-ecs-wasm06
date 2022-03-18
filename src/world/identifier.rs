use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
// We use usize here because that is the largest `Atomic` we want to require
/// A unique identifier for a [`super::World`].
// Note that this *is* used by external crates as well as for internal safety checks
pub struct WorldId(usize);

/// The next [`WorldId`].
static MAX_WORLD_ID: AtomicUsize = AtomicUsize::new(0);

impl WorldId {
    /// Create a new, unique [`WorldId`]. Returns [`None`] if the supply of unique
    /// [`WorldId`]s has been exhausted
    ///
    /// Please note that the [`WorldId`]s created from this method are unique across
    /// time - if a given [`WorldId`] is [`Drop`]ped its value still cannot be reused
    pub fn new() -> Option<Self> {
        MAX_WORLD_ID
            // We use `Relaxed` here since this atomic only needs to be consistent with itself
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                val.checked_add(1)
            })
            .map(WorldId)
            .ok()
    }
}