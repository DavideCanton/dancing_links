use slab::Slab;
use std::hash::Hash;
use std::marker::PhantomData;
use std::{collections::HashSet, fmt};

use crate::cells::{Cell, CellRow, HeaderCell, HeaderName};
use crate::keys::{HeaderKey, Key};

pub struct ColumnSpec<T> {
    pub(crate) name: T,
    pub(crate) primary: bool,
}

impl<T> ColumnSpec<T> {
    pub fn primary(name: T) -> ColumnSpec<T> {
        ColumnSpec {
            name,
            primary: true,
        }
    }

    pub fn secondary(name: T) -> ColumnSpec<T> {
        ColumnSpec {
            name,
            primary: false,
        }
    }
}

impl<T> From<T> for ColumnSpec<T> {
    fn from(name: T) -> Self {
        Self::primary(name)
    }
}

pub struct DancingLinksMatrix<T> {
    pub(crate) header_key: HeaderKey,
    pub(crate) rows: usize,
    pub(crate) columns: usize,
    pub(crate) headers: Slab<HeaderCell<T>>,
    pub(crate) cells: Slab<Cell>,
}

impl<T: Eq> DancingLinksMatrix<T> {
    pub fn iter_rows<U: ?Sized>(&self) -> RowIterator<T, U>
    where
        T: AsRef<U>,
    {
        RowIterator::new(self)
    }

    pub(crate) fn min_column(&self) -> Option<&HeaderCell<T>> {
        self.iterate_headers(self.header_key, |h| h.right, false)
            .min_by_key(|h| h.size)
    }

    #[cfg(feature = "random")]
    pub(crate) fn random_column(&self) -> Option<&HeaderCell<T>> {
        use rand::{thread_rng, Rng};

        if self.columns == 0 {
            return None;
        }

        let num = thread_rng().gen_range(0..self.columns);

        self.iterate_headers(self.header_key, |h| h.right, false)
            .nth(num)
    }

    pub fn cover(&mut self, key: HeaderKey) {
        let header_cell_index = self.header(key).cell;
        let header_cell = self.cell(header_cell_index);

        let header_r_index = header_cell.right;
        let header_l_index = header_cell.left;

        let header_r = self.cell_mut(header_r_index);
        header_r.left = header_l_index;

        let header_l = self.cell_mut(header_l_index);
        header_l.right = header_r_index;

        let mut v = Vec::new();

        for i in self.iterate_cells(header_cell_index, |c| c.down, false) {
            for j in self.iterate_cells(i.index, |c| c.right, false) {
                v.push(j.index)
            }
        }

        for j in v {
            let cell = self.cell(j);

            let cell_d_index = cell.down;
            let cell_u_index = cell.up;
            let cell_header = cell.header;

            let cell_d = self.cell_mut(cell_d_index);
            cell_d.up = cell_u_index;

            let cell_u = self.cell_mut(cell_u_index);
            cell_u.down = cell_d_index;

            self.header_mut(cell_header).size -= 1;
        }
    }

    pub fn uncover(&mut self, key: HeaderKey) {
        let header_cell_index = self.header(key).cell;

        let mut v = Vec::new();

        for i in self.iterate_cells(header_cell_index, |c| c.up, false) {
            for j in self.iterate_cells(i.index, |c| c.left, false) {
                v.push(j.index)
            }
        }

        for j in v {
            let cell = self.cell(j);

            let cell_d_index = cell.down;
            let cell_u_index = cell.up;
            let cell_index = cell.index;
            let cell_header = cell.header;

            let cell_d = self.cell_mut(cell_d_index);
            cell_d.up = cell_index;

            let cell_u = self.cell_mut(cell_u_index);
            cell_u.down = cell_index;

            self.header_mut(cell_header).size += 1;
        }

        let header_cell = self.cell(header_cell_index);

        let cell_r_index = header_cell.right;
        let cell_l_index = header_cell.left;

        let cell_r = self.cell_mut(cell_r_index);
        cell_r.left = header_cell_index;

        let cell_l = self.cell_mut(cell_l_index);
        cell_l.right = header_cell_index;
    }

