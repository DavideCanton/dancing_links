use slab::Slab;
use std::{borrow::Cow, collections::HashSet, fmt::Display};

use crate::cells::{Cell as _Cell, CellRow, HeaderCell as _HeaderCell, HeaderName};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Key(usize);

impl Display for Key {
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

type Cell = _Cell<Key>;
type HeaderCell = _HeaderCell<Key>;

pub struct MatrixBuilder {
    columns: Vec<ColumnSpec>,
}

impl MatrixBuilder {
    pub fn new() -> MatrixBuilder {
        MatrixBuilder { columns: vec![] }
    }

    pub fn add_primary_column(mut self, name: usize) -> MatrixBuilder {
        self.columns.push(ColumnSpec::primary(name));
        self
    }

    pub fn add_column(mut self, spec: ColumnSpec) -> MatrixBuilder {
        self.columns.push(spec);
        self
    }

    pub fn build(self) -> DancingLinksMatrix {
        let column_names = self.columns;

        let headers = Slab::with_capacity(column_names.len());
        let mut matrix = DancingLinksMatrix {
            header_key: headers.vacant_key().into(),
            rows: 0,
            columns: column_names.len(),
            headers,
            cells: Slab::with_capacity(column_names.len() * 5),
        };

        let (header_key, _) = matrix.add_header(HeaderName::First);
        matrix.header_key = header_key;

        let mut prev_key = header_key;

        for spec in column_names {
            let primary = spec.primary;
            let (_, cell_key) = matrix.add_header(HeaderName::Other(spec.name));

            if primary {
                matrix.link_right(prev_key, cell_key);
                prev_key = cell_key;
            }
        }

        matrix.link_right(prev_key, header_key);

        matrix
    }
}

pub struct DancingLinksMatrix {
    header_key: Key,
    rows: usize,
    columns: usize,
    headers: Slab<HeaderCell>,
    cells: Slab<Cell>,
}

impl DancingLinksMatrix {
    pub fn add_sparse_row(&mut self, row: &[usize], already_sorted: bool) {
        let mut sorted = Cow::from(row);
        if !already_sorted {
            sorted.to_mut().sort_unstable();
        }

        let mut cell_index = None;
        let mut prev_index = None;
        let mut start_index = None;

        for ind in sorted.iter().map(|&i| i.into()) {
            // TODO check if ind is valid

            let in_cell_index = self.add_cell(self.header(ind).cell, CellRow::Data(self.rows));
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

    pub fn iter_rows(&self) -> RowIterator {
        RowIterator::new(self)
    }

    fn add_cell(&mut self, header_cell_key: Key, row: CellRow) -> Key {
        let cell_key = self.cells.vacant_key().into();
        let cell = Cell::new(cell_key, header_cell_key, row);
        let actual_key = self.cells.insert(cell).into();
        assert_eq!(actual_key, cell_key);
        actual_key
    }

    fn add_header(&mut self, name: HeaderName) -> (Key, Key) {
        let header_key = self.headers.vacant_key().into();
        let header_cell_key = self.add_cell(header_key, CellRow::Header);

        let header = HeaderCell::new(name, header_cell_key);
        let actual_header_key = self.headers.insert(header).into();
        assert_eq!(header_key, actual_header_key);

        (actual_header_key, header_cell_key)
    }

    fn cell(&self, key: Key) -> &Cell {
        &self.cells[key.into()]
    }

    fn cell_mut(&mut self, key: Key) -> &mut Cell {
        &mut self.cells[key.into()]
    }

    fn header_mut(&mut self, key: Key) -> &mut HeaderCell {
        &mut self.headers[key.into()]
    }

    fn header(&self, key: Key) -> &HeaderCell {
        &self.headers[key.into()]
    }

    fn link_right(&mut self, left: Key, right: Key) {
        self.cell_mut(left).right = right;
        self.cell_mut(right).left = left;
    }

    fn link_down(&mut self, up: Key, down: Key) {
        self.cell_mut(up).down = down;
        self.cell_mut(down).up = up;
    }
}

pub struct RowIterator<'a> {
    matrix: &'a DancingLinksMatrix,
    last: usize,
}

impl<'a> RowIterator<'a> {
    fn new(matrix: &'a DancingLinksMatrix) -> RowIterator<'a> {
        RowIterator { matrix, last: 0 }
    }
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = HashSet<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last >= self.matrix.cells.len() {
            return None;
        }

        let mut set = HashSet::new();
        let mut i = self.last;
        let mut cur_row = None;
        let mut data_found = false;

        while i < self.matrix.cells.len() {
            let c = self.matrix.cell(i.into());

            if let CellRow::Data(row) = c.row {
                data_found = true;
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

                match self.matrix.header(c.header).name {
                    HeaderName::First => panic!("A cell should not have the first header as name"),
                    HeaderName::Other(name) => {
                        set.insert(name);
                    }
                }
            }

            i += 1;
        }

        self.last = i;

        data_found.then_some(set)
    }
}

impl Display for DancingLinksMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut matrix = String::new();

        let mut is_first = true;

        for (_, header) in self.headers.iter() {
            if is_first {
                is_first = false;
            } else {
                matrix.push(' ');
            }
            matrix.push_str(&format!("{}", header.name));
        }

        matrix.push('\n');

        is_first = true;

        for (k, cell) in self.cells.iter() {
            if is_first {
                is_first = false;
            } else {
                matrix.push('\n');
            }

            matrix.push_str(&format!(
                "{} {} {} {} {} {} {:?}",
                k, cell.up, cell.down, cell.left, cell.right, cell.header, cell.row
            ));
        }
        write!(f, "{}", matrix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator() {
        let mut matrix = MatrixBuilder::new()
            .add_primary_column(1)
            .add_primary_column(2)
            .add_primary_column(3)
            .build();

        matrix.add_sparse_row(&[1, 2], true);
        matrix.add_sparse_row(&[1, 3], true);
        matrix.add_sparse_row(&[2, 3], true);
        matrix.add_sparse_row(&[1, 2, 3], true);

        let mut it = matrix.iter_rows();
        assert_eq!(it.next().unwrap(), HashSet::from([1, 2]));
        assert_eq!(it.next().unwrap(), HashSet::from([1, 3]));
        assert_eq!(it.next().unwrap(), HashSet::from([2, 3]));
        assert_eq!(it.next().unwrap(), HashSet::from([1, 2, 3]));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_no_rows() {
        let matrix = MatrixBuilder::new()
            .add_primary_column(1)
            .add_primary_column(2)
            .add_primary_column(3)
            .build();

        let mut it = matrix.iter_rows();
        assert_eq!(it.next(), None);
    }
}
