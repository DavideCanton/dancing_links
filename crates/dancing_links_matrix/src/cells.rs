//! The module contains the implementation of the cell and header cells.

use std::fmt::{self, Display};

use crate::keys::{HeaderKey, Key};

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

impl Display for CellRow {
    /// Formats the `CellRow` as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellRow::Header => "H".fmt(f),
            CellRow::Data(name) => name.fmt(f),
        }
    }
}

/// The cell of the matrix.
#[derive(Debug)]
pub(crate) struct Cell {
    /// The index of the cell.
    pub(crate) index: Key,
    /// The index of the cell above the current cell.
    pub(crate) up: Key,
    /// The index of the cell below the current cell.
    pub(crate) down: Key,
    /// The index of the cell to the left of the current cell.
    pub(crate) left: Key,
    /// The index of the cell to the right of the current cell.
    pub(crate) right: Key,
    /// The index of the header cell.
    pub(crate) header: HeaderKey,
    /// The row of the cell.
    pub(crate) row: CellRow,
}

impl Cell {
    /// Creates a new cell.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the cell.
    /// * `header` - The index of the header cell.
    /// * `row` - The row of the cell.
    pub fn new(index: Key, header: HeaderKey, row: CellRow) -> Cell {
        Cell {
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

/// The name of the header cell.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) enum HeaderName<T> {
    /// The header cell is the first header cell.
    First,
    /// The header cell is a regular header cell.
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

/// The header cell of the matrix.
///
/// Contains the key of a physical cell linked to the header cell.
#[derive(Debug)]
pub(crate) struct HeaderCell<T> {
    /// The index of the header cell.
    pub(crate) index: HeaderKey,
    /// The name of the header cell.
    pub(crate) name: HeaderName<T>,
    /// The size of the header cell.
    pub(crate) size: usize,
    /// The key of the cell.
    pub(crate) cell: Key,
}

impl<T> HeaderCell<T> {
    /// Creates a new header cell.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the header cell.
    /// * `index` - The index of the header cell.
    /// * `cell_key` - The key of the linked cell.
    pub fn new(name: HeaderName<T>, index: HeaderKey, cell_key: Key) -> HeaderCell<T> {
        HeaderCell {
            index,
            name,
            size: 0,
            cell: cell_key,
        }
    }

    /// Checks if the header cell is the first header cell.
    #[allow(dead_code)]
    pub fn is_first(&self) -> bool {
        matches!(self.name, HeaderName::First)
    }
}
