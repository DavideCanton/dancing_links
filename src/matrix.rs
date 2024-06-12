use slab::Slab;
use std::{
    collections::HashSet,
    fmt::{self},
};

use crate::cells::{Cell as _Cell, CellRow, HeaderCell as _HeaderCell, HeaderName};
use crate::keys::{HeaderKey, Key};

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

type Cell = _Cell<Key, HeaderKey>;
type HeaderCell = _HeaderCell<Key, HeaderKey>;

pub struct MatrixBuilder;

impl MatrixBuilder {
    pub fn add_primary_column(self, name: usize) -> MatrixColBuilder {
        MatrixColBuilder::new().add_primary_column(name)
    }

    pub fn add_column(self, spec: ColumnSpec) -> MatrixColBuilder {
        MatrixColBuilder::new().add_column(spec)
    }
}

pub struct MatrixColBuilder {
    columns: Vec<ColumnSpec>,
}

impl MatrixColBuilder {
    fn new() -> MatrixColBuilder {
        MatrixColBuilder { columns: vec![] }
    }

    pub fn add_primary_column(mut self, name: usize) -> MatrixColBuilder {
        self.columns.push(ColumnSpec::primary(name));
        self
    }

    pub fn add_column(mut self, spec: ColumnSpec) -> MatrixColBuilder {
        self.columns.push(spec);
        self
    }

    pub fn end_columns(self) -> MatrixRowBuilder {
        let column_names = self.columns;

        let headers = Slab::with_capacity(column_names.len());
        let mut matrix = DancingLinksMatrix {
            header_key: headers.vacant_key().into(),
            rows: 0,
            columns: column_names.len(),
            headers,
            cells: Slab::with_capacity(column_names.len() * 5),
        };

        let (header_key, header_cell_key) = matrix.add_header(HeaderName::First);
        matrix.header_key = header_key;

        let mut prev_cell_key = header_cell_key;

        for spec in column_names {
            let primary = spec.primary;
            let (_, cell_key) = matrix.add_header(HeaderName::Other(spec.name));

            if primary {
                matrix.link_right(prev_cell_key, cell_key);
                prev_cell_key = cell_key;
            }
        }

        matrix.link_right(prev_cell_key, header_cell_key);

        MatrixRowBuilder { matrix }
    }
}

pub struct MatrixRowBuilder {
    matrix: DancingLinksMatrix,
}

impl MatrixRowBuilder {
    pub fn add_row(self, row: &[usize]) -> Self {
        let mut sorted = row.to_vec();
        sorted.sort_unstable();
        self.add_sorted_row(&sorted)
    }

    pub fn add_sorted_row(mut self, row: &[usize]) -> Self {
        let mx = &mut self.matrix;

        let mut cell_index = None;
        let mut prev_index = None;
        let mut start_index = None;

        for ind in row.iter() {
            // TODO check if ind is valid
            let header_key = HeaderKey::from(*ind);

            let in_cell_index = mx.add_cell(header_key, CellRow::Data(mx.rows + 1));
            cell_index = Some(in_cell_index);

            match prev_index {
                Some(prev_index) => {
                    mx.link_right(prev_index, in_cell_index);
                }
                None => {
                    start_index = cell_index;
                }
            }

            let header = mx.header_mut(header_key);
            let header_cell_index = header.cell;
            header.size += 1;
            let last = mx.cell_mut(header_cell_index).up;

            mx.link_down(in_cell_index, header_cell_index);
            mx.link_down(last, in_cell_index);

            prev_index = cell_index;
        }

        mx.link_right(cell_index.unwrap(), start_index.unwrap());

        mx.rows += 1;
        self
    }

    pub fn build(self) -> DancingLinksMatrix {
        self.matrix
    }
}

pub struct DancingLinksMatrix {
    header_key: HeaderKey,
    rows: usize,
    columns: usize,
    headers: Slab<HeaderCell>,
    cells: Slab<Cell>,
}

impl DancingLinksMatrix {
    pub fn iter_rows(&self) -> RowIterator {
        RowIterator::new(self)
    }

    fn min_column(&self) -> Option<&HeaderCell> {
        if self.columns == 0 {
            return None;
        }

        todo!()
    }

    fn iterate_cells<F: Fn(&Cell) -> Key>(&self, start: Key, getter: F) -> CellIterator<F> {
        CellIterator::new(self, start, getter)
    }

    fn iterate_headers<F: Fn(&Cell) -> Key>(
        &self,
        mut start: HeaderKey,
        getter: F,
    ) -> HeaderCellIterator<F> {
        if start == self.header_key {
            let cell = self.header(start).cell;
            let next_cell = getter(self.cell(cell));
            start = self.cell(next_cell).header;
        }

        HeaderCellIterator::new(self, start, getter)
    }