    pub(crate) fn iterate_cells<F: Fn(&Cell) -> Key>(
        &self,
        start: Key,
        getter: F,
        include_start: bool,
    ) -> CellIterator<F, T> {
        CellIterator::new(self, start, getter, include_start)
    }

    fn iterate_headers<F: Fn(&Cell) -> Key>(
        &self,
        start: HeaderKey,
        getter: F,
        include_start: bool,
    ) -> HeaderCellIterator<F, T> {
        HeaderCellIterator::new(self, start, getter, include_start)
    }

    pub(crate) fn add_cell(&mut self, header_cell_key: HeaderKey, row: CellRow) -> Key {
        let cell_key = self.cells.vacant_key().into();
        let cell = Cell::new(cell_key, header_cell_key, row);
        let actual_key = self.cells.insert(cell).into();
        assert_eq!(actual_key, cell_key);
        actual_key
    }

    pub(crate) fn add_header(&mut self, name: HeaderName<T>) -> (HeaderKey, Key) {
        let header_key = self.headers.vacant_key().into();
        let header_cell_key = self.add_cell(header_key, CellRow::Header);

        let header = HeaderCell::new(name, header_key, header_cell_key);
        let actual_header_key = self.headers.insert(header).into();
        assert_eq!(header_key, actual_header_key);

        (actual_header_key, header_cell_key)
    }

    fn locate_cell<R: Into<CellRow>, C: Eq + ?Sized>(&self, row: R, column: &C) -> Option<Key>
    where
        T: AsRef<C>,
    {
        let header_key = self.locate_header(column)?;
        let header = self.header(header_key);
        let row = row.into();

        self.iterate_cells(header.cell, |c| c.down, true)
            .find(|c| c.row == row)
            .map(|c| c.index)
    }

    fn locate_header<C: Eq + ?Sized>(&self, column: &C) -> Option<HeaderKey>
    where
        T: AsRef<C>,
    {
        self.iterate_headers(self.header_key, |c| c.right, true)
            .find(|h| match h.name {
                HeaderName::First => false,
                HeaderName::Other(ref c) => *c.as_ref() == *column,
            })
            .map(|h| h.index)
    }

    pub(crate) fn cell(&self, key: Key) -> &Cell {
        &self.cells[key.into()]
    }

    pub(crate) fn cell_mut(&mut self, key: Key) -> &mut Cell {
        &mut self.cells[key.into()]
    }

    pub(crate) fn header_mut(&mut self, key: HeaderKey) -> &mut HeaderCell<T> {
        &mut self.headers[key.into()]
    }

    pub(crate) fn header(&self, key: HeaderKey) -> &HeaderCell<T> {
        &self.headers[key.into()]
    }

    pub(crate) fn link_right(&mut self, left: Key, right: Key) {
        self.cell_mut(left).right = right;
        self.cell_mut(right).left = left;
    }

    pub(crate) fn link_down(&mut self, up: Key, down: Key) {
        self.cell_mut(up).down = down;
        self.cell_mut(down).up = up;
    }
}

impl<T: fmt::Display> fmt::Display for DancingLinksMatrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut matrix = String::new();

        let mut is_first = true;

        for (_, header) in self.headers.iter() {
            if is_first {
                is_first = false;
            } else {
                matrix.push(' ');
            }

            matrix.push_str(&format!("{:>4}", header.name));
        }

        matrix.push('\n');

        is_first = true;

        matrix.push_str(&format!(
            "{:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
            "K", "U", "D", "L", "R", "H", "ROW"
        ));
        matrix.push('\n');

        for (k, cell) in self.cells.iter() {
            if is_first {
                is_first = false;
            } else {
                matrix.push('\n');
            }

            matrix.push_str(&format!(
                "{:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
                k, cell.up, cell.down, cell.left, cell.right, cell.header, cell.row
            ));
        }
        write!(f, "{}", matrix)
    }
}

