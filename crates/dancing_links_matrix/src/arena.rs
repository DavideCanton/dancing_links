pub trait Arena {
    fn alloc<T>(&self, val: T) -> &T;
}
