mod enum_variant_meta;
pub mod label;
pub use std::collections::hash_map::DefaultHasher as AHasher;
pub use enum_variant_meta::*;
pub use bevy_tracing as tracing;
use std::collections::hash_map::RandomState;
use std::{future::Future, pin::Pin};

#[cfg(not(target_arch = "wasm32"))]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(target_arch = "wasm32")]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;


/// A [`HashMap`][std::collections::HashMap] implementing [`aHash`], a high
/// speed keyed hashing algorithm intended for use in in-memory hashmaps.
///
/// `aHash` is designed for performance and is NOT cryptographically secure.
///
/// # Construction
///
/// Users may be surprised when a `HashMap` cannot be constructed with `HashMap::new()`:
///
/// ```compile_fail
/// # fn main() {
/// use bevy_utils::HashMap;
///
/// // Produces an error like "no function or associated item named `new` found [...]"
/// let map: HashMap<String, String> = HashMap::new();
/// # }
/// ```
///
/// The standard library's [`HashMap::new`][std::collections::HashMap::new] is
/// implemented only for `HashMap`s which use the
/// [`DefaultHasher`][std::collections::hash_map::DefaultHasher], so it's not
/// available for Bevy's `HashMap`.
///
/// However, an empty `HashMap` can easily be constructed using the `Default`
/// implementation:
///
/// ```
/// # fn main() {
/// use bevy_utils::HashMap;
///
/// // This works!
/// let map: HashMap<String, String> = HashMap::default();
/// assert!(map.is_empty());
/// # }
/// ```
///
/// [`aHash`]: https://github.com/tkaitchuck/aHash
pub type HashMap<K, V> = std::collections::HashMap<K, V, RandomState>;

pub trait AHashExt {
    fn with_capacity(capacity: usize) -> Self;
}

impl<K, V> AHashExt for HashMap<K, V> {
    /// Creates an empty `HashMap` with the specified capacity with aHash.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_utils::{HashMap, AHashExt};
    /// let mut map: HashMap<&str, i32> = HashMap::with_capacity(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        HashMap::with_capacity_and_hasher(capacity, RandomState::default())
    }
}


/// A [`HashSet`][std::collections::HashSet] implementing [`aHash`], a high
/// speed keyed hashing algorithm intended for use in in-memory hashmaps.
///
/// `aHash` is designed for performance and is NOT cryptographically secure.
///
/// # Construction
///
/// Users may be surprised when a `HashSet` cannot be constructed with `HashSet::new()`:
///
/// ```compile_fail
/// # fn main() {
/// use bevy_utils::HashSet;
///
/// // Produces an error like "no function or associated item named `new` found [...]"
/// let map: HashSet<String> = HashSet::new();
/// # }
/// ```
///
/// The standard library's [`HashSet::new`][std::collections::HashSet::new] is
/// implemented only for `HashSet`s which use the
/// [`DefaultHasher`][std::collections::hash_map::DefaultHasher], so it's not
/// available for Bevy's `HashSet`.
///
/// However, an empty `HashSet` can easily be constructed using the `Default`
/// implementation:
///
/// ```
/// # fn main() {
/// use bevy_utils::HashSet;
///
/// // This works!
/// let map: HashSet<String> = HashSet::default();
/// assert!(map.is_empty());
/// # }
/// ```
///
/// [`aHash`]: https://github.com/tkaitchuck/aHash
pub type HashSet<K> = std::collections::HashSet<K, RandomState>;

impl<K> AHashExt for HashSet<K> {
    /// Creates an empty `HashSet` with the specified capacity with aHash.
    ///
    /// The hash set will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash set will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_utils::{HashSet, AHashExt};
    /// let set: HashSet<i32> = HashSet::with_capacity(10);
    /// assert!(set.capacity() >= 10);
    /// ```
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        HashSet::with_capacity_and_hasher(capacity, RandomState::default())
    }
}