pub struct RowIterator<'a, T, U: ?Sized> {
    matrix: &'a DancingLinksMatrix<T>,
    last: usize,
    _p: PhantomData<Box<U>>,
}

impl<'a, T, U: ?Sized> RowIterator<'a, T, U> {
    fn new(matrix: &'a DancingLinksMatrix<T>) -> RowIterator<'a, T, U> {
        RowIterator {
            matrix,
            last: 0,
            _p: PhantomData,
        }
    }
}

impl<'a, T: Eq, U: 'a + Eq + Hash + ?Sized> Iterator for RowIterator<'a, T, U>
where
    T: AsRef<U>,
{
    type Item = HashSet<&'a U>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let r = self.matrix.rows;
        (r, Some(r))
    }

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

                match &self.matrix.header(c.header).name {
                    HeaderName::First => panic!("A cell should not have the first header as name"),
                    HeaderName::Other(name) => {
                        set.insert(name.as_ref());
                    }
                }
            }

            i += 1;
        }

        self.last = i;

        data_found.then_some(set)
    }
}

impl<'a, T: Eq + AsRef<U>, U: 'a + Eq + Hash + ?Sized> ExactSizeIterator for RowIterator<'a, T, U> {
    fn len(&self) -> usize {
        self.matrix.rows
    }
}

pub(crate) struct CellIterator<'a, F, T> {
    matrix: &'a DancingLinksMatrix<T>,
    start: Key,
    current: Key,
    getter: F,
    end: bool,
    include_start: bool,
}

impl<'a, F, T> CellIterator<'a, F, T>
where
    F: Fn(&Cell) -> Key,
{
    fn new(
        matrix: &'a DancingLinksMatrix<T>,
        start: Key,
        getter: F,
        include_start: bool,
    ) -> CellIterator<'a, F, T> {
        CellIterator {
            matrix,
            start,
            current: start,
            getter,
            end: false,
            include_start,
        }
    }
}

impl<'a, F, T: Eq> Iterator for CellIterator<'a, F, T>
where
    F: Fn(&Cell) -> Key,
{
    type Item = &'a Cell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        if self.include_start && self.current == self.start {
            self.include_start = false;
            return Some(self.matrix.cell(self.current));
        }

        let cell = self.matrix.cell(self.current);
        self.current = (self.getter)(cell);
        if self.current == self.start {
            self.end = true;
            return None;
        }
        Some(self.matrix.cell(self.current))
    }
}

struct HeaderCellIterator<'a, F, T> {
    matrix: &'a DancingLinksMatrix<T>,
    start: HeaderKey,
    current: HeaderKey,
    getter: F,
    end: bool,
    include_start: bool,
}

impl<'a, F, T> HeaderCellIterator<'a, F, T>
where
    F: Fn(&Cell) -> Key,
{
    fn new(
        matrix: &'a DancingLinksMatrix<T>,
        start: HeaderKey,
        getter: F,
        include_start: bool,
    ) -> HeaderCellIterator<'a, F, T> {
        HeaderCellIterator {
            matrix,
            start,
            current: start,
            getter,
            end: false,
            include_start,
        }
    }
}

impl<'a, F, T: Eq> Iterator for HeaderCellIterator<'a, F, T>
where
    F: Fn(&Cell) -> Key,
{
    type Item = &'a HeaderCell<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        if self.include_start && self.current == self.start {
            self.include_start = false;
            return Some(self.matrix.header(self.current));
        }

        let current_header = self.matrix.header(self.current);
        let current_header_cell = self.matrix.cell(current_header.cell);

        let next_header_cell_key = (self.getter)(current_header_cell);
        let next_header_key = self.matrix.cell(next_header_cell_key).header;

        self.current = next_header_key;

        if self.current == self.start {
            self.end = true;
            return None;
        }

        Some(self.matrix.header(self.current))
    }
}

