//! Arena trait for allocating memory for data structures.

/// An arena is a place to allocate memory for data structures.
///
/// The primary purpose of an arena is to allocate memory for data structures that
/// have a lifetime that is the same as the arena itself. This is useful because
/// it allows the data structures to contain references to other data structures
/// within the same arena, without the need for lifetimes.
pub trait Arena {
    /// Allocate memory for a value and return a reference to it.
    /// The value will have the same lifetime as the arena itself.
    /// The value will be dropped when the arena is dropped.
    fn alloc<T>(&self, val: T) -> &T;
}
