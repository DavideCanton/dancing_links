use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Key(usize);

impl fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for Key {
    fn from(name: usize) -> Self {
        Key(name)
    }
}

impl From<Key> for usize {
    fn from(key: Key) -> Self {
        key.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct HeaderKey(usize);

impl fmt::Display for HeaderKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for HeaderKey {
    fn from(name: usize) -> Self {
        HeaderKey(name)
    }
}

impl From<HeaderKey> for usize {
    fn from(key: HeaderKey) -> Self {
        key.0
    }
}
