use std::env::args;

use dancing_links_matrix::{
    ColumnSpec, DancingLinksMatrix, IterativeAlgorithmXSolver, MatrixBuilder,
    RecursiveAlgorithmXSolver, Solution,
};
use itertools::Itertools;
use logging_timer::{time, Level};

fn names(n: usize) -> Vec<ColumnSpec<String>> {
    let mut names = Vec::new();

    for i in 0..n {
        names.push(format!("R{i}").into());
    }
    for i in 0..n {
        names.push(format!("F{i}").into());
    }
    for i in 0..2 * n - 1 {
        names.push(ColumnSpec::secondary(format!("A{i}")));
    }
    for i in 0..2 * n - 1 {
        names.push(ColumnSpec::secondary(format!("B{i}")));
    }

    names
}

fn compute_row(i: usize, j: usize, n: usize) -> Vec<usize> {
    // R is 0 .. N-1
    // F is N .. 2*N-1
    // A is 2*N .. 4*N - 2
    // B is 4*N - 1 .. 6*N - 3
    [i, n + j, 2 * n + i + j, 5 * n - 2 - i + j]
        .into_iter()
        .map(|v| v + 1)
        .collect()
}

#[time("info")]
fn build_matrix(n: usize) -> DancingLinksMatrix<String> {
    let mut matrix_builder = MatrixBuilder::from_iterable(names(n));

    for (i, j) in (0..n).cartesian_product(0..n) {
        let row = compute_row(i, j, n);
        matrix_builder = matrix_builder.add_sorted_row_key(row);
    }

    matrix_builder.build()
}

#[time("info")]
fn solve(matrix: DancingLinksMatrix<String>, n: usize) {
    let mut solver = IterativeAlgorithmXSolver::new(matrix, true, true);
    let solutions = solver.solve();

    match solutions.into_iter().next() {
        None => {
            println!("No solution found");
        }
        Some(sol) => {
            print_sol(n, &sol);
        }
    }
}

#[time("info")]
fn solve_rec(matrix: DancingLinksMatrix<String>, n: usize) {
    let mut solver = RecursiveAlgorithmXSolver::new(matrix, move |sol| print_sol(n, sol), true);
    let found = solver.solve();

    if !found {
        println!("No solution found");
    }
}

fn print_sol(n: usize, sol: &Solution<String>) -> bool {
    let mut pos = vec![0; n];

    for v in sol.solution_map.values() {
        let v = v.iter().sorted().collect_vec();
        let c = v[2][1..].parse::<usize>().unwrap();
        let r = v[3][1..].parse::<usize>().unwrap();
        pos[r] = c;
    }

    for i in 0..n {
        let mut r = vec![' '; n];
        r[pos[i]] = 'O';
        println!("|{}|", r.into_iter().join("|"));
    }

    true
}

fn main() {
    let level = if cfg!(debug_assertions) {
        Level::Debug
    } else {
        Level::Info
    };
    simple_logger::init_with_level(level).unwrap();

    let n = args()
        .nth(1)
        .map(|v| v.parse::<usize>().expect("Invalid size"))
        .unwrap_or(8);

    let iterative = args().nth(2).map(|v| v == "1").unwrap_or(false);
    let matrix = build_matrix(n);

    if iterative {
        solve(matrix, n);
    } else {
        solve_rec(matrix, n);
    }
}
