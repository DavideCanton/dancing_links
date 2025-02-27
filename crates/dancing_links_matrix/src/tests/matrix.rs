use bumpalo::Bump;
use itertools::Itertools;
use test_case::test_matrix;

use crate::{
    Arena, DancingLinksMatrix, MatrixBuilder,
    cells::{CellRow, ColumnName, ColumnRef, MatrixCellRef},
    matrix::{CellIteratorDir, ColumnIteratorDir},
    tests::utils::BumpArena,
};
use ColumnName::{First as F, Other as O};

use super::utils::create_row;

#[test]
fn test_locate_cell() {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);

    assert_eq!(find_cell(&matrix, 1, "1").unwrap(), 4);
    assert_eq!(find_cell(&matrix, 1, "2").unwrap(), 5);
    assert_eq!(find_cell(&matrix, 1, "3"), None);
    assert_eq!(find_cell(&matrix, 2, "1").unwrap(), 6);
    assert_eq!(find_cell(&matrix, 2, "2"), None);
    assert_eq!(find_cell(&matrix, 2, "3").unwrap(), 7);
}

#[test]
fn test_locate_column() {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);
    assert_eq!(find_column(&matrix, "1").unwrap(), 1);
    assert_eq!(find_column(&matrix, "2").unwrap(), 2);
    assert_eq!(find_column(&matrix, "6"), None);
}

#[test]
fn test_iterator() {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);

    let mut it = matrix.iter_rows::<str>();

    fn n<'a, IT, IIT>(it: &mut IT) -> Vec<&'a str>
    where
        IT: Iterator<Item = IIT>,
        IIT: IntoIterator<Item = &'a str>,
    {
        it.next().unwrap().into_iter().sorted().collect_vec()
    }

    assert_eq!(n(&mut it), ["1", "2"]);
    assert_eq!(n(&mut it), ["1", "3"]);
    assert_eq!(n(&mut it), ["2", "3"]);
    assert_eq!(n(&mut it), ["1", "2", "3"]);
    assert_eq!(it.next(), None);
}

#[test]
fn test_iterator_no_rows() {
    let arena: BumpArena = Bump::new().into();
    let matrix = MatrixBuilder
        .add_column("1")
        .add_column("2")
        .add_column("3")
        .end_columns()
        .build(&arena);

    let mut it = matrix.iter_rows::<str>();
    assert_eq!(it.next(), None);
}

#[test_matrix([true, false]; "include_start")]
fn test_column_cell_iterator_right_from_first(include_start: bool) {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);

    let actual: Vec<ColumnName<_>> = matrix
        .iterate_columns(
            matrix.first_column(),
            ColumnIteratorDir::Right,
            include_start,
        )
        .map(|h| h.name.clone())
        .collect();

    let mut exp = vec![O("1".to_string()), O("2".to_string()), O("3".to_string())];
    if include_start {
        exp.insert(0, F);
    }

    assert_eq!(actual, exp);
}

#[test_matrix([true, false]; "include_start")]
fn test_column_cell_iterator_right(include_start: bool) {
    let arena: BumpArena = Bump::new().into();
    let matrix: DancingLinksMatrix<String> = build_matrix(&arena);
    let index = locate_column(&matrix, "1").unwrap();

    let actual: Vec<ColumnName<_>> = matrix
        .iterate_columns(index, ColumnIteratorDir::Right, include_start)
        .map(|h| h.name.clone())
        .collect();

    let mut exp = vec![O("2".to_string()), O("3".to_string()), F];
    if include_start {
        exp.insert(0, O("1".to_string()));
    }

    assert_eq!(actual, exp);
}

#[test_matrix([true, false]; "include_start")]
fn test_column_cell_iterator_left(include_start: bool) {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);
    let index = locate_column(&matrix, "1").unwrap();

    let actual: Vec<ColumnName<_>> = matrix
        .iterate_columns(index, ColumnIteratorDir::Left, include_start)
        .map(|h| h.name.clone())
        .collect();
    let mut exp = vec![F, O("3".to_string()), O("2".to_string())];
    if include_start {
        exp.insert(0, O("1".to_string()));
    }

    assert_eq!(actual, exp);
}

