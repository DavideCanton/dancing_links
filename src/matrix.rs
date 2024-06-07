use std::{borrow::Cow, collections::HashSet, fmt::Debug};

use crate::cells::{Cell, CellRow, HeaderCell};

pub struct ColumnSpec {
    name: usize,
    primary: bool,
}

impl ColumnSpec {
    pub fn primary(name: usize) -> ColumnSpec {
        ColumnSpec {
            name,
            primary: true,
        }
    }

    pub fn secondary(name: usize) -> ColumnSpec {
        ColumnSpec {
            name,
            primary: false,
        }
    }
}

impl From<usize> for ColumnSpec {
    fn from(name: usize) -> Self {
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

        let (header_index, _) = matrix.add_header(0, true, 0);
        matrix.header = header_index;

        let mut prev_index = header_index;

        for (n, spec) in names.iter().enumerate() {
            let primary = spec.primary;
            let (_, cell_index) = matrix.add_header(spec.name, false, n);

            if primary {
                matrix.link_right(prev_index, cell_index);
                prev_index = cell_index;
            }
        }

        matrix.link_right(prev_index, header_index);
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
                    self.link_right(prev_index, in_cell_index);
                }
                None => {
                    start_index = cell_index;
                }
            }

            let header = self.header_mut(ind);
            let header_cell_index = header.cell;
            header.size += 1;
            let last = self.cell_mut(header_cell_index).up;

            self.link_down(in_cell_index, header_cell_index);
            self.link_down(last, in_cell_index);

            prev_index = cell_index;
        }

        self.link_right(cell_index.unwrap(), start_index.unwrap());

        self.rows += 1;
    }

    pub fn iter_rows(&self) -> SetIterator {
        SetIterator::new(self)
    }

    fn add_cell(&mut self, header: usize, row: CellRow, column: usize) -> usize {
        let index = self.cells_vec.len();
        let cell = Cell::new(index, header, row, column);
        self.cells_vec.push(cell);
        index
    }

    fn add_header(&mut self, name: usize, first: bool, column: usize) -> (usize, usize) {
        let cell_index = self.add_cell(999, CellRow::Header, column);
        let header = HeaderCell::new(name, first, cell_index);
        let index = self.headers_vec.len();
        self.headers_vec.push(header);
        self.cell_mut(cell_index).header = index;
        (index, cell_index)
    }

    fn cell(&self, index: usize) -> &Cell {
        &self.cells_vec[index]
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

    fn link_right(&mut self, left: usize, right: usize) {
        self.cell_mut(left).right = right;
        self.cell_mut(right).left = left;
    }

    fn link_down(&mut self, up: usize, down: usize) {
        self.cell_mut(up).down = down;
        self.cell_mut(down).up = up;
    }
}

pub struct SetIterator<'a> {
    matrix: &'a DancingLinksMatrix,
    last: usize,
}

impl<'a> SetIterator<'a> {
    fn new(matrix: &'a DancingLinksMatrix) -> SetIterator<'a> {
        SetIterator { matrix, last: 0 }
    }
}

impl<'a> Iterator for SetIterator<'a> {
    type Item = HashSet<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last >= self.matrix.cells_vec.len() {
            return None;
        }

        let mut set = HashSet::new();
        let mut i = self.last;
        let mut cur_row = None;

        while i < self.matrix.cells_vec.len() {
            let c = self.matrix.cell(i);

            if let CellRow::Data(row) = c.row {
                match cur_row {
                    Some(cur_row) => {
                        if cur_row != row {
                            break;
                        }
                    }
                    None => {
                        cur_row.replace(row);
                    }
                }
                set.insert(self.matrix.header(c.header).name);
            }

            i += 1;
        }

        self.last = i;
        Some(set)
    }
}

impl Debug for DancingLinksMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut matrix = String::new();

        for (i, header) in self.headers_vec.iter().enumerate() {
            if i != 0 {
                matrix.push(' ');
            }
            matrix.push_str(&format!("{}", header.name));
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
