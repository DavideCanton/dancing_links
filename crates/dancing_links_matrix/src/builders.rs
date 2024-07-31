//! # Matrix Builder
//!
//! The builder is used to create a [`DancingLinksMatrix`].
//!
//! [`DancingLinksMatrix`]: crate::matrix::DancingLinksMatrix

use itertools::Itertools;

use crate::{
    arena::Arena,
    cells::{CellRow, Header, HeaderName, MatrixCell, ProtoCell, ProtoHeader},
    matrix::{ColumnSpec, DancingLinksMatrix},
    queue::HeaderPriorityQueue,
};

/// A builder for a [`DancingLinksMatrix`].
///
/// Columns must be added to the matrix before the rows can be added.
///
/// [`DancingLinksMatrix`]: crate::matrix::DancingLinksMatrix
pub struct MatrixBuilder;

impl MatrixBuilder {
    /// Create a new [`MatrixBuilder`] from an iterable of column specifications.
    ///
    /// Returns a [`MatrixRowBuilder`], that can be used to add rows to the matrix.
    ///
    /// [`MatrixBuilder`]: MatrixBuilder
    pub fn from_iterable<T>(
        iterable: impl IntoIterator<Item = impl Into<ColumnSpec<T>>>,
    ) -> MatrixRowBuilder<T>
    where
        T: Eq,
    {
        iterable.into_iter().collect::<MatrixRowBuilder<T>>()
    }

    /// Add a column to the matrix being built.
    ///
    /// Returns a [`MatrixColBuilder`], that can be used to add more columns to the matrix.
    ///
    /// [`MatrixBuilder`]: MatrixBuilder
    pub fn add_column<T>(self, spec: impl Into<ColumnSpec<T>>) -> MatrixColBuilder<T>
    where
        T: Eq,
    {
        MatrixColBuilder::new().add_column(spec)
    }
}

impl Default for MatrixBuilder {
    fn default() -> Self {
        MatrixBuilder
    }
}

pub struct MatrixColBuilder<T> {
    columns: Vec<ColumnSpec<T>>,
}

impl<T: Eq, I: Into<ColumnSpec<T>>> FromIterator<I> for MatrixColBuilder<T> {
    fn from_iter<IT>(iter: IT) -> Self
    where
        IT: IntoIterator<Item = I>,
    {
        let iter = iter.into_iter();

        let mut builder = MatrixColBuilder::new();
        for col in iter {
            builder = builder.add_column(col);
        }

        builder
    }
}

impl<T: Eq> MatrixColBuilder<T> {
    fn new() -> MatrixColBuilder<T> {
        MatrixColBuilder { columns: vec![] }
    }

    /// Add a column to the matrix being built.
    ///
    /// Returns `self`, for chaining.
    ///
    /// [`MatrixColBuilder`]: MatrixColBuilder
    pub fn add_column(mut self, spec: impl Into<ColumnSpec<T>>) -> MatrixColBuilder<T> {
        self.columns.push(spec.into());
        self
    }

    /// Marks the ending of the phase of adding columns to the matrix.
    ///
    /// Returns a [`MatrixRowBuilder`], that can be used to add rows to the matrix.
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn end_columns(self) -> MatrixRowBuilder<T> {
        if self.columns.is_empty() {
            panic!("No columns were added");
        }
        let column_names = self.columns;

        let mut matrix = BuildingMatrix {
            rows: 0,
            columns: column_names.len(),
            headers: Vec::new(),
            cells: Vec::new(),
        };

        let first_header_index = matrix.add_header(HeaderName::First, true);
        let mut prev_index = first_header_index;

        for spec in column_names {
            let header_index = matrix.add_header(HeaderName::Other(spec.name), spec.primary);

            if spec.primary {
                matrix.link_right(prev_index, header_index);
                prev_index = header_index;
            }
        }

        matrix.link_right(prev_index, first_header_index);

        MatrixRowBuilder { matrix }
    }
}

pub struct MatrixRowBuilder<T> {
    matrix: BuildingMatrix<T>,
}

impl<T, I> FromIterator<I> for MatrixRowBuilder<T>
where
    T: Eq,
    I: Into<ColumnSpec<T>>,
{
    fn from_iter<IT>(iter: IT) -> Self
    where
        IT: IntoIterator<Item = I>,
    {
        iter.into_iter()
            .collect::<MatrixColBuilder<T>>()
            .end_columns()
    }
}

