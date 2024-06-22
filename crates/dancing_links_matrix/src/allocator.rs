//! Allocator for vector backed allocation of types.
//!
//! This module provides a trait `Allocator` that defines an interface for
//! allocating values of type `T` and returning a key of type `K`.
//!
//! The `VecAllocator` type implements this trait and uses a `Vec<T>` as the
//! underlying storage.

use std::ops::{Index, IndexMut};

/// A trait for allocating values of type `T` and returning a key of type `K`.
pub trait Allocator<T, K> {
    /// Returns the next key that can be used to allocate a value of type `T`.
    fn next_key(&self) -> K;

    /// Inserts a value of type `T` into the allocator and returns the key
    /// associated with the value.
    fn insert(&mut self, val: T) -> K;

    /// Returns the number of values currently in the allocator.
    fn len(&self) -> usize;

    /// Returns an iterator over the values in the allocator.
    ///
    /// The iterator yields `&T`, where `&T` is a reference to the value.
    fn iter<'s>(&'s self) -> impl Iterator<Item = &'s T> + 's
    where
        T: 's;
}

/// A vector backed allocator for values of type `T`.
///
/// This type implements the `Allocator` trait and uses a `Vec<T>` as the
/// underlying storage.
pub struct VecAllocator<T> {
    buffer: Vec<T>,
}

impl<T> VecAllocator<T> {
    /// Creates a new `VecAllocator`.
    pub(crate) fn new() -> VecAllocator<T> {
        VecAllocator { buffer: Vec::new() }
    }

    /// Creates a new `VecAllocator` with the specified capacity.
    pub(crate) fn with_capacity(len: usize) -> VecAllocator<T> {
        VecAllocator {
            buffer: Vec::with_capacity(len),
        }
    }
}

impl<T> Allocator<T, usize> for VecAllocator<T> {
    fn next_key(&self) -> usize {
        self.buffer.len()
    }

    fn insert(&mut self, val: T) -> usize {
        self.buffer.push(val);
        self.buffer.len() - 1
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> + 'a
    where
        T: 'a,
    {
        self.buffer.iter()
    }
}

impl<T> Index<usize> for VecAllocator<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

impl<T> IndexMut<usize> for VecAllocator<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}
