use slab::Slab;

use crate::{cells::{CellRow, HeaderName}, keys::HeaderKey, matrix::{ColumnSpec, DancingLinksMatrix}};

pub struct MatrixBuilder;

impl MatrixBuilder {
    pub fn from_iterable<I: Into<ColumnSpec>, IT: IntoIterator<Item = I>>(
        iterable: IT,
    ) -> MatrixRowBuilder {
        iterable.into_iter().collect::<MatrixRowBuilder>()
    }

    pub fn add_column<I: Into<ColumnSpec>>(self, spec: I) -> MatrixColBuilder {
        MatrixColBuilder::new().add_column(spec)
    }
}

pub struct MatrixColBuilder {
    columns: Vec<ColumnSpec>,
}

impl<I: Into<ColumnSpec>> FromIterator<I> for MatrixColBuilder {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let mut builder = MatrixColBuilder::new();
        for col in iter {
            builder = builder.add_column(col);
        }

        builder
    }
}

impl MatrixColBuilder {
    fn new() -> MatrixColBuilder {
        MatrixColBuilder { columns: vec![] }
    }

    pub fn add_column<I: Into<ColumnSpec>>(mut self, spec: I) -> MatrixColBuilder {
        self.columns.push(spec.into());
        self
    }

    pub fn end_columns(self) -> MatrixRowBuilder {
        if self.columns.is_empty() {
            panic!("No columns were added");
        }
        let column_names = self.columns;

        let headers = Slab::with_capacity(column_names.len());
        let mut matrix = DancingLinksMatrix {
            header_key: headers.vacant_key().into(),
            rows: 0,
            columns: column_names.len(),
            headers,
            cells: Slab::with_capacity(column_names.len() * 5),
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

pub struct MatrixRowBuilder {
    matrix: DancingLinksMatrix,
}

impl<I: Into<ColumnSpec>> FromIterator<I> for MatrixRowBuilder {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        iter.into_iter().collect::<MatrixColBuilder>().end_columns()
    }
}

impl MatrixRowBuilder {
    pub fn add_row(self, row: &[usize]) -> Self {
        let mut sorted = row.to_vec();
        sorted.sort_unstable();
        self.add_sorted_row(&sorted)
    }

    pub fn add_sorted_row(mut self, row: &[usize]) -> Self {
        let mx = &mut self.matrix;

        let mut cell_index = None;
        let mut prev_index = None;
        let mut start_index = None;

        for ind in row.iter() {
            // TODO check if ind is valid
            let header_key = HeaderKey::from(*ind);

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

    pub fn build(self) -> DancingLinksMatrix {
        self.matrix
    }
}
