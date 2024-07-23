//! # Matrix Builder
//!
//! The builder is used to create a [`DancingLinksMatrix`].
//!
//! [`DancingLinksMatrix`]: crate::matrix::DancingLinksMatrix

use itertools::Itertools;

use crate::{
    cells::{CellRow, HeaderCell, HeaderName, MatrixCell},
    index::{IndexBuilder, IndexOps, VecIndexBuilder},
    keys::{HeaderKey, Key},
    matrix::{ColumnSpec, DancingLinksMatrix},
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

        let headers = VecIndexBuilder::with_capacity(column_names.len() + 1);

        let mut matrix = BuildingMatrix {
            header_key: headers.next_key(),
            headers,
            cells: VecIndexBuilder::new(),
            rows: 0,
            columns: column_names.len(),
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
    /// Add a row to the [`MatrixRowBuilder`] using key values.
    ///
    /// Keys must be of a type that is convertible into an usize, and must be in the range from 1 to `n`,
    /// where `n` is the number of columns in the matrix, in the order that the columns were added.
    ///
    /// Use `add_sorted_row_key` if the keys are already sorted, to avoid sorting them twice.
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn add_row_key(self, row: impl IntoIterator<Item = impl Into<HeaderKey> + Ord>) -> Self {
        let mut sorted = row.into_iter().collect_vec();
        sorted.sort_unstable();
        self.add_sorted_row_key(sorted)
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

    /// Add a sorted row to the [`MatrixRowBuilder`] using key values.
    ///
    /// Keys must be of a type that is convertible into an usize, and must be in the range from 1 to `n`,
    /// where `n` is the number of columns in the matrix, in the order that the columns were added.
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn add_sorted_row_key(self, row: impl IntoIterator<Item = impl Into<HeaderKey>>) -> Self {
        self._add_sorted_row(row.into_iter().map(|v| v.into()))
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
                            to_add.push(header.key);
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

    fn _add_sorted_row(mut self, row: impl IntoIterator<Item = HeaderKey>) -> Self {
        let mx = &mut self.matrix;

        let mut cell_index = None;
        let mut prev_index = None;
        let mut start_index = None;

        for header_key in row {
            // TODO check if ind is valid

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

    pub fn build(self) -> DancingLinksMatrix<T> {
        let matrix = self.matrix;

        DancingLinksMatrix {
            headers: matrix.headers.finalize(),
            cells: matrix.cells.finalize(),
            rows: matrix.rows,
            columns: matrix.columns,
            header_key: matrix.header_key,
        }
    }
}

struct BuildingMatrix<T> {
    pub(crate) header_key: HeaderKey,
    pub(crate) headers: VecIndexBuilder<HeaderCell<T>, HeaderKey>,
    pub(crate) cells: VecIndexBuilder<MatrixCell, Key>,
    pub(crate) rows: usize,
    pub(crate) columns: usize,
}

impl<T> BuildingMatrix<T> {
    fn add_cell(&mut self, header_cell_key: HeaderKey, row: CellRow) -> Key {
        let cell_key = self.cells.next_key();
        let cell = MatrixCell::new(cell_key, header_cell_key, row);
        let actual_key = self.cells.insert(cell);
        assert_eq!(actual_key, cell_key);
        actual_key
    }

    fn add_header(&mut self, name: HeaderName<T>) -> (HeaderKey, Key) {
        let header_key = self.headers.next_key();
        let header_cell_key = self.add_cell(header_key, CellRow::Header);

        let header = HeaderCell::new(name, header_key, header_cell_key);
        let actual_header_key = self.headers.insert(header);
        assert_eq!(header_key, actual_header_key);

        (actual_header_key, header_cell_key)
    }

    fn link_right(&mut self, left: Key, right: Key) {
        self.cell_mut(left).right = right;
        self.cell_mut(right).left = left;
    }

    fn link_down(&mut self, up: Key, down: Key) {
        self.cell_mut(up).down = down;
        self.cell_mut(down).up = up;
    }

    fn cell_mut(&mut self, key: Key) -> &mut MatrixCell {
        self.cells.get_mut(key)
    }

    fn header_mut(&mut self, key: HeaderKey) -> &mut HeaderCell<T> {
        self.headers.get_mut(key)
    }
}
