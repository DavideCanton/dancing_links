#![allow(dead_code)]

use std::collections::HashMap;

use dancing_links_matrix::{AlgorithmXSolver, MatrixBuilder};
use itertools::Itertools;

fn prod() -> impl Iterator<Item = (usize, usize)> {
    let range = 1..=9;
    range.clone().cartesian_product(range.clone())
}

fn names() -> Vec<String> {
    let mut names = Vec::new();

    for (i, j) in prod() {
        names.push(format!("R{i}C{j}"));
    }

    for (i, j) in prod() {
        names.push(format!("R{i}#{j}"));
    }

    for (i, j) in prod() {
        names.push(format!("C{i}#{j}"));
    }

    for (i, j) in prod() {
        names.push(format!("B{i}#{j}"));
    }

    names
}

fn compute_row(mut i: usize, mut j: usize, mut v: usize) -> Vec<usize> {
    i -= 1;
    j -= 1;
    v -= 1;
    let i1 = j + 9 * i;
    let i2 = 81 + v + 9 * i;
    let i3 = 81 * 2 + v + 9 * j;

    let iq = i / 3;
    let jq = j / 3;

    let i4 = 81 * 3 + v + 9 * 3 * iq + jq;
    [i1, i2, i3, i4].into_iter().map(|v| v + 1).collect_vec()
}

fn main() {
    let mut known: HashMap<(usize, usize), usize> = HashMap::new();
    known.insert((0, 3), 9);
    known.insert((0, 7), 4);
    known.insert((0, 8), 8);
    known.insert((1, 3), 1);
    known.insert((1, 5), 8);
    known.insert((1, 8), 9);
    known.insert((2, 2), 9);
    known.insert((3, 4), 6);
    known.insert((3, 6), 1);
    known.insert((3, 7), 9);
    known.insert((4, 2), 6);
    known.insert((5, 3), 8);
    known.insert((5, 5), 9);
    known.insert((5, 8), 6);
    known.insert((6, 5), 6);
    known.insert((6, 7), 8);
    known.insert((7, 4), 8);
    known.insert((8, 2), 8);
    known.insert((8, 4), 7);
    known.insert((8, 6), 6);

    let mut matrix_builder = MatrixBuilder::from_iterable(names());

    for (i, j) in prod() {
        match known.get(&(i, j)) {
            Some(v) => {
                let row = compute_row(i, j, *v);
                matrix_builder = matrix_builder.add_row_key(row);
            }
            None => {
                for v in 1..=9 {
                    let row = compute_row(i, j, v);
                    matrix_builder = matrix_builder.add_row_key(row);
                }
            }
        }
    }

    let matrix = matrix_builder.build();

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
