//! Allocator for vector backed allocation of types.
//!
//! This module provides a trait `Allocator` that defines an interface for
//! allocating values of type `T` and returning a key of type `K`.
//!
//! The `VecAllocator` type implements this trait and uses a `Vec<T>` as the
//! underlying storage.

use std::marker::PhantomData;

pub trait IndexOps<T, K> {
    /// Returns an iterator over the values in the allocator.
    ///
    /// The iterator yields `&T`, where `&T` is a reference to the value.
    fn iter<'s>(&'s self) -> impl Iterator<Item = &'s T> + 's
    where
        T: 's;

    /// Returns the number of values currently in the allocator.
    fn len(&self) -> usize;
}

pub trait IndexBuilder<T, K>: IndexOps<T, K> {
    type Index: Index<T, K>;

    /// Returns the next key that can be used to allocate a value of type `T`.
    fn next_key(&self) -> K;

    /// Inserts a value of type `T` into the allocator and returns the key
    /// associated with the value.
    fn insert(&mut self, val: T) -> K;

    fn finalize(self) -> Self::Index;

    fn get_mut(&mut self, key: K) -> &mut T;
}

/// A trait for allocating values of type `T` and returning a key of type `K`.
pub trait Index<T, K>: IndexOps<T, K> {
    fn get(&self, key: K) -> &T;

    fn get_mut(&mut self, key: K) -> &mut T;
}

pub struct VecIndexBuilder<T, K> {
    buffer: Vec<T>,
    _key: PhantomData<K>,
}

impl<T, K> VecIndexBuilder<T, K> {
    /// Creates a new `VecIndexBuilder`.
    pub(crate) fn new() -> VecIndexBuilder<T, K> {
        VecIndexBuilder {
            buffer: Vec::new(),
            _key: PhantomData,
        }
    }

    /// Creates a new `VecIndexBuilder` with the specified capacity.
    pub(crate) fn with_capacity(len: usize) -> VecIndexBuilder<T, K> {
        VecIndexBuilder {
            buffer: Vec::with_capacity(len),
            _key: PhantomData,
        }
    }
}

impl<T, K: Into<usize> + From<usize>> IndexBuilder<T, K> for VecIndexBuilder<T, K> {
    type Index = VecIndex<T, K>;

    fn next_key(&self) -> K {
        self.buffer.len().into()
    }

    fn insert(&mut self, val: T) -> K {
        self.buffer.push(val);
        (self.buffer.len() - 1).into()
    }

    fn finalize(mut self) -> VecIndex<T, K> {
        self.buffer.shrink_to_fit();

        VecIndex {
            buffer: self.buffer.into_boxed_slice(),
            _key: PhantomData,
        }
    }

    fn get_mut(&mut self, key: K) -> &mut T {
        self.buffer.get_mut(key.into()).unwrap()
    }
}

impl<T, K: Into<usize>> IndexOps<T, K> for VecIndexBuilder<T, K> {
    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn iter<'s>(&'s self) -> impl Iterator<Item = &'s T> + 's
    where
        T: 's,
    {
        self.buffer.iter()
    }
}

pub struct VecIndex<T, K> {
    buffer: Box<[T]>,
    _key: PhantomData<K>,
}

impl<T, K: Into<usize>> IndexOps<T, K> for VecIndex<T, K> {
    fn iter<'s>(&'s self) -> impl Iterator<Item = &'s T> + 's
    where
        T: 's,
    {
        self.buffer.iter()
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

impl<T, K: Into<usize>> Index<T, K> for VecIndex<T, K> {
    fn get(&self, key: K) -> &T {
        unsafe { self.buffer.get_unchecked(key.into()) }
    }

    fn get_mut(&mut self, key: K) -> &mut T {
        unsafe { self.buffer.get_unchecked_mut(key.into()) }
    }
}