#[cfg(test)]
mod tests {
    use crate::builders::MatrixBuilder;

    use super::*;
    use itertools::Itertools;
    use test_case::test_matrix;
    use HeaderName::First as F;
    use HeaderName::Other as O;

    #[test]
    fn test_locate_cell() {
        let matrix = build_matrix();
        assert_eq!(matrix.locate_cell(1, "1").unwrap(), 4.into());
        assert_eq!(matrix.locate_cell(1, "2").unwrap(), 5.into());
        assert_eq!(matrix.locate_cell(1, "3"), None);
        assert_eq!(matrix.locate_cell(2, "1").unwrap(), 6.into());
        assert_eq!(matrix.locate_cell(2, "2"), None);
        assert_eq!(matrix.locate_cell(2, "3").unwrap(), 7.into());
    }

    #[test]
    fn test_locate_header() {
        let matrix = build_matrix();
        assert_eq!(matrix.locate_header("1").unwrap(), 1.into());
        assert_eq!(matrix.locate_header("2").unwrap(), 2.into());
        assert_eq!(matrix.locate_header("6"), None);
    }

    #[test]
    fn test_iterator() {
        let matrix = build_matrix();

        let mut it = matrix.iter_rows::<str>();
        assert_eq!(it.len(), 4);

        fn n<'a>(it: &'a mut RowIterator<String, str>) -> Vec<&'a str> {
            it.next().unwrap().into_iter().sorted().collect_vec()
        }

        assert_eq!(n(&mut it), ["1", "2"]);
        assert_eq!(n(&mut it), ["1", "3"]);
        assert_eq!(n(&mut it), ["2", "3"]);
        assert_eq!(n(&mut it), ["1", "2", "3"]);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_no_rows() {
        let matrix = MatrixBuilder::new()
            .add_column("1")
            .add_column("2")
            .add_column("3")
            .end_columns()
            .build();

        let mut it = matrix.iter_rows::<str>();
        assert_eq!(it.len(), 0);
        assert_eq!(it.next(), None);
    }

    #[test_matrix([true, false]; "include_start")]
    fn test_header_cell_iterator_right_from_first(include_start: bool) {
        let matrix = build_matrix();

        let actual: Vec<HeaderName<_>> = matrix
            .iterate_headers(matrix.header_key, |cell| cell.right, include_start)
            .map(|h| h.name.clone())
            .collect();

        let mut exp = vec![O("1".to_string()), O("2".to_string()), O("3".to_string())];
        if include_start {
            exp.insert(0, F);
        }

        assert_eq!(actual, exp);
    }

    #[test_matrix([true, false]; "include_start")]
    fn test_header_cell_iterator_right(include_start: bool) {
        let matrix = build_matrix();
        let key = matrix.locate_header("1").unwrap();

        let actual: Vec<HeaderName<_>> = matrix
            .iterate_headers(key, |cell| cell.right, include_start)
            .map(|h| h.name.clone())
            .collect();

        let mut exp = vec![O("2".to_string()), O("3".to_string()), F];
        if include_start {
            exp.insert(0, O("1".to_string()));
        }

        assert_eq!(actual, exp);
    }

    #[test_matrix([true, false]; "include_start")]
    fn test_header_cell_iterator_left(include_start: bool) {
        let matrix = build_matrix();
        let key = matrix.locate_header("1").unwrap();

        let actual: Vec<HeaderName<_>> = matrix
            .iterate_headers(key, |cell| cell.left, include_start)
            .map(|h| h.name.clone())
            .collect();
        let mut exp = vec![F, O("3".to_string()), O("2".to_string())];
        if include_start {
            exp.insert(0, O("1".to_string()));
        }

        assert_eq!(actual, exp);
    }

    #[test_matrix([true, false]; "include_start")]
    fn test_header_cell_iterator_up(include_start: bool) {
        let matrix = build_matrix();
        let key = matrix.locate_header("1").unwrap();

        let actual: Vec<HeaderName<_>> = matrix
            .iterate_headers(key, |c| c.up, include_start)
            .map(|h| h.name.clone())
            .collect();

        let mut exp = vec![];
        if include_start {
            exp.insert(0, O("1".to_string()));
        }

        assert_eq!(actual, exp);
    }

    #[test_matrix([true, false]; "include_start")]
    fn test_header_cell_iterator_down(include_start: bool) {
        let matrix = build_matrix();
        let key = matrix.locate_header("1").unwrap();

        let actual: Vec<HeaderName<_>> = matrix
            .iterate_headers(key, |c| c.down, include_start)
            .map(|h| h.name.clone())
            .collect();

        let mut exp = vec![];
        if include_start {
            exp.insert(0, O("1".to_string()));
        }

        assert_eq!(actual, exp);
    }

    #[test_matrix([true, false]; "include_start")]
    fn test_cell_iterator_left(include_start: bool) {
        let matrix = build_matrix();

        let key = matrix.locate_cell(4, "2").unwrap();

        let actual: Vec<_> = matrix
            .iterate_cells(key, |cell| cell.left, include_start)
            .map(|h| h.index)
            .collect();

        let mut exp = vec!["1", "3"];

        if include_start {
            exp.insert(0, "2");
        }

        assert_eq!(
            actual,
            exp.into_iter()
                .map(|v| matrix.locate_cell(4, v).unwrap())
                .collect::<Vec<_>>()
        );
    }

    #[test_matrix([true, false]; "include_start")]
    fn test_cell_iterator_right(include_start: bool) {
        let matrix = build_matrix();

        let key = matrix.locate_cell(4, "2").unwrap();

        let actual: Vec<_> = matrix
            .iterate_cells(key, |cell| cell.right, include_start)
            .map(|h| h.index)
            .collect();

        let mut exp = vec!["3", "1"];

        if include_start {
            exp.insert(0, "2");
        }

        assert_eq!(
            actual,
            exp.into_iter()
                .map(|v| matrix.locate_cell(4, v).unwrap())
                .collect::<Vec<_>>()
        );
    }

    #[test_matrix([true, false]; "include_start")]
    fn test_cell_iterator_up(include_start: bool) {
        let matrix = build_matrix();

        let key = matrix.locate_cell(2, "1").unwrap();

        let actual: Vec<_> = matrix
            .iterate_cells(key, |cell| cell.up, include_start)
            .map(|h| h.index)
            .collect();

        let mut exp = vec![1, 0, 4];

        if include_start {
            exp.insert(0, 2);
        }

        assert_eq!(
            actual,
            exp.into_iter()
                .map(|v| matrix.locate_cell(v, "1").unwrap())
                .collect::<Vec<_>>()
        );
    }

    #[test_matrix([true, false]; "include_start")]
    fn test_cell_iterator_down(include_start: bool) {
        let matrix = build_matrix();

        let key = matrix.locate_cell(1, "2").unwrap();

        let actual: Vec<_> = matrix
            .iterate_cells(key, |cell| cell.down, include_start)
            .map(|h| h.index)
            .collect();

        let mut exp = vec![3, 4, 0];

        if include_start {
            exp.insert(0, 1);
        }

        assert_eq!(
            actual,
            exp.into_iter()
                .map(|v| matrix.locate_cell(v, "2").unwrap())
                .collect::<Vec<_>>()
        );
    }

    fn build_matrix() -> DancingLinksMatrix<String> {
        fn r<const N: usize>(v: [&str; N]) -> Vec<String> {
            v.iter().map(|v| v.to_string()).collect()
        }

        MatrixBuilder::new()
            .add_column(1.to_string())
            .add_column(2.to_string())
            .add_column(3.to_string())
            .end_columns()
            .add_sorted_row(r(["1", "2"]))
            .add_sorted_row(r(["1", "3"]))
            .add_sorted_row(r(["2", "3"]))
            .add_sorted_row(r(["1", "2", "3"]))
            .build()
    }
}
