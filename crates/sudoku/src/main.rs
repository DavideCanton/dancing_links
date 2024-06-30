use std::{
    collections::HashMap,
    env::args,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};

use dancing_links_matrix::{
    DancingLinksMatrix, IterativeAlgorithmXSolver, MatrixBuilder, Solution,
};
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

fn print_sol(sol: Solution<String>) {
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
}

#[time]
fn solve(matrix: DancingLinksMatrix<String>) {
    let mut solver = IterativeAlgorithmXSolver::new(matrix, true, true);
    let solutions = solver.solve();

    match solutions.into_iter().next() {
        None => {
            println!("No solution found");
        }
        Some(sol) => {
            print_sol(sol);
        }
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

fn load_board(path: &Path) -> HashMap<(usize, usize), usize> {
    let mut map = HashMap::new();

    let mut reader = BufReader::new(File::open(path).expect("Failed to open file"));

    let mut line_buf = String::with_capacity(11);

    for i in 1..=9 {
        reader
            .read_line(&mut line_buf)
            .expect("Failed to read line");

        for (j, c) in line_buf.chars().enumerate() {
            if j == 9 {
                break;
            }

            if c != '.' {
                map.insert((i, j + 1), c.to_digit(10).unwrap() as usize);
            }
        }
        line_buf.clear();
    }

    map
}

fn main() {
    let level = if cfg!(debug_assertions) {
        Level::Debug
    } else {
        Level::Info
    };
    simple_logger::init_with_level(level).unwrap();

    let path = args()
        .nth(1)
        .map(|v| PathBuf::from_str(&v).expect("Invalid path"))
        .expect("Path not specified");

    if !path.is_file() {
        panic!("Not a file");
    }

    let known = load_board(&path);

    let matrix = build_matrix(known);

    solve(matrix);
}
