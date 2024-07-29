//! The module contains the implementation of the cell and headers.

use std::{
    cell::Cell,
    fmt::{self, Display},
    num::NonZero,
};

/// The row of the cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CellRow {
    /// The row is the header row.
    Header,
    /// The row is a data row.
    Data(NonZero<usize>),
}

impl From<usize> for CellRow {
    /// Converts a `usize` into a `CellRow`.
    fn from(name: usize) -> Self {
        match name {
            0 => CellRow::Header,
            name => CellRow::Data(unsafe { NonZero::new_unchecked(name) }),
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
/// The cell prototype is then converted to a `MatrixCell` by converting the indexes
/// to pointers.
#[derive(Debug)]
pub(crate) struct ProtoCell {
    pub(crate) index: usize,
    pub(crate) up: usize,
    pub(crate) down: usize,
    pub(crate) left: usize,
    pub(crate) right: usize,
    pub(crate) header: usize,
    pub(crate) row: CellRow,
}

impl ProtoCell {
    /// Creates a new cell prototype.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the cell.
    /// * `header` - The index of the header.
    /// * `row` - The row of the cell.
    pub(crate) fn new(index: usize, header: usize, row: CellRow) -> Self {
        ProtoCell {
            index,
            up: index,
            down: index,
            left: index,
            right: index,
            header,
            row,
        }
    }
}

pub type MatrixCellRef<'a, T> = &'a MatrixCell<'a, T>;
pub type MatrixCellPtr<'a, T> = Cell<Option<MatrixCellRef<'a, T>>>;

pub type HeaderRef<'a, T> = &'a Header<'a, T>;
pub type HeaderPtr<'a, T> = Cell<Option<HeaderRef<'a, T>>>;

/// The cell of the matrix.
#[derive(Debug)]
pub(crate) struct MatrixCell<'a, T> {
    /// The index of the cell.
    pub(crate) index: usize,
    /// Pointer to the cell above the current cell.
    pub(crate) up: MatrixCellPtr<'a, T>,
    /// Pointer to the cell below the current cell.
    pub(crate) down: MatrixCellPtr<'a, T>,
    /// Pointer to the cell to the left of the current cell.
    pub(crate) left: MatrixCellPtr<'a, T>,
    /// Pointer to the cell to the right of the current cell.
    pub(crate) right: MatrixCellPtr<'a, T>,
    /// Pointer to the header.
    pub(crate) header: HeaderPtr<'a, T>,
    /// The row of the cell.
    pub(crate) row: CellRow,
}

impl<'a, T> MatrixCell<'a, T> {
    /// Creates a new cell.
    ///
    /// All its links are set to `null_mut()`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the cell.
    /// * `header` - A pointer to the header.
    /// * `row` - The row of the cell.
    pub fn new(index: usize, row: CellRow) -> MatrixCell<'a, T> {
        MatrixCell {
            index,
            up: Cell::new(None),
            down: Cell::new(None),
            left: Cell::new(None),
            right: Cell::new(None),
            header: Cell::new(None),
            row,
        }
    }
}

/// The name of the header.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) enum HeaderName<T> {
    /// The header is the first header.
    First,
    /// The header is a regular header.
    Other(T),
}

impl<T: Display> Display for HeaderName<T> {
    /// Formats the `HeaderName` as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeaderName::First => "<H>".fmt(f),
            HeaderName::Other(name) => name.fmt(f),
        }
    }
}

/// Struct containing a header prototype. It is constructed while building the matrix.
///
/// The header prototype is then converted to a `HeaderCell` by converting the `cell`
/// index to a pointer.
#[derive(Debug)]
pub(crate) struct ProtoHeader<T> {
    /// The index of the header.
    pub(crate) index: usize,
    /// The name of the header.
    pub(crate) name: HeaderName<T>,
    /// The size of the header.
    pub(crate) size: usize,
}

impl<T> ProtoHeader<T> {
    pub(crate) fn new(index: usize, name: HeaderName<T>, size: usize) -> Self {
        Self { index, name, size }
    }
}

/// A header of the matrix.
///
/// Contains a pointer to a physical cell linked to the header.
#[derive(Debug)]
pub(crate) struct Header<'a, T> {
    /// The index of the header.
    pub(crate) index: usize,
    /// The name of the header.
    pub(crate) name: HeaderName<T>,
    /// The size of the header.
    pub(crate) size: Cell<usize>,
    /// Pointer to the cell.
    pub(crate) cell: MatrixCellPtr<'a, T>,
}

impl<'a, T> Header<'a, T> {
    /// Creates a new header.
    ///
    /// The `cell` field is left to null.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the header.
    /// * `index` - The index of the header.
    /// * `size` - The size of the header.
    pub fn new(name: HeaderName<T>, index: usize, size: usize) -> Header<'a, T> {
        Header {
            index,
            name,
            size: Cell::new(size),
            cell: Cell::new(None),
        }
    }

    /// Creates a new header from a `ProtoHeader`.
    ///
    /// The `cell` field is left to null.
    ///
    /// # Arguments
    ///
    /// * `proto` - The prototype of the header.
    pub fn from_proto(proto: ProtoHeader<T>) -> Header<'a, T> {
        Self::new(proto.name, proto.index, proto.size)
    }

    /// Checks if the header is the first header.
    #[allow(dead_code)]
    pub fn is_first(&self) -> bool {
        matches!(self.name, HeaderName::First)
    }
}
