use bumpalo::Bump;

use crate::Arena;

pub(super) fn create_row<const N: usize>(v: [&str; N]) -> Vec<String> {
    v.iter().map(|v| v.to_string()).collect()
}

pub(super) struct BumpArena(Bump);

impl Arena for BumpArena {
    fn alloc<T>(&self, val: T) -> &T {
        self.0.alloc(val)
    }
}

impl From<Bump> for BumpArena {
    fn from(value: Bump) -> Self {
        BumpArena(value)
    }
}
