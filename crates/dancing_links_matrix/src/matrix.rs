use std::{
    collections::{HashMap, HashSet},
    fmt,
    hash::Hash,
    marker::PhantomData,
};

use itertools::Itertools;
use rand::{thread_rng, Rng};

use crate::{
    cells::{CellRow, HeaderCell, HeaderName, MatrixCell},
    index::{Index, IndexOps, VecIndex},
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
    pub(crate) header_key: *mut HeaderCell<T>,
    pub(crate) rows: usize,
    pub(crate) columns: usize,
    pub(crate) headers: VecIndex<HeaderCell<T>>,
    pub(crate) cells: VecIndex<MatrixCell<T>>,
}

impl<T: Eq> DancingLinksMatrix<T> {
    pub fn iter_rows<Ret>(&self) -> RowIterator<T, Ret>
    where
        T: AsRef<Ret>,
        Ret: ?Sized,
    {
        RowIterator::new(self)
    }

    pub(crate) fn min_column(&self) -> Option<*mut HeaderCell<T>> {
        self.iterate_headers(self.header_key, HeaderIteratorDirection::Right, false)
            .min_by_key(|h| unsafe { (*(*h)).size })
    }

    pub(crate) fn random_column(&self) -> Option<*mut HeaderCell<T>> {
        if self.columns == 0 {
            return None;
        }

        let num = thread_rng().gen_range(0..self.columns);

        self.iterate_headers(self.header_key, HeaderIteratorDirection::Right, false)
            .nth(num)
    }

    pub(crate) fn cover(&mut self, header: *mut HeaderCell<T>) {
        unsafe {
            let hr = (*(*header).cell).right;
            let hl = (*(*header).cell).left;

            (*hr).left = hl;
            (*hl).right = hr;

            let v = self
                .iterate_cells((*header).cell, CellIteratorDirection::Down, false)
                .flat_map(|i| self.iterate_cells(i, CellIteratorDirection::Right, false));

            for j in v {
                let jd = (*j).down;
                let ju = (*j).up;

                (*jd).up = ju;
                (*ju).down = jd;
                (*(*j).header).size -= 1;
            }
        }
    }

    pub(crate) fn uncover(&mut self, header: *const HeaderCell<T>) {
        unsafe {
            let v = self
                .iterate_cells((*header).cell, CellIteratorDirection::Up, false)
                .flat_map(|j| self.iterate_cells(j, CellIteratorDirection::Left, false));

            for j in v {
                let jd = (*j).down;
                let ju = (*j).up;

                (*jd).up = j;
                (*ju).down = j;
                (*(*j).header).size += 1;
            }

            let hc = (*header).cell;
            (*(*hc).right).left = hc;
            (*(*hc).left).right = hc;
        }
    }

    pub(crate) fn iterate_cells(
        &self,
        start: *mut MatrixCell<T>,
        direction: CellIteratorDirection,
        include_start: bool,
    ) -> impl Iterator<Item = *mut MatrixCell<T>> {
        CellIterator::new(start, direction, include_start)
    }

    pub(crate) fn iterate_headers(
        &self,
        start: *mut HeaderCell<T>,
        direction: HeaderIteratorDirection,
        include_start: bool,
    ) -> impl Iterator<Item = *mut HeaderCell<T>> {
        HeaderCellIterator::new(start, direction, include_start)
    }

    #[cfg(test)]
    pub(crate) fn locate_cell<R: Into<CellRow>, C: Eq + ?Sized>(
        &self,
        row: R,
        column: &C,
    ) -> Option<*mut MatrixCell<T>>
    where
        T: AsRef<C>,
    {
        let header = self.locate_header(column)?;
        let row = row.into();

        unsafe {
            self.iterate_cells((*header).cell, CellIteratorDirection::Down, true)
                .find(|c| (*(*c)).row == row)
        }
    }

    #[cfg(test)]
    pub(crate) fn locate_header<C: Eq + ?Sized>(&self, column: &C) -> Option<*mut HeaderCell<T>>
    where
        T: AsRef<C>,
    {
        unsafe {
            self.iterate_headers(self.header_key, HeaderIteratorDirection::Right, true)
                .find(|h| match (*(*h)).name {
                    HeaderName::Other(ref c) => *c.as_ref() == *column,
                    _ => false,
                })
        }
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

            unsafe {
                matrix.push_str(&format!(
                    "{:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
                    cell.key,
                    (*cell.up).key,
                    (*cell.down).key,
                    (*cell.left).key,
                    (*cell.right).key,
                    (*cell.header).key,
                    cell.row
                ));
            }
        }
        write!(f, "{}", matrix)
    }
}

