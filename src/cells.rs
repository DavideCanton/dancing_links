use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CellRow {
    Header,
    Data(usize),
}

#[derive(Debug)]
pub(crate) struct Cell<K> {
    pub(crate) index: K,
    pub(crate) up: K,
    pub(crate) down: K,
    pub(crate) left: K,
    pub(crate) right: K,
    pub(crate) header: K,
    pub(crate) row: CellRow,
}

impl<K: Copy + Clone> Cell<K> {
    pub fn new(index: K, header: K, row: CellRow) -> Cell<K> {
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

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) enum HeaderName {
    First,
    Other(usize),
}

#[derive(Debug)]
pub(crate) struct HeaderCell<K> {
    pub(crate) name: HeaderName,
    pub(crate) size: usize,
    pub(crate) cell: K,
}

impl Display for HeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderName::First => write!(f, "<H>"),
            HeaderName::Other(name) => write!(f, "{}", name),
        }
    }
}

impl<K: Copy + Clone> HeaderCell<K> {
    pub fn new(name: HeaderName, cell_index: K) -> HeaderCell<K> {
        HeaderCell {
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
        let header_cell = HeaderCell::new(HeaderName::Other(1), 2);
        assert_eq!(header_cell.name, HeaderName::Other(1));
        assert_eq!(header_cell.size, 0);
        assert!(!header_cell.is_first());
        assert_eq!(header_cell.cell, 2);

        let header_cell = HeaderCell::new(HeaderName::First, 12);
        assert_eq!(header_cell.name, HeaderName::First);
        assert_eq!(header_cell.size, 0);
        assert!(header_cell.is_first());
        assert_eq!(header_cell.cell, 12);
    }
}
