use std::cell::UnsafeCell;

pub trait IndexOps<T> {
    fn iter<'s>(&'s self) -> impl Iterator<Item = &'s T> + 's
    where
        T: 's;

    /// Returns the number of values currently in the allocator.
    fn len(&self) -> usize;
}

pub trait IndexBuilder<T>: IndexOps<T> {
    type Index<U>;

    fn insert(&mut self, val: T);

    fn finalize<U>(self, mapper: impl Fn(Vec<T>) -> Vec<U>) -> Self::Index<U>;

    fn get_mut(&mut self, index: usize) -> &mut T;
}

pub trait Index<T>: IndexOps<T> {
    unsafe fn get_mut_ptr(&self, index: usize) -> *mut T;

    fn get(&self, index: usize) -> &T;
}

pub struct VecIndexBuilder<T> {
    buffer: Vec<T>,
}

impl<T> VecIndexBuilder<T> {
    pub(crate) fn new() -> VecIndexBuilder<T> {
        VecIndexBuilder { buffer: Vec::new() }
    }

    pub(crate) fn with_capacity(len: usize) -> VecIndexBuilder<T> {
        VecIndexBuilder {
            buffer: Vec::with_capacity(len),
        }
    }
}

impl<T> IndexBuilder<T> for VecIndexBuilder<T> {
    type Index<U> = VecIndex<U>;

    fn insert(&mut self, val: T) {
        self.buffer.push(val);
    }

    fn finalize<U>(self, mapper: impl FnOnce(Vec<T>) -> Vec<U>) -> Self::Index<U> {
        VecIndex {
            buffer: UnsafeCell::new(mapper(self.buffer).into_boxed_slice()),
        }
    }

    fn get_mut(&mut self, index: usize) -> &mut T {
        self.buffer.get_mut(index).unwrap()
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
    pub(crate) fn iter_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut T> + 's
    where
        T: 's,
    {
        unsafe { (*self.buffer.get()).iter_mut() }
    }
}

impl<T> Index<T> for VecIndex<T> {
    unsafe fn get_mut_ptr(&self, index: usize) -> *mut T {
        (*self.buffer.get()).as_mut_ptr().add(index)
    }

    fn get(&self, index: usize) -> &T {
        unsafe { (*self.buffer.get()).get(index).unwrap() }
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
