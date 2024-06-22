//! # Matrix Builder
//!
//! The builder is used to create a [`DancingLinksMatrix`].
//!
//! [`DancingLinksMatrix`]: crate::matrix::DancingLinksMatrix

use std::marker::PhantomData;

use itertools::Itertools;

use crate::{
    allocator::{Allocator, VecAllocator},
    cells::{CellRow, HeaderName},
    keys::HeaderKey,
    matrix::{ColumnSpec, DancingLinksMatrix},
};

/// A builder for a [`DancingLinksMatrix`].
///
/// Columns must be added to the matrix before the rows can be added.
///
/// [`DancingLinksMatrix`]: crate::matrix::DancingLinksMatrix
pub struct MatrixBuilder<T>(PhantomData<T>);

impl<T: Eq> MatrixBuilder<T> {
    /// Create a new [`MatrixBuilder`].
    ///
    /// [`MatrixBuilder`]: MatrixBuilder
    pub fn new() -> MatrixBuilder<T> {
        MatrixBuilder(PhantomData)
    }

    /// Create a new [`MatrixBuilder`] from an iterable of column specifications.
    ///
    /// Returns a [`MatrixRowBuilder`], that can be used to add rows to the matrix.
    ///
    /// [`MatrixBuilder`]: MatrixBuilder
    pub fn from_iterable<I: Into<ColumnSpec<T>>, IT: IntoIterator<Item = I>>(
        iterable: IT,
    ) -> MatrixRowBuilder<T> {
        iterable.into_iter().collect::<MatrixRowBuilder<T>>()
    }

    /// Add a column to the matrix being built.
    ///
    /// Returns a [`MatrixColBuilder`], that can be used to add more columns to the matrix.
    ///
    /// [`MatrixBuilder`]: MatrixBuilder
    pub fn add_column<I: Into<ColumnSpec<T>>>(self, spec: I) -> MatrixColBuilder<T> {
        MatrixColBuilder::new().add_column(spec)
    }
}

impl<T: Eq> Default for MatrixBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MatrixColBuilder<T> {
    columns: Vec<ColumnSpec<T>>,
}

impl<T: Eq, I: Into<ColumnSpec<T>>> FromIterator<I> for MatrixColBuilder<T> {
    fn from_iter<IT: IntoIterator<Item = I>>(iter: IT) -> Self {
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
    pub fn add_column<I: Into<ColumnSpec<T>>>(mut self, spec: I) -> MatrixColBuilder<T> {
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

        let headers = VecAllocator::with_capacity(column_names.len());
        let mut matrix = DancingLinksMatrix {
            header_key: headers.next_key().into(),
            rows: 0,
            columns: column_names.len(),
            headers,
            cells: VecAllocator::new(),
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
    matrix: DancingLinksMatrix<T>,
}

impl<T: Eq, I: Into<ColumnSpec<T>>> FromIterator<I> for MatrixRowBuilder<T> {
    fn from_iter<IT: IntoIterator<Item = I>>(iter: IT) -> Self {
        iter.into_iter()
            .collect::<MatrixColBuilder<T>>()
            .end_columns()
    }
}

impl<T: Eq> MatrixRowBuilder<T> {
    /// Add a row to the [`MatrixRowBuilder`] using key values.
    ///
    /// Keys must be of a type that is convertible into an usize, and must be in the range from 1 to `n`,
    /// where `n` is the number of columns in the matrix, in the order that the columns were added.
    ///
    /// Use `add_sorted_row_key` if the keys are already sorted, to avoid sorting them twice.
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn add_row_key<K: Into<HeaderKey> + Ord, IT: IntoIterator<Item = K>>(
        self,
        row: IT,
    ) -> Self {
        let mut sorted = row.into_iter().collect_vec();
        sorted.sort_unstable();
        self.add_sorted_row_key(sorted)
    }

    /// Add a row to the [`MatrixRowBuilder`].
    ///
    /// Use `add_sorted_row` if the values are already sorted, to avoid sorting them twice.
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn add_row<IT: IntoIterator<Item = T>>(self, row: IT) -> Self
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
    pub fn add_sorted_row_key<K: Into<HeaderKey>, IT: IntoIterator<Item = K>>(
        self,
        row: IT,
    ) -> Self {
        self._add_sorted_row(row.into_iter().map(|v| v.into()))
    }

    /// Add a sorted row to the [`MatrixRowBuilder`].
    ///
    /// [`MatrixRowBuilder`]: MatrixRowBuilder
    pub fn add_sorted_row<IT: IntoIterator<Item = T>>(self, row: IT) -> Self {
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

    fn _add_sorted_row<IT: IntoIterator<Item = HeaderKey>>(mut self, row: IT) -> Self {
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
        self.matrix
    }
}
