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

    pub fn is_first(&self) -> bool {
        matches!(self.name, HeaderName::First)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_new() {
        let cell = Cell::new(42.into(), 2.into(), CellRow::Data(3));
        assert_eq!(cell.index, 42.into());
        assert_eq!(cell.up, 42.into());
        assert_eq!(cell.down, 42.into());
        assert_eq!(cell.left, 42.into());
        assert_eq!(cell.right, 42.into());
        assert_eq!(cell.header, 2.into());
        assert_eq!(cell.row, CellRow::Data(3));
    }

    #[test]
    fn test_header_cell_new() {
        let header_cell = HeaderCell::new(HeaderName::Other(1), 3.into(), 2.into());
        assert_eq!(header_cell.name, HeaderName::Other(1));
        assert_eq!(header_cell.size, 0);
        assert_eq!(header_cell.index, 3.into());
        assert!(!header_cell.is_first());
        assert_eq!(header_cell.cell, 2.into());

        let header_cell = HeaderCell::<u32>::new(HeaderName::First, 10.into(), 12.into());
        assert_eq!(header_cell.name, HeaderName::First);
        assert_eq!(header_cell.size, 0);
        assert_eq!(header_cell.index, 10.into());
        assert!(header_cell.is_first());
        assert_eq!(header_cell.cell, 12.into());
    }
}
