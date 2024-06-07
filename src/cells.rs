#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellRow {
    Header,
    Data(usize),
}

#[derive(Debug)]
pub struct Cell {
    pub(crate) index: usize,
    pub(crate) up: usize,
    pub(crate) down: usize,
    pub(crate) left: usize,
    pub(crate) right: usize,
    pub(crate) header: usize,
    pub(crate) row: CellRow,
    pub(crate) column: usize,
}

impl Cell {
    pub fn new(index: usize, header: usize, row: CellRow, column: usize) -> Cell {
        Cell {
            index,
            up: index,
            down: index,
            left: index,
            right: index,
            header,
            row,
            column,
        }
    }
}

#[derive(Debug)]
pub struct HeaderCell {
    pub(crate) size: usize,
    pub(crate) name: usize,
    pub(crate) first: bool,
    pub(crate) cell: usize,
}

impl HeaderCell {
    pub fn new(name: usize, first: bool, cell_index: usize) -> HeaderCell {
        HeaderCell {
            size: 0,
            name,
            first,
            cell: cell_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_new() {
        let cell = Cell::new(42, 2, CellRow::Data(3), 4);
        assert_eq!(cell.index, 42);
        assert_eq!(cell.up, 42);
        assert_eq!(cell.down, 42);
        assert_eq!(cell.left, 42);
        assert_eq!(cell.right, 42);
        assert_eq!(cell.header, 2);
        assert_eq!(cell.row, CellRow::Data(3));
        assert_eq!(cell.column, 4);
    }

    #[test]
    fn test_header_cell_new() {
        let header_cell = HeaderCell::new(1, true, 2);
        assert_eq!(header_cell.size, 0);
        assert_eq!(header_cell.name, 1);
        assert!(header_cell.first);
        assert_eq!(header_cell.cell, 2);

        let header_cell = HeaderCell::new(1, false, 2);
        assert_eq!(header_cell.size, 0);
        assert_eq!(header_cell.name, 1);
        assert!(!header_cell.first);
        assert_eq!(header_cell.cell, 2);
    }
}
