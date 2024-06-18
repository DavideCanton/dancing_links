use std::marker::PhantomData;

use itertools::Itertools;
use slab::Slab;

use crate::{
    cells::{CellRow, HeaderName},
    keys::HeaderKey,
    matrix::{ColumnSpec, DancingLinksMatrix},
};

pub struct MatrixBuilder<T>(PhantomData<T>);

impl<T: Eq> MatrixBuilder<T> {
    pub fn new() -> MatrixBuilder<T> {
        MatrixBuilder(PhantomData)
    }

    pub fn from_iterable<I: Into<ColumnSpec<T>>, IT: IntoIterator<Item = I>>(
        iterable: IT,
    ) -> MatrixRowBuilder<T> {
        iterable.into_iter().collect::<MatrixRowBuilder<T>>()
    }

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

    pub fn add_column<I: Into<ColumnSpec<T>>>(mut self, spec: I) -> MatrixColBuilder<T> {
        self.columns.push(spec.into());
        self
    }

    pub fn end_columns(self) -> MatrixRowBuilder<T> {
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

impl<T: Eq + Ord + Clone> MatrixRowBuilder<T> {
    pub fn add_row_key<IT: IntoIterator<Item = usize>>(self, row: IT) -> Self {
        let mut sorted = row.into_iter().collect_vec();
        sorted.sort_unstable();
        self.add_sorted_row_key(sorted)
    }

    pub fn add_row<IT: IntoIterator<Item = T>>(self, row: IT) -> Self {
        let mut sorted = row.into_iter().collect_vec();
        sorted.sort_unstable();
        self.add_sorted_row(sorted)
    }
}

impl<T: Eq> MatrixRowBuilder<T> {
    pub fn add_sorted_row_key<IT: IntoIterator<Item = usize>>(self, row: IT) -> Self {
        self.add_sorted_row_fn(row, |_mx, v| v.into())
    }

    pub fn add_sorted_row<IT: IntoIterator<Item = T>>(self, row: IT) -> Self {
        self.add_sorted_row_fn(row, |mx, v| {
            mx.headers
                .iter()
                .map(|h| h.1)
                .find(|h| match h.name {
                    HeaderName::First => false,
                    HeaderName::Other(ref c) => *c == v,
                })
                .unwrap()
                .index
        })
    }

    fn add_sorted_row_fn<
        U,
        F: Fn(&mut DancingLinksMatrix<T>, U) -> HeaderKey,
        IT: IntoIterator<Item = U>,
    >(
        mut self,
        row: IT,
        fun: F,
    ) -> Self {
        let mx = &mut self.matrix;

        let mut cell_index = None;
        let mut prev_index = None;
        let mut start_index = None;

        for ind in row {
            // TODO check if ind is valid
            let header_key = fun(mx, ind);

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