impl<T: fmt::Display + Eq> fmt::Display for DancingLinksMatrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let mut rows = vec![" ".repeat(self.headers.len() * 5); self.rows + 1];
            let mut inds = HashMap::new();

            for (i, header) in self
                .iterate_headers(self.header_key, HeaderIteratorDirection::Right, true)
                .enumerate()
            {
                let ind = i * 5;
                inds.insert((*header).key, ind);
                rows[0].replace_range(ind..ind + 4, &format!("{:>4}", (*header).name));
            }

            for header in
                self.iterate_headers(self.header_key, HeaderIteratorDirection::Right, true)
            {
                for c in self.iterate_cells((*header).cell, CellIteratorDirection::Down, false) {
                    let header = (*c).header;
                    let ind = inds[&(*header).key];

                    let row: usize = (*c).row.into();
                    rows[row].replace_range(ind..ind + 4, &format!("{:>4}", (*c).key));
                }
            }

            write!(
                f,
                "{}",
                rows.into_iter().filter(|r| !r.trim().is_empty()).join("\n")
            )
        }
    }
}

pub struct RowIterator<'a, T, Ret: ?Sized> {
    matrix: &'a DancingLinksMatrix<T>,
    last: usize,
    _p: PhantomData<Box<Ret>>,
}

impl<'a, T, Ret> RowIterator<'a, T, Ret>
where
    Ret: ?Sized,
{
    fn new(matrix: &'a DancingLinksMatrix<T>) -> RowIterator<'a, T, Ret> {
        RowIterator {
            matrix,
            last: 0,
            _p: PhantomData,
        }
    }
}

impl<'a, T, Ret> Iterator for RowIterator<'a, T, Ret>
where
    T: AsRef<Ret> + Eq,
    Ret: 'a + Eq + Hash + ?Sized,
{
    type Item = HashSet<&'a Ret>;

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
            let c = self.matrix.cells.get(i);

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

                match unsafe { &(*c.header).name } {
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

impl<'a, T, Ret> ExactSizeIterator for RowIterator<'a, T, Ret>
where
    T: Eq + AsRef<Ret>,
    Ret: 'a + Eq + Hash + ?Sized,
{
    fn len(&self) -> usize {
        self.matrix.rows
    }
}

pub(crate) enum CellIteratorDirection {
    Up,
    Down,
    Left,
    Right,
}

pub(crate) struct CellIterator<T> {
    start: *mut MatrixCell<T>,
    current: *mut MatrixCell<T>,
    direction: CellIteratorDirection,
    end: bool,
    include_start: bool,
}

impl<T> CellIterator<T> {
    fn new(
        start: *mut MatrixCell<T>,
        direction: CellIteratorDirection,
        include_start: bool,
    ) -> CellIterator<T> {
        CellIterator {
            start,
            current: start,
            direction,
            end: false,
            include_start,
        }
    }
}

impl<T> Iterator for CellIterator<T>
where
    T: Eq,
{
    type Item = *mut MatrixCell<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        if self.include_start && self.current == self.start {
            self.include_start = false;
            return Some(self.current);
        }

        let cell = self.current;

        self.current = unsafe {
            match self.direction {
                CellIteratorDirection::Up => (*cell).up,
                CellIteratorDirection::Down => (*cell).down,
                CellIteratorDirection::Left => (*cell).left,
                CellIteratorDirection::Right => (*cell).right,
            }
        };

        if self.current == self.start {
            self.end = true;
            return None;
        }
        Some(self.current)
    }
}

#[allow(dead_code)]
pub(crate) enum HeaderIteratorDirection {
    Right,
    Left,
}

pub(crate) struct HeaderCellIterator<T> {
    start: *mut HeaderCell<T>,
    current: *mut HeaderCell<T>,
    direction: HeaderIteratorDirection,
    end: bool,
    include_start: bool,
}

impl<T> HeaderCellIterator<T> {
    fn new(
        start: *mut HeaderCell<T>,
        direction: HeaderIteratorDirection,
        include_start: bool,
    ) -> HeaderCellIterator<T> {
        HeaderCellIterator {
            start,
            current: start,
            direction,
            end: false,
            include_start,
        }
    }
}

impl<T> Iterator for HeaderCellIterator<T>
where
    T: Eq,
{
    type Item = *mut HeaderCell<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        if self.include_start && self.current == self.start {
            self.include_start = false;
            return Some(self.current);
        }

        unsafe {
            let current_header = self.current;
            let cell = (*current_header).cell;

            let next_header_cell = match &self.direction {
                HeaderIteratorDirection::Right => (*cell).right,
                HeaderIteratorDirection::Left => (*cell).left,
            };

            self.current = (*next_header_cell).header;
        }

        if self.current == self.start {
            self.end = true;
            return None;
        }

        Some(self.current)
    }
}
