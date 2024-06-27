use std::collections::{HashMap, HashSet};

use crate::{AlgorithmXSolver, DancingLinksMatrix, MatrixBuilder};

#[test]
fn solve_single_sol() {
    let matrix = MatrixBuilder::from_iterable([1, 2, 3, 4, 5, 6])
        .add_row([1, 2])
        .add_row([3, 4])
        .add_row([5, 6])
        .add_row([2, 3, 5])
        .build();

    let (found, solutions) = solver(matrix, true);
    assert!(found);
    assert!(solutions.len() == 1);

    let solution = &solutions[0];
    let k: HashSet<_> = solution.keys().copied().collect();

    assert_eq!(k, HashSet::from_iter([1, 2, 3]));
    assert_eq!(solution[&1], vec![1, 2]);
    assert_eq!(solution[&2], vec![3, 4]);
    assert_eq!(solution[&3], vec![5, 6]);
}

#[test]
fn solve_multiple_sol() {
    let matrix = MatrixBuilder::from_iterable([1, 2, 3, 4, 5, 6])
        .add_row([1, 2])
        .add_row([3, 4])
        .add_row([5, 6])
        .add_row([2, 3, 5])
        .add_row([1, 4, 6])
        .build();

    let (found, solutions) = solver(matrix, false);
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

#[test]
fn solve_first_sol() {
    let matrix = MatrixBuilder::from_iterable([1, 2, 3, 4])
        .add_row([1, 2])
        .add_row([3, 4])
        .add_row([2, 3])
        .add_row([1, 4])
        .build();

    let (found, solutions) = solver(matrix, true);
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
    matrix: DancingLinksMatrix<usize>,
    stop: bool,
) -> (bool, Vec<HashMap<usize, Vec<usize>>>) {
    let mut solutions = Vec::new();

    let sol = &mut solutions;
    let found = AlgorithmXSolver::new(
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
