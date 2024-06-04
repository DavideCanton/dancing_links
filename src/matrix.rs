use std::{borrow::Cow, fmt::Debug};

use crate::cells::{Cell, CellRow, HeaderCell};

pub struct ColumnSpec {
    name: String,
    primary: bool,
}

impl ColumnSpec {
    pub fn primary(name: &str) -> ColumnSpec {
        ColumnSpec {
            name: name.to_string(),
            primary: true,
        }
    }

    pub fn secondary(name: &str) -> ColumnSpec {
        ColumnSpec {
            name: name.to_string(),
            primary: false,
        }
    }
}

impl From<&str> for ColumnSpec {
    fn from(name: &str) -> Self {
        Self::primary(name)
    }
}

pub struct DancingLinksMatrix {
    header: usize,
    rows: usize,
    columns: usize,
    headers_vec: Vec<HeaderCell>,
    cells_vec: Vec<Cell>,
}

impl DancingLinksMatrix {
    pub fn new(names: &[ColumnSpec]) -> DancingLinksMatrix {
        let mut matrix = DancingLinksMatrix {
            header: 0,
            rows: 0,
            columns: names.len(),
            headers_vec: Vec::with_capacity(names.len()),
            cells_vec: Vec::with_capacity(names.len() * 4),
        };

        let (header_index, _) = matrix.add_header("<H>", true, 0);
        matrix.header = header_index;

        let mut prev_index = header_index;

        for (n, spec) in names.iter().enumerate() {
            let primary = spec.primary;
            let (_, cell_index) = matrix.add_header(&spec.name, false, n);

            if primary {
                matrix.cell_mut(prev_index).right = cell_index;
                matrix.cell_mut(cell_index).left = prev_index;
                prev_index = cell_index;
            }
        }

        matrix.cell_mut(prev_index).right = header_index;
        matrix.cell_mut(header_index).left = prev_index;

        matrix
    }

    pub fn add_sparse_row(&mut self, row: &[usize], already_sorted: bool) {
        let mut sorted = Cow::from(row);
        if !already_sorted {
            sorted.to_mut().sort_unstable();
        }

        let mut cell_index = None;
        let mut prev_index = None;
        let mut start_index = None;

        for &ind in sorted.iter() {
            let in_cell_index = self.add_cell(self.header(ind).cell, CellRow::Data(self.rows), ind);
            cell_index = Some(in_cell_index);

            match prev_index {
                Some(prev_index) => {
                    self.cell_mut(prev_index).right = in_cell_index;
                    self.cell_mut(in_cell_index).left = prev_index;
                }
                None => {
                    start_index = cell_index;
                }
            }

            let header = self.header_mut(ind);
            let header_cell_index = header.cell;
            header.size += 1;
            let header_cell = self.cell_mut(header_cell_index);

            let last = header_cell.up;
            header_cell.up = in_cell_index;

            self.cell_mut(last).down = in_cell_index;

            let cell_mut = self.cell_mut(in_cell_index);
            cell_mut.up = last;
            cell_mut.down = header_cell_index;

            prev_index = cell_index;
        }

        self.cell_mut(start_index.unwrap()).left = cell_index.unwrap();
        self.cell_mut(cell_index.unwrap()).right = start_index.unwrap();

        self.rows += 1;
    }

    fn add_cell(&mut self, header: usize, row: CellRow, column: usize) -> usize {
        let index = self.cells_vec.len();
        let cell = Cell::new(index, header, row, column);
        self.cells_vec.push(cell);
        index
    }

    fn add_header(&mut self, name: &str, first: bool, column: usize) -> (usize, usize) {
        let cell_index = self.add_cell(999, CellRow::Header, column);
        let header = HeaderCell::new(name, first, cell_index);
        let index = self.headers_vec.len();
        self.headers_vec.push(header);
        self.cell_mut(cell_index).header = index;
        (index, cell_index)
    }

    fn cell_mut(&mut self, index: usize) -> &mut Cell {
        &mut self.cells_vec[index]
    }

    fn header_mut(&mut self, index: usize) -> &mut HeaderCell {
        &mut self.headers_vec[index]
    }

    fn header(&self, index: usize) -> &HeaderCell {
        &self.headers_vec[index]
    }
}

impl Debug for DancingLinksMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut matrix = String::new();

        for (i, header) in self.headers_vec.iter().enumerate() {
            if i != 0 {
                matrix.push(' ');
            }
            matrix.push_str(&header.name);
        }
        matrix.push('\n');

        for (i, cell) in self.cells_vec.iter().enumerate() {
            if i != 0 {
                matrix.push('\n');
            }

            matrix.push_str(&format!(
                "{} {} {} {} {} {:?} {:?} {}",
                i, cell.up, cell.down, cell.left, cell.right, cell.header, cell.row, cell.column
            ));
        }
        write!(f, "{}", matrix)
    }
}
