//! Allocator for vector backed allocation of types.
//!
//! This module provides a trait `Allocator` that defines an interface for
//! allocating values of type `T` and returning a key of type `K`.
//!
//! The `VecAllocator` type implements this trait and uses a `Vec<T>` as the
//! underlying storage.

use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

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

    fn finalize(&mut self) {}
}

/// A vector backed allocator for values of type `T`.
///
/// This type implements the `Allocator` trait and uses a `Vec<T>` as the
/// underlying storage.
pub struct VecAllocator<T, K> {
    buffer: Vec<T>,
    _key: PhantomData<K>,
}

impl<T, K> VecAllocator<T, K> {
    /// Creates a new `VecAllocator`.
    pub(crate) fn new() -> VecAllocator<T, K> {
        VecAllocator {
            buffer: Vec::new(),
            _key: PhantomData,
        }
    }

    /// Creates a new `VecAllocator` with the specified capacity.
    pub(crate) fn with_capacity(len: usize) -> VecAllocator<T, K> {
        VecAllocator {
            buffer: Vec::with_capacity(len),
            _key: PhantomData,
        }
    }
}

impl<T, K: Into<usize> + From<usize>> Allocator<T, K> for VecAllocator<T, K> {
    fn next_key(&self) -> K {
        self.buffer.len().into()
    }

    fn insert(&mut self, val: T) -> K {
        self.buffer.push(val);
        (self.buffer.len() - 1).into()
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

    fn finalize(&mut self) {
        self.buffer.shrink_to_fit()
    }
}

impl<T, K: Into<usize>> Index<K> for VecAllocator<T, K> {
    type Output = T;

    fn index(&self, index: K) -> &Self::Output {
        unsafe { self.buffer.get_unchecked(index.into()) }
    }
}

impl<T, K: Into<usize>> IndexMut<K> for VecAllocator<T, K> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        unsafe { self.buffer.get_unchecked_mut(index.into()) }
    }
}
