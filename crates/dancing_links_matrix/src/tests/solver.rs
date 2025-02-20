use std::collections::{HashMap, HashSet};

use bumpalo::Bump;

use crate::{MatrixBuilder, solver::IterativeAlgorithmXSolver, tests::utils::BumpArena};

#[test]
fn solve_single_sol() {
    let arena: BumpArena = Bump::new().into();
    let matrix = MatrixBuilder::from_iterable([1, 2, 3, 4, 5, 6])
        .add_row([1, 2])
        .add_row([3, 4])
        .add_row([5, 6])
        .add_row([2, 3, 5])
        .build(&arena);

    let solver = IterativeAlgorithmXSolver::new(matrix, true, true);
    let mut solutions = solve(&solver);
    assert!(solutions.len() == 1);

    let solution = &mut solutions[0];

    check(solution.keys(), [1, 2, 3]);
    check(solution.remove(&1).unwrap(), [1, 2]);
    check(solution.remove(&2).unwrap(), [3, 4]);
    check(solution.remove(&3).unwrap(), [5, 6]);
}

#[test]
fn solve_multiple_sol() {
    let arena: BumpArena = Bump::new().into();
    let matrix = MatrixBuilder::from_iterable([1, 2, 3, 4, 5, 6])
        .add_row([1, 2])
        .add_row([3, 4])
        .add_row([5, 6])
        .add_row([2, 3, 5])
        .add_row([1, 4, 6])
        .build(&arena);

    let solver = IterativeAlgorithmXSolver::new(matrix, true, false);
    let mut solutions = solve(&solver);
    assert_eq!(solutions.len(), 2);

    let mut solution = find_and_remove(&mut solutions, |v| v.len() == 3).unwrap();

    check(solution.keys(), [1, 2, 3]);
    check(solution.remove(&1).unwrap(), [1, 2]);
    check(solution.remove(&2).unwrap(), [3, 4]);
    check(solution.remove(&3).unwrap(), [5, 6]);

    let mut solution = find_and_remove(&mut solutions, |v| v.len() == 2).unwrap();

    check(solution.keys(), [4, 5]);
    check(solution.remove(&4).unwrap(), [2, 3, 5]);
    check(solution.remove(&5).unwrap(), [1, 4, 6]);
}

#[test]
fn solve_first_sol() {
    let arena: BumpArena = Bump::new().into();
    let matrix = MatrixBuilder::from_iterable([1, 2, 3, 4])
        .add_row([1, 2])
        .add_row([3, 4])
        .add_row([2, 3])
        .add_row([1, 4])
        .build(&arena);

    let solver = IterativeAlgorithmXSolver::new(matrix, true, true);
    let mut solutions = solve(&solver);
    assert_eq!(solutions.len(), 1);

    let solution = &mut solutions[0];
    let k: HashSet<_> = solution.keys().copied().collect();

    if k == HashSet::from_iter([1, 2]) {
        check(solution.remove(&1).unwrap(), [1, 2]);
        check(solution.remove(&2).unwrap(), [3, 4]);
    } else if k == HashSet::from_iter([3, 4]) {
        check(solution.remove(&3).unwrap(), [3, 4]);
        check(solution.remove(&4).unwrap(), [1, 4]);
    } else {
        panic!("Unexpected solution {k:?}");
    }
}

fn solve<'a>(
    solver: &'a IterativeAlgorithmXSolver<'a, usize>,
) -> Vec<HashMap<usize, Vec<&'a usize>>> {
    let solutions = solver.solve();
    solutions.into_iter().map(|v| v.solution_map).collect()
}

fn check<'a, 'b>(
    actual: impl IntoIterator<Item = &'a usize>,
    expected: impl IntoIterator<Item = usize>,
) {
    let actual: HashSet<_> = actual.into_iter().copied().collect();
    let expected: HashSet<_> = expected.into_iter().collect();
    assert_eq!(actual, expected);
}

fn find_and_remove<T>(v: &mut Vec<T>, pred: impl Fn(&T) -> bool) -> Option<T> {
    let idx = v.iter().position(pred)?;
    Some(v.remove(idx))
}
