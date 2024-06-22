#![allow(dead_code)]
use std::ops::{Index, IndexMut};

pub trait Allocator<T, K> {
    fn next_key(&self) -> K;
    fn insert(&mut self, val: T) -> K;
    fn len(&self) -> usize;
    fn iter<'s>(&'s self) -> impl Iterator<Item = (K, &'s T)> + 's
    where
        T: 's;
}

pub struct VecAllocator<T> {
    buffer: Vec<T>,
}

impl<T> VecAllocator<T> {
    pub(crate) fn new() -> VecAllocator<T> {
        VecAllocator { buffer: Vec::new() }
    }

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

    fn iter<'a>(&'a self) -> impl Iterator<Item = (usize, &'a T)> + 'a
    where
        T: 'a,
    {
        self.buffer.iter().enumerate()
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
