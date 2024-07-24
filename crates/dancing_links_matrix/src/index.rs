//! Allocator for vector backed allocation of types.
//!
//! This module provides a trait `Allocator` that defines an interface for
//! allocating values of type `T` and returning a key of type `K`.
//!
//! The `VecAllocator` type implements this trait and uses a `Vec<T>` as the
//! underlying storage.

use std::cell::UnsafeCell;

pub trait IndexOps<T> {
    /// Returns an iterator over the values in the allocator.
    ///
    /// The iterator yields `&T`, where `&T` is a reference to the value.
    fn iter<'s>(&'s self) -> impl Iterator<Item = &'s T> + 's
    where
        T: 's;

    /// Returns the number of values currently in the allocator.
    fn len(&self) -> usize;
}

pub trait IndexBuilder<T>: IndexOps<T> {
    type Index<U>;

    /// Returns the next key that can be used to allocate a value of type `T`.
    fn next_key(&self) -> usize;

    /// Inserts a value of type `T` into the allocator and returns the key
    /// associated with the value.
    fn insert(&mut self, val: T) -> usize;

    fn finalize<U>(self, mapper: impl Fn(Vec<T>) -> Vec<U>) -> Self::Index<U>;

    fn get_mut(&mut self, key: usize) -> &mut T;
}

/// A trait for allocating values of type `T` and returning a key of type `K`.
pub trait Index<T>: IndexOps<T> {
    fn get(&self, key: usize) -> &T;

    fn get_mut(&mut self, key: usize) -> &mut T;
}

pub struct VecIndexBuilder<T> {
    buffer: Vec<T>,
}

impl<T> VecIndexBuilder<T> {
    /// Creates a new `VecIndexBuilder`.
    pub(crate) fn new() -> VecIndexBuilder<T> {
        VecIndexBuilder { buffer: Vec::new() }
    }

    /// Creates a new `VecIndexBuilder` with the specified capacity.
    pub(crate) fn with_capacity(len: usize) -> VecIndexBuilder<T> {
        VecIndexBuilder {
            buffer: Vec::with_capacity(len),
        }
    }
}

impl<T> IndexBuilder<T> for VecIndexBuilder<T> {
    type Index<U> = VecIndex<U>;

    fn next_key(&self) -> usize {
        self.buffer.len()
    }

    fn insert(&mut self, val: T) -> usize {
        self.buffer.push(val);
        self.buffer.len() - 1
    }

    fn finalize<U>(self, mapper: impl FnOnce(Vec<T>) -> Vec<U>) -> Self::Index<U> {
        VecIndex {
            buffer: UnsafeCell::new(mapper(self.buffer).into_boxed_slice()),
        }
    }

    fn get_mut(&mut self, key: usize) -> &mut T {
        self.buffer.get_mut(key).unwrap()
    }
}

impl<T> IndexOps<T> for VecIndexBuilder<T> {
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

pub struct VecIndex<T> {
    buffer: UnsafeCell<Box<[T]>>,
}

impl<T> VecIndex<T> {
    pub(crate) fn get_ptr(&self, k: usize) -> *mut T {
        unsafe { (*self.buffer.get()).as_mut_ptr().add(k) }
    }

    pub(crate) fn get(&self, k: usize) -> &T {
        unsafe { (*self.buffer.get()).get(k).unwrap() }
    }

    pub(crate) fn iter_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut T> + 's
    where
        T: 's,
    {
        unsafe { (*self.buffer.get()).iter_mut() }
    }
}

impl<T> IndexOps<T> for VecIndex<T> {
    fn iter<'s>(&'s self) -> impl Iterator<Item = &'s T> + 's
    where
        T: 's,
    {
        unsafe { (*self.buffer.get()).iter() }
    }

    fn len(&self) -> usize {
        unsafe { (*self.buffer.get()).len() }
    }
}
