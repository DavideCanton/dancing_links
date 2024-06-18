use std::collections::HashMap;

use dancing_links_matrix::{AlgorithmXSolver, DancingLinksMatrix, MatrixBuilder, Solution};
use itertools::Itertools;
use logging_timer::{time, Level};

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

    let i4 = 81 * 3 + v + 9 * (3 * iq + jq);
    [i1, i2, i3, i4].into_iter().map(|v| v + 1).collect_vec()
}

const M: usize = 20;

fn sol_callback(sol: &Solution<String>) -> bool {
    let mut matrix = vec![0; 81];

    for v in sol.solution_map.values() {
        let mut i = M;
        let mut j = M;
        let mut val = 0;
        for el in v {
            let chs = el
                .chars()
                .skip(1)
                .take(3)
                .collect_tuple::<(char, char, char)>()
                .unwrap();

            if chs.1 == '#' {
                if val == 0 {
                    val = chs.2.to_digit(10).unwrap() as usize;
                }
            } else {
                if i == M {
                    i = chs.0.to_digit(10).unwrap() as usize - 1;
                }
                if j == M {
                    j = chs.2.to_digit(10).unwrap() as usize - 1;
                }
            }
        }
        debug_assert_ne!(val, 0);
        debug_assert_ne!(i, M);
        debug_assert_ne!(j, M);

        let fi = 9 * i + j;
        matrix[fi] = val;
    }

    for i in 0..9 {
        for j in 0..9 {
            print!("{}", matrix[9 * i + j]);
            if j < 8 {
                print!(" ");
            }
        }
        println!();
    }
    true
}

#[time]
fn solve(matrix: DancingLinksMatrix<String>) {
    let mut solver = AlgorithmXSolver::new(matrix, sol_callback, true);

    if !solver.solve() {
        println!("No solution found");
    }
}

#[time]
fn build_matrix(
    known: HashMap<(usize, usize), usize>,
) -> dancing_links_matrix::DancingLinksMatrix<String> {
    let mut matrix_builder = MatrixBuilder::from_iterable(names());

    for (i, j) in prod() {
        match known.get(&(i, j)) {
            Some(v) => {
                let row = compute_row(i, j, *v);
                matrix_builder = matrix_builder.add_sorted_row_key(row);
            }
            None => {
                for v in 1..=9 {
                    let row = compute_row(i, j, v);
                    matrix_builder = matrix_builder.add_sorted_row_key(row);
                }
            }
        }
    }

    matrix_builder.build()
}

fn load_board() -> HashMap<(usize, usize), usize> {
    let mut known: HashMap<(usize, usize), usize> = HashMap::new();
    known.insert((1, 4), 9);
    known.insert((1, 8), 4);
    known.insert((1, 9), 8);
    known.insert((2, 4), 1);
    known.insert((2, 6), 8);
    known.insert((2, 9), 9);
    known.insert((3, 3), 9);
    known.insert((4, 5), 6);
    known.insert((4, 7), 1);
    known.insert((4, 8), 9);
    known.insert((5, 3), 6);
    known.insert((6, 4), 8);
    known.insert((6, 6), 9);
    known.insert((6, 9), 6);
    known.insert((7, 6), 6);
    known.insert((7, 8), 8);
    known.insert((8, 5), 8);
    known.insert((9, 3), 8);
    known.insert((9, 5), 7);
    known.insert((9, 7), 6);
    known
}

fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let known = load_board();

    let matrix = build_matrix(known);

    solve(matrix);
}
