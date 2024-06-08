#![allow(dead_code)]

use matrix::MatrixBuilder;

mod cells;
mod matrix;

fn main() {
    let mut builder = MatrixBuilder::new();
    for name in 1..=5 {
        builder = builder.add_primary_column(name);
    }

    let mut matrix = builder.build();

    matrix.add_sparse_row(&[1, 2], true);
    matrix.add_sparse_row(&[1, 3], true);
    matrix.add_sparse_row(&[2, 4], true);
    matrix.add_sparse_row(&[1, 2, 3, 4], true);

    println!("{matrix}");

    println!("{:?}", matrix.iter_rows().collect::<Vec<_>>());
}