    fn add_cell(&mut self, header_cell_key: HeaderKey, row: CellRow) -> Key {
        let cell_key = self.cells.vacant_key().into();
        let cell = Cell::new(cell_key, header_cell_key, row);
        let actual_key = self.cells.insert(cell).into();
        assert_eq!(actual_key, cell_key);
        actual_key
    }

    fn add_header(&mut self, name: HeaderName) -> (HeaderKey, Key) {
        let header_key = self.headers.vacant_key().into();
        let header_cell_key = self.add_cell(header_key, CellRow::Header);

        let header = HeaderCell::new(name, header_key, header_cell_key);
        let actual_header_key = self.headers.insert(header).into();
        assert_eq!(header_key, actual_header_key);

        (actual_header_key, header_cell_key)
    }

    fn locate_cell<R: Into<CellRow>, C: Into<HeaderName>>(&self, row: R, column: C) -> Option<Key> {
        let header_key = self.locate_header(column)?;
        let header = self.header(header_key);
        let row = row.into();

        self.iterate_cells(header.cell, |c| c.down)
            .find(|c| c.row == row)
            .map(|c| c.index)
    }

    fn locate_header<C: Into<HeaderName>>(&self, column: C) -> Option<HeaderKey> {
        let column = column.into();
        self.iterate_headers(self.header_key, |c| c.right)
            .find(|h| h.name == column)
            .map(|h| h.index)
    }

    fn cell(&self, key: Key) -> &Cell {
        &self.cells[key.into()]
    }

    fn cell_mut(&mut self, key: Key) -> &mut Cell {
        &mut self.cells[key.into()]
    }

    fn header_mut(&mut self, key: HeaderKey) -> &mut HeaderCell {
        &mut self.headers[key.into()]
    }

    fn header(&self, key: HeaderKey) -> &HeaderCell {
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

impl fmt::Display for DancingLinksMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

struct CellIterator<'a, F> {
    matrix: &'a DancingLinksMatrix,
    start: Key,
    current: Key,
    getter: F,
    started: bool,
}

impl<'a, F> CellIterator<'a, F>
where
    F: Fn(&Cell) -> Key,
{
    fn new(matrix: &'a DancingLinksMatrix, start: Key, getter: F) -> CellIterator<'a, F> {
        CellIterator {
            matrix,
            start,
            current: start,
            getter,
            started: false,
        }
    }
}

impl<'a, F> Iterator for CellIterator<'a, F>
where
    F: Fn(&Cell) -> Key,
{
    type Item = &'a Cell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.start && self.started {
            return None;
        }

        self.started = true;
        let cell = self.matrix.cell(self.current);
        self.current = (self.getter)(cell);
        Some(cell)
    }
}

struct HeaderCellIterator<'a, F> {
    matrix: &'a DancingLinksMatrix,
    start_key: HeaderKey,
    current_key: HeaderKey,
    getter: F,
    started: bool,
}

impl<'a, F> HeaderCellIterator<'a, F>
where
    F: Fn(&Cell) -> Key,
{
    fn new(
        matrix: &'a DancingLinksMatrix,
        start: HeaderKey,
        getter: F,
    ) -> HeaderCellIterator<'a, F> {
        HeaderCellIterator {
            matrix,
            start_key: start,
            current_key: start,
            getter,
            started: false,
        }
    }
}