#[test_matrix([true, false]; "include_start")]
fn test_cell_iterator_left(include_start: bool) {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);

    let index = locate_cell(&matrix, 4, "2").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(index, CellIteratorDir::Left, include_start)
        .map(|h| h.index)
        .collect();

    let mut exp = vec!["1", "3"];

    if include_start {
        exp.insert(0, "2");
    }

    assert_eq!(
        actual,
        exp.into_iter()
            .map(|v| find_cell(&matrix, 4, v).unwrap())
            .collect::<Vec<_>>()
    );
}

#[test_matrix([true, false]; "include_start")]
fn test_cell_iterator_right(include_start: bool) {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);

    let index = locate_cell(&matrix, 4, "2").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(index, CellIteratorDir::Right, include_start)
        .map(|h| h.index)
        .collect();

    let mut exp = vec!["3", "1"];

    if include_start {
        exp.insert(0, "2");
    }

    assert_eq!(
        actual,
        exp.into_iter()
            .map(|v| find_cell(&matrix, 4, v).unwrap())
            .collect::<Vec<_>>()
    );
}

#[test_matrix([true, false]; "include_start")]
fn test_cell_iterator_up(include_start: bool) {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);

    let index = locate_cell(&matrix, 2, "1").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(index, CellIteratorDir::Up, include_start)
        .map(|h| h.index)
        .collect();

    let mut exp = vec![1, 0, 4];

    if include_start {
        exp.insert(0, 2);
    }

    assert_eq!(
        actual,
        exp.into_iter()
            .map(|v| find_cell(&matrix, v, "1").unwrap())
            .collect::<Vec<_>>()
    );
}

#[test_matrix([true, false]; "include_start")]
fn test_cell_iterator_down(include_start: bool) {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);

    let index = locate_cell(&matrix, 1, "2").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(index, CellIteratorDir::Down, include_start)
        .map(|h| h.index)
        .collect();

    let mut exp = vec![3, 4, 0];

    if include_start {
        exp.insert(0, 1);
    }

    assert_eq!(
        actual,
        exp.into_iter()
            .map(|v| find_cell(&matrix, v, "2").unwrap())
            .collect::<Vec<_>>()
    );
}

fn build_matrix(arena: &impl Arena) -> DancingLinksMatrix<'_, String> {
    MatrixBuilder
        .add_column(1.to_string())
        .add_column(2.to_string())
        .add_column(3.to_string())
        .end_columns()
        .add_sorted_row(create_row(["1", "2"]))
        .add_sorted_row(create_row(["1", "3"]))
        .add_sorted_row(create_row(["2", "3"]))
        .add_sorted_row(create_row(["1", "2", "3"]))
        .build(arena)
}

fn locate_cell<'a, T, C>(
    matrix: &DancingLinksMatrix<'a, T>,
    row: impl Into<CellRow>,
    column: &C,
) -> Option<MatrixCellRef<'a, T>>
where
    T: AsRef<C>,
    C: Eq + ?Sized,
{
    let column = locate_column(matrix, column)?;
    let row = row.into();

    matrix
        .iterate_cells(column.cell(), CellIteratorDir::Down, true)
        .find(|c| c.row == row)
}

fn locate_column<'a, T, C>(
    matrix: &DancingLinksMatrix<'a, T>,
    column: &C,
) -> Option<ColumnRef<'a, T>>
where
    T: AsRef<C>,
    C: Eq + ?Sized,
{
    use crate::cells::ColumnName;

    matrix
        .iterate_columns(matrix.first_column(), ColumnIteratorDir::Right, true)
        .find(|h| matches!(h.name, ColumnName::Other(ref c) if *c.as_ref() == *column))
}

fn find_cell<'a>(mat: &'a DancingLinksMatrix<'a, String>, row: usize, name: &str) -> Option<usize> {
    locate_cell(mat, row, name).map(|c| c.index)
}

fn find_column<'a>(mat: &'a DancingLinksMatrix<'a, String>, name: &str) -> Option<usize> {
    locate_column(mat, name).map(|h| h.index)
}
