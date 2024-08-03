//! # Matrix Builder
//!
//! The builder is used to create a [`DancingLinksMatrix`].
//!
//! [`DancingLinksMatrix`]: crate::matrix::DancingLinksMatrix

use itertools::Itertools;

use crate::{
    arena::Arena,
    cells::{CellRow, ColumnInfo, ColumnName, MatrixCell, ProtoCell, ProtoColumn},
    matrix::{ColumnSpec, DancingLinksMatrix},
    queue::ColumnPriorityQueue,
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
    ) -> MatrixRowBuilder<T> {
        iterable.into_iter().collect::<MatrixRowBuilder<T>>()
    }

    /// Add a column to the matrix being built.
    ///
    /// Returns a [`MatrixColBuilder`], that can be used to add more columns to the matrix.
    ///
    /// [`MatrixBuilder`]: MatrixBuilder
    pub fn add_column<T>(self, spec: impl Into<ColumnSpec<T>>) -> MatrixColBuilder<T> {
        MatrixColBuilder::new().add_column(spec)
    }
}

impl Default for MatrixBuilder {
    fn default() -> Self {
        MatrixBuilder
    }
}

/// A builder for adding columns to a matrix.
///
/// Columns must be added to the matrix before the rows can be added.
pub struct MatrixColBuilder<T> {
    columns: Vec<ColumnSpec<T>>,
}

impl<T, I: Into<ColumnSpec<T>>> FromIterator<I> for MatrixColBuilder<T> {
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

impl<T> MatrixColBuilder<T> {
    /// Create a new [`MatrixColBuilder`].
    ///
    /// [`MatrixColBuilder`]: MatrixColBuilder
    fn new() -> MatrixColBuilder<T> {
        MatrixColBuilder {
            columns: Vec::new(),
        }
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

        let mut matrix = ProtoMatrix {
            row_count: 0,
            column_count: column_names.len(),
            columns: Vec::new(),
            cells: Vec::new(),
        };

        let first_col_index = matrix.add_column(ColumnName::First, true);
        let mut prev_index = first_col_index;

        for spec in column_names {
            let col_index = matrix.add_column(ColumnName::Other(spec.name), spec.primary);

            if spec.primary {
                matrix.link_horizontal(prev_index, col_index);
                prev_index = col_index;
            }
        }

        matrix.link_horizontal(prev_index, first_col_index);

        MatrixRowBuilder { matrix }
    }
}

/// A builder for adding rows to a matrix.
///
/// Columns must be added to the matrix before the rows can be added.
///
/// This is created by calling [`MatrixBuilder::from_iterable`] or [`MatrixColBuilder::end_columns`].
///
/// [`MatrixBuilder::from_iterable`]: MatrixBuilder::from_iterable
/// [`MatrixColBuilder::end_columns`]: MatrixColBuilder::end_columns
pub struct MatrixRowBuilder<T> {
    matrix: ProtoMatrix<T>,
}

impl<T, I> FromIterator<I> for MatrixRowBuilder<T>
where
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

impl<T> MatrixRowBuilder<T> {
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
    pub fn add_sorted_row(self, row: impl IntoIterator<Item = T>) -> Self
    where
        T: Eq,
    {
        let mut to_add = Vec::new();

        {
            let mut col_iter = self.matrix.columns.iter();

            for val in row {
                let mut added = false;
                for column in col_iter.by_ref() {
                    if let ColumnName::Other(name) = &column.name {
                        if *name == val {
                            to_add.push(column.index);
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

        let mut cur_index = None;
        let mut prev_index = None;
        let mut start_index = None;

        for col_index in row {
            // TODO check if ind is valid

            let new_cell_index = mx.add_cell(col_index, (mx.row_count + 1).into());
            cur_index = Some(new_cell_index);

            match prev_index {
                Some(prev_index) => {
                    mx.link_horizontal(prev_index, new_cell_index);
                }
                None => {
                    start_index = cur_index;
                }
            }

            mx.columns[col_index].size += 1;
            let last = mx.cells[col_index].up;

            mx.link_vertical(new_cell_index, col_index);
            mx.link_vertical(last, new_cell_index);

            prev_index = cur_index;
        }

        mx.link_horizontal(cur_index.unwrap(), start_index.unwrap());

        mx.row_count += 1;
        self
    }

    /// Build the [`DancingLinksMatrix`] from the columns and rows added.
    ///
    /// Receives an [`Arena`] to allocate memory for the matrix cells and columns.
    ///
    /// The matrix will have the same lifetime as the arena itself.
    ///
    /// [`DancingLinksMatrix`]: crate::matrix::DancingLinksMatrix
    /// [`Arena`]: crate::arena::Arena
    pub fn build(self, arena: &impl Arena) -> DancingLinksMatrix<'_, T> {
        let matrix = self.matrix;

        let mut columns = Vec::new();
        let mut cells = Vec::new();

        for column in matrix.columns {
            let column = arena.alloc(ColumnInfo::from_proto(column));
            columns.push(column);
        }

        for ProtoCell { index, row, .. } in matrix.cells.iter() {
            cells.push(arena.alloc(MatrixCell::new(*index, *row)));
        }

        for ProtoCell {
            index,
            up,
            down,
            left,
            right,
            column,
            ..
        } in matrix.cells
        {
            cells[index].update_pointers(
                cells[up],
                cells[down],
                cells[left],
                cells[right],
                columns[column],
            );
        }

        let columns_queue = ColumnPriorityQueue::new();

        for col in columns.iter_mut() {
            col.update_pointer(cells[col.index]);

            if let ColumnInfo {
                name: ColumnName::Other(_),
                primary: true,
                ..
            } = col
            {
                columns_queue.push(*col);
            }
        }

        DancingLinksMatrix {
            columns: columns.into_boxed_slice(),
            cells: cells.into_boxed_slice(),
            row_count: matrix.row_count,
            column_count: matrix.column_count,
            columns_queue,
        }
    }
}

/// A matrix being built.
struct ProtoMatrix<T> {
    pub(crate) row_count: usize,
    pub(crate) column_count: usize,
    pub(crate) columns: Vec<ProtoColumn<T>>,
    pub(crate) cells: Vec<ProtoCell>,
}

impl<T> ProtoMatrix<T> {
    /// Adds a cell to the matrix, returning the index of the cell.
    fn add_cell(&mut self, column: usize, row: CellRow) -> usize {
        let cell_index = self.cells.len();
        self.cells.push(ProtoCell::new(cell_index, column, row));
        cell_index
    }

    /// Adds a column to the matrix, returning the index of the column.
    fn add_column(&mut self, name: ColumnName<T>, primary: bool) -> usize {
        let column_index = self.columns.len();
        let column_cell_index = self.add_cell(column_index, CellRow::Header);

        assert_eq!(column_index, column_cell_index);

        self.columns
            .push(ProtoColumn::new(column_index, name, primary));

        column_index
    }

    /// Links two cells together, from left to right.
    fn link_horizontal(&mut self, left: usize, right: usize) {
        self.cells[left].right = right;
        self.cells[right].left = left;
    }

    /// Links two cells together, from up to down.
    fn link_vertical(&mut self, up: usize, down: usize) {
        self.cells[up].down = down;
        self.cells[down].up = up;
    }
}
