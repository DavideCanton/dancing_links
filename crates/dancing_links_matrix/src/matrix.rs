use std::{
    collections::{HashMap, HashSet},
    fmt,
    hash::Hash,
    iter, ptr,
};

use itertools::Itertools;
use rand::{Rng, thread_rng};

use crate::{
    cells::{CellRow, ColumnRef, MatrixCell, MatrixCellRef},
    queue::ColumnPriorityQueue,
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

pub struct DancingLinksMatrix<'a, T> {
    pub(crate) row_count: usize,
    pub(crate) column_count: usize,
    pub(crate) columns: Box<[ColumnRef<'a, T>]>,
    pub(crate) cells: Box<[MatrixCellRef<'a, T>]>,
    pub(crate) columns_queue: ColumnPriorityQueue<'a, T>,
}

impl<'a, T> DancingLinksMatrix<'a, T> {
    pub(crate) fn first_column(&self) -> ColumnRef<'a, T> {
        self.columns[0]
    }

    pub fn iter_rows<Ret>(&'a self) -> impl Iterator<Item = HashSet<&'a Ret>>
    where
        T: AsRef<Ret>,
        Ret: ?Sized + Eq + Hash + 'a,
    {
        self.cells
            .chunk_by(|a, b| a.row == b.row)
            .filter_map(|x| match x.first().unwrap().row {
                CellRow::Header => None,
                CellRow::Data(_) => Some(HashSet::from_iter(
                    x.iter()
                        .map(|c| c.column().name.get_name().unwrap().as_ref()),
                )),
            })
    }

    pub(crate) fn min_column(&self) -> ColumnRef<'a, T> {
        self.columns_queue.peek().unwrap()
    }

    pub(crate) fn random_column(&self) -> ColumnRef<'a, T> {
        let num = thread_rng().gen_range(0..self.column_count);

        let start = self.first_column();

        let mut iter = self.iterate_columns(start, ColumnIteratorDir::Right, false);

        iter.nth(num).unwrap()
    }

    pub(crate) fn cover(&self, column: ColumnRef<'a, T>) {
        let pq = &self.columns_queue;

        let hc = column.cell();
        hc.skip_horizontal();

        pq.remove(column);

        for i in self.iterate_cells(hc, CellIteratorDir::Down, false) {
            for j in self.iterate_cells(i, CellIteratorDir::Right, false) {
                j.skip_vertical();
                j.column().decrease_size();
                pq.change_priority(j.column());
            }
        }
    }

    pub(crate) fn uncover(&self, column: ColumnRef<'a, T>) {
        let hc = column.cell();

        let pq = &self.columns_queue;

        for i in self.iterate_cells(hc, CellIteratorDir::Up, false) {
            for j in self.iterate_cells(i, CellIteratorDir::Left, false) {
                j.restore_vertical();
                j.column().increase_size();
                pq.change_priority(j.column());
            }
        }

        hc.restore_horizontal();

        if column.primary {
            pq.push(column);
        }
    }

    pub(crate) fn iterate_cells(
        &self,
        start: MatrixCellRef<'a, T>,
        direction: CellIteratorDir,
        mut include_start: bool,
    ) -> impl Iterator<Item = MatrixCellRef<'a, T>> + use<'a, T> {
        use CellIteratorDir::*;

        let mut end = false;
        let mut current = start;

        let get_next = match direction {
            Up => MatrixCell::up,
            Down => MatrixCell::down,
            Left => MatrixCell::left,
            Right => MatrixCell::right,
        };

        iter::from_fn(move || {
            if end {
                return None;
            }

            if include_start && ptr::eq(current, start) {
                include_start = false;
                return Some(current);
            }

            current = get_next(current);

            if ptr::eq(current, start) {
                end = true;
                return None;
            }

            Some(current)
        })
    }

    pub(crate) fn iterate_columns(
        &self,
        start: ColumnRef<'a, T>,
        direction: ColumnIteratorDir,
        mut include_start: bool,
    ) -> impl Iterator<Item = ColumnRef<'a, T>> + use<'a, T> {
        use ColumnIteratorDir::*;

        let mut end = false;
        let mut current = start;

        let get_next = match direction {
            Right => MatrixCell::right,
            Left => MatrixCell::left,
        };

        iter::from_fn(move || {
            if end {
                return None;
            }

            if include_start && ptr::eq(current, start) {
                include_start = false;
                return Some(current);
            }

            let next_col_cell = get_next(current.cell());

            current = next_col_cell.column();

            if ptr::eq(current, start) {
                end = true;
                return None;
            }

            Some(current)
        })
    }
}

impl<T: fmt::Debug> fmt::Debug for DancingLinksMatrix<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut matrix = String::new();

        let mut is_first = true;

        for column in &self.columns {
            if is_first {
                is_first = false;
            } else {
                matrix.push(' ');
            }

            matrix.push_str(&format!("{:>4?}", column.name));
        }

        matrix.push('\n');

        is_first = true;

        matrix.push_str(&format!(
            "{:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
            "K", "U", "D", "L", "R", "H", "ROW"
        ));
        matrix.push('\n');

        for cell in &self.cells {
            if is_first {
                is_first = false;
            } else {
                matrix.push('\n');
            }

            matrix.push_str(&format!(
                "{:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
                cell.index,
                cell.up().index,
                cell.down().index,
                cell.left().index,
                cell.right().index,
                cell.column().index,
                cell.row
            ));
        }
        write!(f, "{}", matrix)
    }
}

impl<'a, T: fmt::Display> fmt::Display for &'a DancingLinksMatrix<'a, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rows = vec![" ".repeat(self.columns.len() * 5); self.row_count + 1];
        let mut inds = HashMap::new();

        for (i, column) in self
            .iterate_columns(self.first_column(), ColumnIteratorDir::Right, true)
            .enumerate()
        {
            let ind = i * 5;
            inds.insert(column.index, ind);
            rows[0].replace_range(ind..ind + 4, &format!("{:>4}", column.name));
        }

        for column in self.iterate_columns(self.first_column(), ColumnIteratorDir::Right, true) {
            for c in self.iterate_cells(column.cell(), CellIteratorDir::Down, false) {
                let ind = inds[&c.column().index];

                let row: usize = c.row.into();
                rows[row].replace_range(ind..ind + 4, &format!("{:>4}", c.index));
            }
        }

        let result = rows.into_iter().filter(|r| !r.trim().is_empty()).join("\n");

        write!(fmt, "{}", result)
    }
}

pub(crate) enum CellIteratorDir {
    Up,
    Down,
    Left,
    Right,
}

#[allow(dead_code)]
pub(crate) enum ColumnIteratorDir {
    Right,
    Left,
}
