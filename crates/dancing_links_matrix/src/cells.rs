use std::fmt::Display;

use crate::keys::{HeaderKey, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CellRow {
    Header,
    Data(usize),
}

impl From<usize> for CellRow {
    fn from(name: usize) -> Self {
        match name {
            0 => CellRow::Header,
            name => CellRow::Data(name),
        }
    }
}

impl Display for CellRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellRow::Header => "H".fmt(f),
            CellRow::Data(name) => name.fmt(f),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Cell {
    pub(crate) index: Key,
    pub(crate) up: Key,
    pub(crate) down: Key,
    pub(crate) left: Key,
    pub(crate) right: Key,
    pub(crate) header: HeaderKey,
    pub(crate) row: CellRow,
}

impl Cell {
    pub fn new(index: Key, header: HeaderKey, row: CellRow) -> Cell {
        Cell {
            index,
            up: index,
            down: index,
            left: index,
            right: index,
            header,
            row,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) enum HeaderName<T> {
    First,
    Other(T),
}

impl<T: Display> Display for HeaderName<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderName::First => "<H>".fmt(f),
            HeaderName::Other(name) => name.fmt(f),
        }
    }
}

#[derive(Debug)]
pub(crate) struct HeaderCell<T> {
    pub(crate) index: HeaderKey,
    pub(crate) name: HeaderName<T>,
    pub(crate) size: usize,
    pub(crate) cell: Key,
}

impl<T> HeaderCell<T> {
    pub fn new(name: HeaderName<T>, index: HeaderKey, cell_index: Key) -> HeaderCell<T> {
        HeaderCell {
            index,
            name,
            size: 0,
            cell: cell_index,
        }
    }

    #[allow(dead_code)]
    pub fn is_first(&self) -> bool {
        matches!(self.name, HeaderName::First)
    }
}
