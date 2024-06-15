#![allow(dead_code)]

use matrix::MatrixBuilder;

mod cells;
mod keys;
mod matrix;

fn main() {
    let mut matrix = MatrixBuilder::from_iterable_end(1..=5)
        .add_sorted_row(&[1, 2])
        .add_sorted_row(&[1, 3])
        .add_sorted_row(&[2, 5])
        .add_sorted_row(&[1, 2, 3, 4])
        .build();

    println!("{matrix}");

    // println!("{:?}", matrix.iter_rows().collect::<Vec<_>>());

    matrix.cover(1.into());

    println!("{matrix}");

    matrix.uncover(1.into());

    println!("{matrix}");
}
