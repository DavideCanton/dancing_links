#![allow(dead_code)]

use matrix::{ColumnSpec, DancingLinksMatrix};

mod cells;
mod matrix;

fn main() {
    let names = 1..=5;
    let specs: Vec<_> = names.into_iter().map(ColumnSpec::from).collect();
    let mut matrix = DancingLinksMatrix::new(&specs);

    matrix.add_sparse_row(&[1, 2], true);
    matrix.add_sparse_row(&[1, 3], true);
    matrix.add_sparse_row(&[2, 4], true);
    matrix.add_sparse_row(&[1, 2, 3, 4], true);

    println!("{matrix:?}");

    println!("{:?}", matrix.iter_rows().collect::<Vec<_>>());
}
