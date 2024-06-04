#![allow(dead_code)]

use matrix::{ColumnSpec, DancingLinksMatrix};

mod cells;
mod matrix;

fn main() {
    let names = ["a", "b", "c", "d", "e"];
    let specs: Vec<_> = names.into_iter().map(ColumnSpec::from).collect();
    let mut matrix = DancingLinksMatrix::new(&specs);

    matrix.add_sparse_row(&[1, 2], true);
    matrix.add_sparse_row(&[1, 3], true);
    matrix.add_sparse_row(&[2, 4], true);

    println!("{matrix:?}");
}
