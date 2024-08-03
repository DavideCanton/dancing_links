//! The module contains the implementation of the cell and columns.

use concat_idents::concat_idents;
use std::{
    cell::Cell,
    fmt::{self, Display},
    hash::{Hash, Hasher},
    num::NonZero,
};

/// The row of the cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CellRow {
    /// The row is the column row.
    Header,
    /// The row is a data row.
    Data(NonZero<usize>),
}

impl From<usize> for CellRow {
    /// Converts a `usize` into a `CellRow`.
    fn from(index: usize) -> Self {
        match index {
            0 => CellRow::Header,
            index => CellRow::Data(unsafe { NonZero::new_unchecked(index) }),
        }
    }
}

impl From<CellRow> for usize {
    /// Converts a `CellRow` into a `usize`.
    fn from(row: CellRow) -> Self {
        match row {
            CellRow::Header => panic!("Cannot convert Header to usize"),
            CellRow::Data(name) => name.into(),
        }
    }
}

impl Display for CellRow {
    /// Formats the `CellRow` as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellRow::Header => "H".fmt(f),
            CellRow::Data(name) => name.fmt(f),
        }
    }
}

/// Struct containing a cell prototype. It is constructed while building the matrix.
///
/// The cell prototype is then converted to a `MatrixCell` by converting the `cell`
/// index to a reference.
#[derive(Debug)]
pub(crate) struct ProtoCell {
    pub(crate) index: usize,
    pub(crate) up: usize,
    pub(crate) down: usize,
    pub(crate) left: usize,
    pub(crate) right: usize,
    pub(crate) column: usize,
    pub(crate) row: CellRow,
}

impl ProtoCell {
    /// Creates a new cell prototype.
    pub(crate) fn new(index: usize, column: usize, row: CellRow) -> Self {
        ProtoCell {
            index,
            up: index,
            down: index,
            left: index,
            right: index,
            column,
            row,
        }
    }
}

/// A reference to a cell in the matrix.
pub type MatrixCellRef<'a, T> = &'a MatrixCell<'a, T>;

/// The type of a pointer from a cell to another.
///
/// It is a `Cell` containing an `Option` of a reference to a cell.
///
/// It uses `Cell` to ensure interior mutability.
pub type MatrixCellPtr<'a, T> = Cell<Option<MatrixCellRef<'a, T>>>;

/// A reference to a column in the matrix.
pub type ColumnRef<'a, T> = &'a ColumnInfo<'a, T>;

/// The type of a pointer from a column to a cell.
///
/// It is a `Cell` containing an `Option` of a reference to a column.
///
/// It uses `Cell` to ensure interior mutability.
pub type ColumnPtr<'a, T> = Cell<Option<ColumnRef<'a, T>>>;

/// The cell of the matrix.
#[derive(Debug)]
pub(crate) struct MatrixCell<'a, T> {
    /// The index of the cell.
    pub(crate) index: usize,
    /// Pointer to the cell above the current cell.
    up: MatrixCellPtr<'a, T>,
    /// Pointer to the cell below the current cell.
    down: MatrixCellPtr<'a, T>,
    /// Pointer to the cell to the left of the current cell.
    left: MatrixCellPtr<'a, T>,
    /// Pointer to the cell to the right of the current cell.
    right: MatrixCellPtr<'a, T>,
    /// Pointer to the column.
    column: ColumnPtr<'a, T>,
    /// The row of the cell.
    pub(crate) row: CellRow,
}

macro_rules! impl_field {
    ($name: ident, $type: ty) => {
        #[inline(always)]
        pub fn $name(&self) -> $type {
            unsafe { self.$name.get().unwrap_unchecked() }
        }

        concat_idents!(fn_name = has_, $name {
            #[inline(always)]
            #[allow(dead_code)]
            pub fn fn_name(&self) -> bool {
                self.$name.get().is_some()
            }
        });
    };
}

