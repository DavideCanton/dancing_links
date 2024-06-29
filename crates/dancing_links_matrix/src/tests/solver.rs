use std::collections::{HashMap, HashSet};

use test_case::test_matrix;

use crate::{
    solver::IterativeAlgorithmXSolver, DancingLinksMatrix, MatrixBuilder, RecursiveAlgorithmXSolver,
};

#[test_matrix(["iter", "rec"]; "solver_type")]
fn solve_single_sol(solver_type: &str) {
    let matrix = MatrixBuilder::from_iterable([1, 2, 3, 4, 5, 6])
        .add_row([1, 2])
        .add_row([3, 4])
        .add_row([5, 6])
        .add_row([2, 3, 5])
        .build();

    let (found, solutions) = solver(solver_type, matrix, true);
    assert!(found);
    assert!(solutions.len() == 1);

    let solution = &solutions[0];
    let k: HashSet<_> = solution.keys().copied().collect();

    assert_eq!(k, HashSet::from_iter([1, 2, 3]));
    assert_eq!(solution[&1], vec![1, 2]);
    assert_eq!(solution[&2], vec![3, 4]);
    assert_eq!(solution[&3], vec![5, 6]);
}

#[test_matrix(["iter", "rec"]; "solver_type")]
fn solve_multiple_sol(solver_type: &str) {
    let matrix = MatrixBuilder::from_iterable([1, 2, 3, 4, 5, 6])
        .add_row([1, 2])
        .add_row([3, 4])
        .add_row([5, 6])
        .add_row([2, 3, 5])
        .add_row([1, 4, 6])
        .build();

    let (found, solutions) = solver(solver_type, matrix, false);
    assert!(found);
    assert_eq!(solutions.len(), 2);

    let solution = solutions.iter().find(|v| v.len() == 3).unwrap();
    let k: HashSet<_> = solution.keys().copied().collect();

    assert_eq!(k, HashSet::from_iter([1, 2, 3]));
    assert_eq!(solution[&1], vec![1, 2]);
    assert_eq!(solution[&2], vec![3, 4]);
    assert_eq!(solution[&3], vec![5, 6]);

    let solution = solutions.iter().find(|v| v.len() == 2).unwrap();
    let k: HashSet<_> = solution.keys().copied().collect();

    assert_eq!(k, HashSet::from_iter([4, 5]));
    assert_eq!(solution[&4], vec![2, 3, 5]);
    assert_eq!(solution[&5], vec![1, 4, 6]);
}

#[test_matrix(["iter", "rec"]; "solver_type")]
fn solve_first_sol(solver_type: &str) {
    let matrix = MatrixBuilder::from_iterable([1, 2, 3, 4])
        .add_row([1, 2])
        .add_row([3, 4])
        .add_row([2, 3])
        .add_row([1, 4])
        .build();

    let (found, solutions) = solver(solver_type, matrix, true);
    assert!(found);
    assert_eq!(solutions.len(), 1);

    let solution = &solutions[0];
    let k: HashSet<_> = solution.keys().copied().collect();

    if k == HashSet::from_iter([1, 2]) {
        assert_eq!(solution[&1], vec![1, 2]);
        assert_eq!(solution[&2], vec![3, 4]);
    } else if k == HashSet::from_iter([3, 4]) {
        assert_eq!(solution[&3], vec![3, 4]);
        assert_eq!(solution[&4], vec![1, 4]);
    } else {
        panic!("Unexpected solution {k:?}");
    }
}

fn solver(
    solver_type: &str,
    matrix: DancingLinksMatrix<usize>,
    stop: bool,
) -> (bool, Vec<HashMap<usize, Vec<usize>>>) {
    match solver_type {
        "iter" => iter_solver(matrix, stop),
        "rec" => rec_solver(matrix, stop),
        _ => unreachable!(),
    }
}

fn rec_solver(
    matrix: DancingLinksMatrix<usize>,
    stop: bool,
) -> (bool, Vec<HashMap<usize, Vec<usize>>>) {
    let mut solutions = Vec::new();

    let sol = &mut solutions;
    let found = RecursiveAlgorithmXSolver::new(
        matrix,
        move |s| {
            sol.push(s.solution_map.clone());
            stop
        },
        true,
    )
    .solve();

    (found, solutions)
}

fn iter_solver(
    matrix: DancingLinksMatrix<usize>,
    stop: bool,
) -> (bool, Vec<HashMap<usize, Vec<usize>>>) {
    let solutions = IterativeAlgorithmXSolver::new(matrix, true, stop).solve();

    (
        !solutions.is_empty(),
        solutions.into_iter().map(|v| v.solution_map).collect(),
    )
}
