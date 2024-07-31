use bumpalo::Bump;
use clap::Parser;
use cmd_common::{init_log, BumpArena, CommonArgs};
use dancing_links_matrix::{
    Arena, ColumnSpec, DancingLinksMatrix, IterativeAlgorithmXSolver, MatrixBuilder, Solution,
};
use itertools::Itertools;
use logging_timer::time;

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
fn build_matrix<'a>(n: usize, arena: &'a impl Arena) -> DancingLinksMatrix<'a, String> {
    let mut matrix_builder = MatrixBuilder::from_iterable(names(n));

    for (i, j) in (0..n).cartesian_product(0..n) {
        let row = compute_row(i, j, n);
        matrix_builder = matrix_builder.add_sorted_row_index(row);
    }

    matrix_builder.build(arena)
}

#[time("info")]
fn solve<'a>(matrix: DancingLinksMatrix<'a, String>, n: usize) {
    let solver = IterativeAlgorithmXSolver::new(matrix, true, true);
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

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "Number of queens", default_value_t = 8)]
    n: usize,
    #[command(flatten)]
    common_args: CommonArgs,
}

fn main() {
    let args = Args::parse();
    init_log(&args.common_args);

    let n = args.n;
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(n, &arena);
    solve(matrix, n);
}
