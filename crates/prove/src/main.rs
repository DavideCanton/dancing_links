#![allow(dead_code)]

use dancing_links_matrix::{AlgorithmXSolver, MatrixBuilder};

fn main() {
    let matrix = MatrixBuilder::from_iterable(["1", "2", "3", "4", "5", "6", "7"])
        .add_sorted_row(["1", "4", "7"])
        .add_sorted_row(["2", "3", "5"])
        .add_sorted_row(["6"])
        .add_sorted_row(["1", "3", "4", "7"])
        .add_sorted_row(["2", "5"])
        .build();

    let mut solver = AlgorithmXSolver::new(
        matrix,
        |s| {
            println!("{s:?}");
            false
        },
        true,
    );
    solver.solve();
}
