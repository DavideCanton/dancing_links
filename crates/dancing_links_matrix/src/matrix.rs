use std::{collections::HashSet, fmt, hash::Hash, iter, marker::PhantomData};

use rand::{thread_rng, Rng};

use crate::cells::{CellRow, Header, HeaderName, HeaderRef, MatrixCellRef};

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

pub struct DancingLinksMatrix<'a, T> {
    pub(crate) rows: usize,
    pub(crate) columns: usize,
    pub(crate) headers: Box<[HeaderRef<'a, T>]>,
    pub(crate) cells: Box<[MatrixCellRef<'a, T>]>,
}

impl<'a, T: Eq> DancingLinksMatrix<'a, T> {
    pub(crate) fn first_header(&'a self) -> &'a Header<T> {
        self.headers[0]
    }

    pub fn iter_rows<Ret>(&'a self) -> RowIterator<'a, T, Ret>
    where
        T: AsRef<Ret>,
        Ret: ?Sized,
    {
        RowIterator::new(self)
    }

    pub(crate) fn min_column(&'a self) -> Option<HeaderRef<'a, T>> {
        self.iterate_headers(self.first_header(), HeaderIteratorDirection::Right, false)
            .min_by_key(|h| h.size.get())
    }

    pub(crate) fn random_column(&'a self) -> Option<HeaderRef<'a, T>> {
        if self.columns == 0 {
            return None;
        }

        let num = thread_rng().gen_range(0..self.columns);

        self.iterate_headers(self.first_header(), HeaderIteratorDirection::Right, false)
            .nth(num)
    }

    pub(crate) fn cover(&'a self, header: HeaderRef<'a, T>) {
        let hc = header.cell.get().unwrap();

        hc.right.get().unwrap().left.set(hc.left.get());
        hc.left.get().unwrap().right.set(hc.right.get());

        let v = self
            .iterate_cells(hc, CellIteratorDirection::Down, false)
            .flat_map(|i| self.iterate_cells(i, CellIteratorDirection::Right, false));

        for j in v {
            j.down.get().unwrap().up.set(j.up.get());
            j.up.get().unwrap().down.set(j.down.get());

            let jh = j.header.get().unwrap();
            jh.size.set(jh.size.get() - 1);
        }
    }

    pub(crate) fn uncover(&'a self, header: HeaderRef<'a, T>) {
        let hc = header.cell.get().unwrap();
        let v = self
            .iterate_cells(hc, CellIteratorDirection::Up, false)
            .flat_map(|j| self.iterate_cells(j, CellIteratorDirection::Left, false));

        for j in v {
            j.down.get().unwrap().up.set(Some(j));
            j.up.get().unwrap().down.set(Some(j));

            let jh = j.header.get().unwrap();
            jh.size.set(jh.size.get() + 1);
        }

        hc.right.get().unwrap().left.set(Some(hc));
        hc.left.get().unwrap().right.set(Some(hc));
    }

    pub(crate) fn iterate_cells(
        &self,
        start: MatrixCellRef<'a, T>,
        direction: CellIteratorDirection,
        mut include_start: bool,
    ) -> impl Iterator<Item = MatrixCellRef<'a, T>> {
        let mut end = false;
        let mut current = start;

        iter::from_fn(move || {
            if end {
                return None;
            }

            if include_start && current.index == start.index {
                include_start = false;
                return Some(current);
            }

            let cell = current;

            current = match direction {
                CellIteratorDirection::Up => cell.up.get().unwrap(),
                CellIteratorDirection::Down => cell.down.get().unwrap(),
                CellIteratorDirection::Left => cell.left.get().unwrap(),
                CellIteratorDirection::Right => cell.right.get().unwrap(),
            };

            if current.index == start.index {
                end = true;
                return None;
            }

            Some(current)
        })
    }

    pub(crate) fn iterate_headers(
        &self,
        start: HeaderRef<'a, T>,
        direction: HeaderIteratorDirection,
        mut include_start: bool,
    ) -> impl Iterator<Item = HeaderRef<'a, T>> {
        let mut end = false;
        let mut current = start;

        iter::from_fn(move || {
            if end {
                return None;
            }

            if include_start && current.index == start.index {
                include_start = false;
                return Some(current);
            }

            let current_header = current;
            let cell = current_header.cell.get().unwrap();

            let next_header_cell = match &direction {
                HeaderIteratorDirection::Right => cell.right.get().unwrap(),
                HeaderIteratorDirection::Left => cell.left.get().unwrap(),
            };

            current = next_header_cell.header.get().unwrap();

            if current.index == start.index {
                end = true;
                return None;
            }

            Some(current)
        })
    }

    #[cfg(test)]
    pub(crate) fn locate_cell<R: Into<CellRow>, C: Eq + ?Sized>(
        &'a self,
        row: R,
        column: &C,
    ) -> Option<MatrixCellRef<'a, T>>
    where
        T: AsRef<C>,
    {
        let header = self.locate_header(column)?;
        let row = row.into();

        self.iterate_cells(
            header.cell.get().unwrap(),
            CellIteratorDirection::Down,
            true,
        )
        .find(|c| c.row == row)
    }

    #[cfg(test)]
    pub(crate) fn locate_header<C: Eq + ?Sized>(&'a self, column: &C) -> Option<HeaderRef<'a, T>>
    where
        T: AsRef<C>,
    {
        self.iterate_headers(self.first_header(), HeaderIteratorDirection::Right, true)
            .find(|h| match h.name {
                HeaderName::Other(ref c) => *c.as_ref() == *column,
                _ => false,
            })
    }
}

impl<'a, T: fmt::Debug + Eq> fmt::Debug for DancingLinksMatrix<'a, T> {
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
                cell.index,
                cell.up.get().unwrap().index,
                cell.down.get().unwrap().index,
                cell.left.get().unwrap().index,
                cell.right.get().unwrap().index,
                cell.header.get().unwrap().index,
                cell.row
            ));
        }
        write!(f, "{}", matrix)
    }
}

