use bumpalo::Bump;
use dancing_links_matrix::Arena;

pub struct BumpArena(Bump);

impl From<Bump> for BumpArena {
    fn from(value: Bump) -> Self {
        BumpArena(value)
    }
}

impl Arena for BumpArena {
    fn alloc<T>(&self, val: T) -> &T {
        &*self.0.alloc(val)
    }
}