impl<T> MatrixRowBuilder<T>
where
    T: Eq,
{
    /// Add a row to the [`MatrixRowBuilder`] using indexes.
    ///
    /// Indexes must be in the range from 1 to `n` where `n` is the number of columns in the matrix, in the order that the columns were added.
    ///
    /// Use `add_sorted_row_index` if the indexes are already sorted, to avoid sorting them twice.
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn add_row_index(self, row: impl IntoIterator<Item = usize>) -> Self {
        let mut sorted = row.into_iter().collect_vec();
        sorted.sort_unstable();
        self.add_sorted_row_index(sorted)
    }

    /// Add a row to the [`MatrixRowBuilder`].
    ///
    /// Use `add_sorted_row` if the values are already sorted, to avoid sorting them twice.
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn add_row(self, row: impl IntoIterator<Item = T>) -> Self
    where
        T: Ord,
    {
        let mut sorted = row.into_iter().collect_vec();
        sorted.sort_unstable();
        self.add_sorted_row(sorted)
    }

    /// Add a sorted row to the [`MatrixRowBuilder`] using index values.
    ///
    /// Indexes must be in the range from 1 to `n`, where `n` is the number of columns in the matrix, in the order that the columns were added.
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn add_sorted_row_index(self, row: impl IntoIterator<Item = usize>) -> Self {
        self._add_sorted_row(row)
    }

    /// Add a sorted row to the [`MatrixRowBuilder`].
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn add_sorted_row(self, row: impl IntoIterator<Item = T>) -> Self {
        let mut to_add = Vec::new();

        {
            let mut headers_iter = self.matrix.headers.iter();

            for val in row {
                let mut added = false;
                for header in headers_iter.by_ref() {
                    if let HeaderName::Other(name) = &header.name {
                        if *name == val {
                            to_add.push(header.index);
                            added = true;
                            break;
                        }
                    }
                }

                if !added {
                    // TODO improve
                    panic!("Column not found");
                }
            }
        }

        self._add_sorted_row(to_add)
    }

    fn _add_sorted_row(mut self, row: impl IntoIterator<Item = usize>) -> Self {
        let mx = &mut self.matrix;

        let mut cell_index = None;
        let mut prev_index = None;
        let mut start_index = None;

        for header_idx in row {
            // TODO check if ind is valid

            let in_cell_index = mx.add_cell(header_idx, (mx.rows + 1).into());
            cell_index = Some(in_cell_index);

            match prev_index {
                Some(prev_index) => {
                    mx.link_right(prev_index, in_cell_index);
                }
                None => {
                    start_index = cell_index;
                }
            }

            mx.headers[header_idx].size += 1;
            let last = mx.cells[header_idx].up;

            mx.link_down(in_cell_index, header_idx);
            mx.link_down(last, in_cell_index);

            prev_index = cell_index;
        }

        mx.link_right(cell_index.unwrap(), start_index.unwrap());

        mx.rows += 1;
        self
    }

    pub fn build(self, arena: &impl Arena) -> DancingLinksMatrix<'_, T> {
        let matrix = self.matrix;

        let mut headers = Vec::new();
        let mut cells = Vec::new();

        for header in matrix.headers {
            let header = arena.alloc(Header::from_proto(header));
            headers.push(header);
        }

        for cell in matrix.cells.iter() {
            let cell = arena.alloc(MatrixCell::new(cell.index, cell.row));
            cells.push(cell);
        }

        for pc in matrix.cells {
            cells[pc.index].update_pointers(
                cells[pc.up],
                cells[pc.down],
                cells[pc.left],
                cells[pc.right],
                headers[pc.header],
            );
        }

        let headers_queue = HeaderPriorityQueue::new();

        for h in headers.iter_mut() {
            h.update_pointer(cells[h.index]);
            if h.index > 0 && h.primary {
                headers_queue.push(*h);
            }
        }

        DancingLinksMatrix {
            headers: headers.into_boxed_slice(),
            cells: cells.into_boxed_slice(),
            rows: matrix.rows,
            columns: matrix.columns,
            headers_queue,
        }
    }
}

struct BuildingMatrix<T> {
    pub(crate) rows: usize,
    pub(crate) columns: usize,
    pub(crate) headers: Vec<ProtoHeader<T>>,
    pub(crate) cells: Vec<ProtoCell>,
}

impl<T> BuildingMatrix<T> {
    fn add_cell(&mut self, header_index: usize, row: CellRow) -> usize {
        let cell_index = self.cells.len();
        self.cells
            .push(ProtoCell::new(cell_index, header_index, row));
        cell_index
    }

    fn add_header(&mut self, name: HeaderName<T>, primary: bool) -> usize {
        let header_index = self.headers.len();
        let header_cell_index = self.add_cell(header_index, CellRow::Header);

        assert_eq!(header_index, header_cell_index);

        self.headers
            .push(ProtoHeader::new(header_index, name, 0, primary));

        header_index
    }

    fn link_right(&mut self, left: usize, right: usize) {
        self.cells[left].right = right;
        self.cells[right].left = left;
    }

    fn link_down(&mut self, up: usize, down: usize) {
        self.cells[up].down = down;
        self.cells[down].up = up;
    }
}
