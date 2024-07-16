use std::{
    collections::{HashMap, HashSet},
    fmt,
    hash::Hash,
    marker::PhantomData,
};

use itertools::Itertools;
use rand::{thread_rng, Rng};

use crate::{
    allocator::{Allocator, VecAllocator},
    cells::{Cell, CellRow, HeaderCell, HeaderName},
    keys::{HeaderKey, Key},
};

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
    pub(crate) headers: VecAllocator<HeaderCell<T>, HeaderKey>,
    pub(crate) cells: VecAllocator<Cell, Key>,
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

    pub(crate) fn random_column(&self) -> Option<&HeaderCell<T>> {
        if self.columns == 0 {
            return None;
        }

        let num = thread_rng().gen_range(0..self.columns);

        self.iterate_headers(self.header_key, |h| h.right, false)
            .nth(num)
    }

    pub(crate) fn cover(&mut self, key: HeaderKey) {
        let header_cell_index = self.header(key).cell;
        let header_cell = self.cell(header_cell_index);

        let header_r_index = header_cell.right;
        let header_l_index = header_cell.left;

        let header_r = self.cell_mut(header_r_index);
        header_r.left = header_l_index;

        let header_l = self.cell_mut(header_l_index);
        header_l.right = header_r_index;

        let v = self
            .iterate_cells(header_cell_index, |c| c.down, false)
            .flat_map(|i| self.iterate_cells(i.key, |c| c.right, false))
            .map(|j| j.key)
            .collect_vec();

        for j in v.iter() {
            let cell = self.cell(*j);

            let cell_d_index = cell.down;
            let cell_u_index = cell.up;
            let cell_header = cell.header;

            self.cell_mut(cell_d_index).up = cell_u_index;
            self.cell_mut(cell_u_index).down = cell_d_index;
            self.header_mut(cell_header).size -= 1;
        }
    }

    pub(crate) fn uncover(&mut self, key: HeaderKey) {
        let header_cell_index = self.header(key).cell;

        let v = self
            .iterate_cells(header_cell_index, |c| c.up, false)
            .flat_map(|j| self.iterate_cells(j.key, |c| c.left, false))
            .map(|j| j.key)
            .collect_vec();
        for j in v.iter() {
            let cell = self.cell(*j);

            let cell_d_index = cell.down;
            let cell_u_index = cell.up;
            let cell_index = cell.key;
            let cell_header = cell.header;

            self.cell_mut(cell_d_index).up = cell_index;
            self.cell_mut(cell_u_index).down = cell_index;
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
    ) -> impl Iterator<Item = &Cell> {
        CellIterator::new(self, start, getter, include_start)
    }

    pub(crate) fn iterate_headers<F: Fn(&Cell) -> Key>(
        &self,
        start: HeaderKey,
        getter: F,
        include_start: bool,
    ) -> impl Iterator<Item = &HeaderCell<T>> {
        HeaderCellIterator::new(self, start, getter, include_start)
    }

    #[allow(dead_code)]
    pub(crate) fn locate_cell<R: Into<CellRow>, C: Eq + ?Sized>(
        &self,
        row: R,
        column: &C,
    ) -> Option<Key>
    where
        T: AsRef<C>,
    {
        let header_key = self.locate_header(column)?;
        let header = self.header(header_key);
        let row = row.into();

        self.iterate_cells(header.cell, |c| c.down, true)
            .find(|c| c.row == row)
            .map(|c| c.key)
    }

    #[allow(dead_code)]
    pub(crate) fn locate_header<C: Eq + ?Sized>(&self, column: &C) -> Option<HeaderKey>
    where
        T: AsRef<C>,
    {
        self.iterate_headers(self.header_key, |c| c.right, true)
            .find(|h| match h.name {
                HeaderName::First => false,
                HeaderName::Other(ref c) => *c.as_ref() == *column,
            })
            .map(|h| h.key)
    }

    pub(crate) fn cell(&self, key: Key) -> &Cell {
        &self.cells[key]
    }

    pub(crate) fn cell_mut(&mut self, key: Key) -> &mut Cell {
        &mut self.cells[key]
    }

    pub(crate) fn header_mut(&mut self, key: HeaderKey) -> &mut HeaderCell<T> {
        &mut self.headers[key]
    }

    pub(crate) fn header(&self, key: HeaderKey) -> &HeaderCell<T> {
        &self.headers[key]
    }
}

impl<T: fmt::Debug + Eq> fmt::Debug for DancingLinksMatrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut matrix = String::new();

        let mut is_first = true;

        for header in self.headers.iter() {
            if is_first {
                is_first = false;
            } else {
                matrix.push(' ');
            }

            matrix.push_str(&format!("{:>4?}", header.name));
        }

        matrix.push('\n');

        is_first = true;

        matrix.push_str(&format!(
            "{:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
            "K", "U", "D", "L", "R", "H", "ROW"
        ));
        matrix.push('\n');

        for cell in self.cells.iter() {
            if is_first {
                is_first = false;
            } else {
                matrix.push('\n');
            }

            matrix.push_str(&format!(
                "{:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
                cell.key, cell.up, cell.down, cell.left, cell.right, cell.header, cell.row
            ));
        }
        write!(f, "{}", matrix)
    }
}

impl<T: fmt::Display + Eq> fmt::Display for DancingLinksMatrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rows = vec![" ".repeat(self.headers.len() * 5); self.rows + 1];
        let mut inds = HashMap::new();

        for (i, header) in self
            .iterate_headers(self.header_key, |c| c.right, true)
            .enumerate()
        {
            let ind = i * 5;
            inds.insert(header.key, ind);
            rows[0].replace_range(ind..ind + 4, &format!("{:>4}", header.name));
        }

        for header in self.iterate_headers(self.header_key, |c| c.right, true) {
            for c in self.iterate_cells(header.cell, |c| c.down, false) {
                let header = c.header;
                let ind = inds[&header];

                let row: usize = c.row.into();
                rows[row].replace_range(ind..ind + 4, &format!("{:>4}", c.key));
            }
        }

        write!(
            f,
            "{}",
            rows.into_iter().filter(|r| !r.trim().is_empty()).join("\n")
        )
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

pub(crate) struct HeaderCellIterator<'a, F, T> {
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
