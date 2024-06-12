#![allow(dead_code)]

use matrix::MatrixBuilder;

mod cells;
mod keys;
mod matrix;

fn main() {
    let mut builder = MatrixBuilder.add_primary_column(1);
    for name in 2..=5 {
        builder = builder.add_primary_column(name);
    }

    let matrix = builder
        .end_columns()
        .add_sorted_row(&[1, 2])
        .add_sorted_row(&[1, 3])
        .add_sorted_row(&[2, 4])
        .add_sorted_row(&[1, 2, 3, 4])
        .build();

    println!("{matrix}");

    println!("{:?}", matrix.iter_rows().collect::<Vec<_>>());
}
