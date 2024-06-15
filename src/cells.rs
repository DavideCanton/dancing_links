use std::fmt::Display;

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
pub(crate) struct Cell<K, H> {
    pub(crate) index: K,
    pub(crate) up: K,
    pub(crate) down: K,
    pub(crate) left: K,
    pub(crate) right: K,
    pub(crate) header: H,
    pub(crate) row: CellRow,
}

impl<K: Copy + Clone, H: Copy + Clone> Cell<K, H> {
    pub fn new(index: K, header: H, row: CellRow) -> Cell<K, H> {
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
pub(crate) enum HeaderName {
    First,
    Other(usize),
}

impl From<usize> for HeaderName {
    fn from(name: usize) -> Self {
        match name {
            0 => HeaderName::First,
            name => HeaderName::Other(name),
        }
    }
}

impl Display for HeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderName::First => "<H>".fmt(f),
            HeaderName::Other(name) => name.fmt(f),
        }
    }
}

#[derive(Debug)]
pub(crate) struct HeaderCell<K, H> {
    pub(crate) index: H,
    pub(crate) name: HeaderName,
    pub(crate) size: usize,
    pub(crate) cell: K,
}

impl<K: Copy + Clone, H: Copy + Clone> HeaderCell<K, H> {
    pub fn new(name: HeaderName, index: H, cell_index: K) -> HeaderCell<K, H> {
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
        let cell = Cell::new(42, 2, CellRow::Data(3));
        assert_eq!(cell.index, 42);
        assert_eq!(cell.up, 42);
        assert_eq!(cell.down, 42);
        assert_eq!(cell.left, 42);
        assert_eq!(cell.right, 42);
        assert_eq!(cell.header, 2);
        assert_eq!(cell.row, CellRow::Data(3));
    }

    #[test]
    fn test_header_cell_new() {
        let header_cell = HeaderCell::new(HeaderName::Other(1), 3, 2);
        assert_eq!(header_cell.name, HeaderName::Other(1));
        assert_eq!(header_cell.size, 0);
        assert_eq!(header_cell.index, 3);
        assert!(!header_cell.is_first());
        assert_eq!(header_cell.cell, 2);

        let header_cell = HeaderCell::new(HeaderName::First, 10, 12);
        assert_eq!(header_cell.name, HeaderName::First);
        assert_eq!(header_cell.size, 0);
        assert_eq!(header_cell.index, 10);
        assert!(header_cell.is_first());
        assert_eq!(header_cell.cell, 12);
    }
}