impl<'a, T> MatrixCell<'a, T> {
    /// Creates a new cell.
    ///
    /// All its links are set to a cell pointing to `None`.
    pub(crate) fn new(index: usize, row: CellRow) -> MatrixCell<'a, T> {
        MatrixCell {
            index,
            up: Cell::new(None),
            down: Cell::new(None),
            left: Cell::new(None),
            right: Cell::new(None),
            column: Cell::new(None),
            row,
        }
    }

    /// Updates the pointers of the cell.
    pub(crate) fn update_pointers(
        &'a self,
        up: MatrixCellRef<'a, T>,
        down: MatrixCellRef<'a, T>,
        left: MatrixCellRef<'a, T>,
        right: MatrixCellRef<'a, T>,
        column: ColumnRef<'a, T>,
    ) {
        self.up.set(Some(up));
        self.down.set(Some(down));
        self.left.set(Some(left));
        self.right.set(Some(right));
        self.column.set(Some(column));
    }

    impl_field!(up, MatrixCellRef<'a, T>);
    impl_field!(down, MatrixCellRef<'a, T>);
    impl_field!(left, MatrixCellRef<'a, T>);
    impl_field!(right, MatrixCellRef<'a, T>);
    impl_field!(column, ColumnRef<'a, T>);

    /// Returns the name of the column.
    #[inline(always)]
    pub fn name(&self) -> &ColumnName<T> {
        &self.column().name
    }

    /// Skips the cell to the left and right.
    ///
    /// It logically removes the cell from the row.
    ///
    /// If the cell is `X`, its left is `L` and its right is `R`,
    /// this method sets L.right to R and R.left to L.
    ///
    /// This means that effectively `X` is not considered in the row while iterating,
    /// but it is still linked to the other cells, so the link can be restored at
    /// a later time.
    pub fn skip_horizontal(&self) {
        self.right().left.set(self.left.get());
        self.left().right.set(self.right.get());
    }

    /// Skips the cell to the up and down.
    ///
    /// It logically removes the cell from the column.
    ///
    /// If the cell is `X`, its up is `U` and its down is `D`,
    /// this method sets U.down to D and D.up to U.
    ///
    /// This means that effectively `X` is not considered in the column while iterating,
    /// but it is still linked to the other cells, so the link can be restored at
    /// a later time.
    pub fn skip_vertical(&self) {
        self.down().up.set(self.up.get());
        self.up().down.set(self.down.get());
    }

    /// Restores the cell in the row.
    ///
    /// It restores the cell in the row after it has been skipped.
    ///
    /// If the cell is `X`, its left is `L` and its right is `R`,
    /// this method sets L.right to X and R.left to X.
    ///
    /// This means that effectively `X` is restored back into the row.
    pub fn restore_horizontal(&'a self) {
        self.right().left.set(Some(self));
        self.left().right.set(Some(self));
    }

    /// Restores the cell in the column.
    ///
    /// It restores the cell in the column after it has been skipped.
    ///
    /// If the cell is `X`, its up is `U` and its down is `D`,
    /// this method sets U.down to X and D.up to X.
    ///
    /// This means that effectively `X` is restored back into the column.
    pub fn restore_vertical(&'a self) {
        self.down().up.set(Some(self));
        self.up().down.set(Some(self));
    }
}

/// The name of the column.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) enum ColumnName<T> {
    /// The column is the first column.
    First,
    /// The column is a regular column.
    Other(T),
}

impl<T> ColumnName<T> {
    pub fn get_name(&self) -> Option<&T> {
        match self {
            ColumnName::Other(name) => Some(name),
            ColumnName::First => None,
        }
    }
}

impl<T: Display> Display for ColumnName<T> {
    /// Formats the `ColumnName` as a string.
    ///
    /// If the column is the first column, it is formatted as `<H>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_name() {
            Some(name) => name.fmt(f),
            None => "<H>".fmt(f),
        }
    }
}

/// Struct containing a column prototype. It is constructed while building the matrix.
///
/// The column prototype is then converted to a `ColumnInfo`.
#[derive(Debug, Clone)]
pub(crate) struct ProtoColumn<T> {
    /// The index of the column.
    pub(crate) index: usize,
    /// The name of the column.
    pub(crate) name: ColumnName<T>,
    /// The size of the column.
    pub(crate) size: usize,
    /// If the column is a primary column.
    pub(crate) primary: bool,
}

impl<T> ProtoColumn<T> {
    /// Creates a new column prototype.
    pub(crate) fn new(index: usize, name: ColumnName<T>, primary: bool) -> Self {
        Self {
            index,
            name,
            size: 0,
            primary,
        }
    }
}

/// A column of the matrix.
///
/// Contains a pointer to a physical cell linked to the column.
#[derive(Debug)]
pub(crate) struct ColumnInfo<'a, T> {
    /// The index of the column.
    pub(crate) index: usize,
    /// The name of the column.
    pub(crate) name: ColumnName<T>,
    /// The size of the column (the number of 1 in the column).
    ///
    /// It is a cell since it will be mutated while solving.
    size: Cell<usize>,
    /// Pointer to the cell.
    cell: MatrixCellPtr<'a, T>,
    /// If the column is a primary column.
    pub(crate) primary: bool,
}

impl<'a, T> ColumnInfo<'a, T> {
    /// Creates a new column.
    ///
    /// The `cell` field is set to a cell containing `None`.
    pub fn new(name: ColumnName<T>, index: usize, size: usize, primary: bool) -> ColumnInfo<'a, T> {
        ColumnInfo {
            index,
            name,
            size: Cell::new(size),
            cell: Cell::new(None),
            primary,
        }
    }

    /// Creates a new column from a `ProtoColumn`.
    ///
    /// The `cell` field is set to a cell containing `None`.
    pub fn from_proto(proto: ProtoColumn<T>) -> ColumnInfo<'a, T> {
        Self::new(proto.name, proto.index, proto.size, proto.primary)
    }

    /// Updates the pointer to the cell.
    ///
    /// The pointer is updated to the cell passed as argument.
    pub fn update_pointer(&self, cell: MatrixCellRef<'a, T>) {
        self.cell.set(Some(cell));
    }

    impl_field!(cell, MatrixCellRef<'a, T>);

    /// Increases the size of the column.
    ///
    /// Used when a cell is restored into a column using one of the `restore` methods.
    pub fn increase_size(&self) -> usize {
        self.size.set(self.size.get() + 1);
        self.size.get()
    }

    /// Decreases the size of the column.
    ///
    /// Used when a cell is skipped from a column using one of the `skip` methods.
    pub fn decrease_size(&self) -> usize {
        self.size.set(self.size.get() - 1);
        self.size.get()
    }

    /// Returns the size of the column.
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.size.get()
    }

    /// Returns if the column has values.
    #[inline(always)]
    pub fn empty(&self) -> bool {
        self.size.get() == 0
    }
}

impl<'a, T> Hash for ColumnInfo<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<'a, T> PartialEq for ColumnInfo<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<'a, T> Eq for ColumnInfo<'a, T> {}