#[cfg(any())]
impl<'a, T: fmt::Display + Eq> fmt::Display for DancingLinksMatrix<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rows = vec![" ".repeat(self.headers.len() * 5); self.rows + 1];
        let mut inds = HashMap::new();

        for (i, header) in self
            .iterate_headers(self.first_header(), HeaderIteratorDirection::Right, true)
            .enumerate()
        {
            let ind = i * 5;
            inds.insert((*header).index, ind);
            rows[0].replace_range(ind..ind + 4, &format!("{:>4}", (*header).name));
        }

        for header in
            self.iterate_headers(self.first_header(), HeaderIteratorDirection::Right, true)
        {
            for c in self.iterate_cells(
                header.cell.get().unwrap(),
                CellIteratorDirection::Down,
                false,
            ) {
                let header = c.header.get().unwrap();
                let ind = inds[&header.index];

                let row: usize = (*c).row.into();
                rows[row].replace_range(ind..ind + 4, &format!("{:>4}", (*c).index));
            }
        }

        write!(
            f,
            "{}",
            rows.into_iter().filter(|r| !r.trim().is_empty()).join("\n")
        )
    }
}

pub struct RowIterator<'a, T, Ret: ?Sized> {
    matrix: &'a DancingLinksMatrix<'a, T>,
    last: usize,
    _p: PhantomData<Box<Ret>>,
}

impl<'a, T, Ret> RowIterator<'a, T, Ret>
where
    Ret: ?Sized,
{
    fn new(matrix: &'a DancingLinksMatrix<'a, T>) -> RowIterator<'a, T, Ret> {
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
            let c = self.matrix.cells[i];

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

                match &c.header.get().unwrap().name {
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

#[allow(dead_code)]
pub(crate) enum HeaderIteratorDirection {
    Right,
    Left,
}
