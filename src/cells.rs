#[derive(Debug)]
pub enum CellRow {
    Header,
    Data(usize),
}

#[derive(Debug)]
pub struct Cell {
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
    pub(crate) name: String,
    pub(crate) first: bool,
    pub(crate) cell: usize,
}

impl HeaderCell {
    pub fn new(name: &str, first: bool, cell_index: usize) -> HeaderCell {
        HeaderCell {
            size: 0,
            name: name.to_string(),
            first,
            cell: cell_index,
        }
    }
}