impl<'a, F> Iterator for HeaderCellIterator<'a, F>
where
    F: Fn(&Cell) -> Key,
{
    type Item = &'a HeaderCell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_key == self.start_key && self.started {
            return None;
        }

        self.started = true;

        let to_return = self.matrix.header(self.current_key);

        for _ in 0..2 {
            let current_header = self.matrix.header(self.current_key);
            let current_header_cell = self.matrix.cell(current_header.cell);

            let next_header_cell_key = (self.getter)(current_header_cell);
            let next_header_key = self.matrix.cell(next_header_cell_key).header;

            self.current_key = next_header_key;

            if self.current_key != self.matrix.header_key {
                break;
            }
        }

        Some(to_return)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use HeaderName::Other as O;

    #[test]
    fn test_locate_cell() {
        let matrix = build_matrix();
        assert_eq!(matrix.locate_cell(1, 1).unwrap(), 4.into());
        assert_eq!(matrix.locate_cell(1, 2).unwrap(), 5.into());
        assert_eq!(matrix.locate_cell(1, 3), None);
        assert_eq!(matrix.locate_cell(2, 1).unwrap(), 6.into());
        assert_eq!(matrix.locate_cell(2, 2), None);
        assert_eq!(matrix.locate_cell(2, 3).unwrap(), 7.into());
    }

    #[test]
    fn test_locate_header() {
        let matrix = build_matrix();
        assert_eq!(matrix.locate_header(1).unwrap(), 1.into());
        assert_eq!(matrix.locate_header(2).unwrap(), 2.into());
        assert_eq!(matrix.locate_header(6), None);
    }

    #[test]
    fn test_iterator() {
        let matrix = build_matrix();

        let mut it = matrix.iter_rows();
        assert_eq!(it.next().unwrap(), HashSet::from([1, 2]));
        assert_eq!(it.next().unwrap(), HashSet::from([1, 3]));
        assert_eq!(it.next().unwrap(), HashSet::from([2, 3]));
        assert_eq!(it.next().unwrap(), HashSet::from([1, 2, 3]));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_no_rows() {
        let matrix = MatrixBuilder
            .add_primary_column(1)
            .add_primary_column(2)
            .add_primary_column(3)
            .end_columns()
            .build();

        let mut it = matrix.iter_rows();
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_header_cell_iterator_right_from_first() {
        let matrix = build_matrix();

        let actual: Vec<HeaderName> = matrix
            .iterate_headers(matrix.header_key, |cell| cell.right)
            .map(|h| h.name.clone())
            .collect();

        assert_eq!(actual, vec![O(1), O(2), O(3)]);
    }

    #[test]
    fn test_header_cell_iterator_right() {
        let matrix = build_matrix();
        let key = matrix.locate_header(1).unwrap();

        let actual: Vec<HeaderName> = matrix
            .iterate_headers(key, |cell| cell.right)
            .map(|h| h.name.clone())
            .collect();

        assert_eq!(actual, vec![O(1), O(2), O(3)]);
    }

    #[test]
    fn test_header_cell_iterator_left() {
        let matrix = build_matrix();
        let key = matrix.locate_header(1).unwrap();

        let actual: Vec<HeaderName> = matrix
            .iterate_headers(key, |cell| cell.left)
            .map(|h| h.name.clone())
            .collect();
        assert_eq!(actual, vec![O(1), O(3), O(2)]);
    }

    #[test]
    fn test_header_cell_iterator_up() {
        let matrix = build_matrix();
        let key = matrix.locate_header(1).unwrap();

        let actual: Vec<HeaderName> = matrix
            .iterate_headers(key, |c| c.up)
            .map(|h| h.name.clone())
            .collect();

        assert_eq!(actual, vec![O(1)]);
    }

    #[test]
    fn test_header_cell_iterator_down() {
        let matrix = build_matrix();
        let key = matrix.locate_header(1).unwrap();

        let actual: Vec<HeaderName> = matrix
            .iterate_headers(key, |c| c.down)
            .map(|h| h.name.clone())
            .collect();

        assert_eq!(actual, vec![O(1)]);
    }

    #[test]
    fn test_cell_iterator_left() {
        let matrix = build_matrix();

        let key = matrix.locate_cell(4, 2).unwrap();

        let actual: Vec<_> = matrix
            .iterate_cells(key, |cell| cell.left)
            .map(|h| h.index)
            .collect();

        assert_eq!(
            actual,
            [2, 1, 3]
                .into_iter()
                .map(|v| matrix.locate_cell(4, v).unwrap())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_cell_iterator_right() {
        let matrix = build_matrix();

        let key = matrix.locate_cell(4, 2).unwrap();

        let actual: Vec<_> = matrix
            .iterate_cells(key, |cell| cell.right)
            .map(|h| h.index)
            .collect();

        assert_eq!(
            actual,
            [2, 3, 1]
                .into_iter()
                .map(|v| matrix.locate_cell(4, v).unwrap())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_cell_iterator_up() {
        let matrix = build_matrix();

        let key = matrix.locate_cell(2, 1).unwrap();

        let actual: Vec<_> = matrix
            .iterate_cells(key, |cell| cell.up)
            .map(|h| h.index)
            .collect();

        assert_eq!(
            actual,
            [2, 1, 0, 4]
                .into_iter()
                .map(|v| matrix.locate_cell(v, 1).unwrap())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_cell_iterator_down() {
        let matrix = build_matrix();

        let key = matrix.locate_cell(1, 2).unwrap();

        let actual: Vec<_> = matrix
            .iterate_cells(key, |cell| cell.down)
            .map(|h| h.index)
            .collect();

        assert_eq!(
            actual,
            [1, 3, 4, 0]
                .into_iter()
                .map(|v| matrix.locate_cell(v, 2).unwrap())
                .collect::<Vec<_>>()
        );
    }

    fn build_matrix() -> DancingLinksMatrix {
        MatrixBuilder
            .add_primary_column(1)
            .add_primary_column(2)
            .add_primary_column(3)
            .end_columns()
            .add_sorted_row(&[1, 2])
            .add_sorted_row(&[1, 3])
            .add_sorted_row(&[2, 3])
            .add_sorted_row(&[1, 2, 3])
            .build()
    }
}
