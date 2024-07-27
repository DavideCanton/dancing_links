//! The module contains the implementation of the cell and headers.

use std::{
    fmt::{self, Display},
    ptr::null_mut,
};

/// The row of the cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CellRow {
    /// The row is the header row.
    Header,
    /// The row is a data row.
    Data(usize),
}

impl From<usize> for CellRow {
    /// Converts a `usize` into a `CellRow`.
    fn from(name: usize) -> Self {
        match name {
            0 => CellRow::Header,
            name => CellRow::Data(name),
        }
    }
}

impl From<CellRow> for usize {
    /// Converts a `CellRow` into a `usize`.
    fn from(row: CellRow) -> Self {
        match row {
            CellRow::Header => panic!("Cannot convert Header to usize"),
            CellRow::Data(name) => name,
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

/// The cell of the matrix.
#[derive(Debug)]
pub(crate) struct MatrixCell<T> {
    /// The index of the cell.
    pub(crate) index: usize,
    /// Pointer to the cell above the current cell.
    pub(crate) up: *mut MatrixCell<T>,
    /// Pointer to the cell below the current cell.
    pub(crate) down: *mut MatrixCell<T>,
    /// Pointer to the cell to the left of the current cell.
    pub(crate) left: *mut MatrixCell<T>,
    /// Pointer to the cell to the right of the current cell.
    pub(crate) right: *mut MatrixCell<T>,
    /// Pointer to the header.
    pub(crate) header: *mut Header<T>,
    /// The row of the cell.
    pub(crate) row: CellRow,
}

impl<T> MatrixCell<T> {
    /// Creates a new cell.
    ///
    /// All its links are set to `null_mut()`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the cell.
    /// * `header` - A pointer to the header.
    /// * `row` - The row of the cell.
    pub fn new(index: usize, header: *mut Header<T>, row: CellRow) -> MatrixCell<T> {
        MatrixCell {
            index,
            up: null_mut(),
            down: null_mut(),
            left: null_mut(),
            right: null_mut(),
            header,
            row,
        }
    }

    /// Updates the pointers of the cell.
    ///
    /// Uses the provided `base` as the base address, and the indexes of the cells above, below, left and right
    /// as offsets from `base`.
    ///
    /// This implies that the cells are stored contiguously in memory.
    ///
    /// # Arguments
    ///
    /// * `base` - A pointer to use as the base when adding the offsets.
    /// * `up` - The index of the cell above the current cell.
    /// * `down` - The index of the cell below the current cell.
    /// * `left` - The index of the cell to the left of the current cell.
    /// * `right` - The index of the cell to the right of the current cell.
    pub(crate) fn update_pointers(
        &mut self,
        base: *mut MatrixCell<T>,
        up: usize,
        down: usize,
        left: usize,
        right: usize,
    ) where
        T: Eq,
    {
        unsafe {
            let addr = base.add(up);
            self.up = addr;

            let addr = base.add(down);
            self.down = addr;

            let addr = base.add(left);
            self.left = addr;

            let addr = base.add(right);
            self.right = addr;
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
pub(crate) struct Header<T> {
    /// The index of the header.
    pub(crate) index: usize,
    /// The name of the header.
    pub(crate) name: HeaderName<T>,
    /// The size of the header.
    pub(crate) size: usize,
    /// Pointer to the cell.
    pub(crate) cell: *mut MatrixCell<T>,
}

impl<T> Header<T> {
    /// Creates a new header.
    ///
    /// The `cell` field is left to null.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the header.
    /// * `index` - The index of the header.
    /// * `size` - The size of the header.
    pub fn new(name: HeaderName<T>, index: usize, size: usize) -> Header<T> {
        Header {
            index,
            name,
            size,
            cell: null_mut(),
        }
    }

    /// Creates a new header from a `ProtoHeader`.
    ///
    /// The `cell` field is left to null.
    ///
    /// # Arguments
    ///
    /// * `proto` - The prototype of the header.
    pub fn from_proto(proto: ProtoHeader<T>) -> Header<T> {
        Self::new(proto.name, proto.index, proto.size)
    }

    pub fn update_pointer(&mut self, cell: *mut MatrixCell<T>) {
        self.cell = cell
    }

    /// Checks if the header is the first header.
    #[allow(dead_code)]
    pub fn is_first(&self) -> bool {
        matches!(self.name, HeaderName::First)
    }
}
