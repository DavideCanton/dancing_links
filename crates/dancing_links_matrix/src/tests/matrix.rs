use itertools::Itertools;
use test_case::test_matrix;

use crate::{
    cells::HeaderName,
    matrix::{CellIteratorDirection, HeaderIteratorDirection, RowIterator},
    DancingLinksMatrix, MatrixBuilder,
};
use HeaderName::{First as F, Other as O};

use super::utils::create_row;

fn find_cell(mat: &DancingLinksMatrix<String>, row: usize, name: &str) -> Option<usize> {
    unsafe { mat.locate_cell(row, name).map(|c| (*c).index) }
}

fn find_header(mat: &DancingLinksMatrix<String>, name: &str) -> Option<usize> {
    unsafe { mat.locate_header(name).map(|h| (*h).index) }
}

#[test]
fn test_locate_cell() {
    let matrix = build_matrix();

    assert_eq!(find_cell(&matrix, 1, "1").unwrap(), 4);
    assert_eq!(find_cell(&matrix, 1, "2").unwrap(), 5);
    assert_eq!(find_cell(&matrix, 1, "3"), None);
    assert_eq!(find_cell(&matrix, 2, "1").unwrap(), 6);
    assert_eq!(find_cell(&matrix, 2, "2"), None);
    assert_eq!(find_cell(&matrix, 2, "3").unwrap(), 7);
}

#[test]
fn test_locate_header() {
    let matrix = build_matrix();
    assert_eq!(find_header(&matrix, "1").unwrap(), 1);
    assert_eq!(find_header(&matrix, "2").unwrap(), 2);
    assert_eq!(find_header(&matrix, "6"), None);
}

#[test]
fn test_iterator() {
    let matrix = build_matrix();

    let mut it = matrix.iter_rows::<str>();
    assert_eq!(it.len(), 4);

    fn n<'a>(it: &'a mut RowIterator<String, str>) -> Vec<&'a str> {
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
    let matrix = MatrixBuilder
        .add_column("1")
        .add_column("2")
        .add_column("3")
        .end_columns()
        .build();

    let mut it = matrix.iter_rows::<str>();
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);
}

#[test_matrix([true, false]; "include_start")]
fn test_header_cell_iterator_right_from_first(include_start: bool) {
    let matrix = build_matrix();

    let actual: Vec<HeaderName<_>> = matrix
        .iterate_headers(
            matrix.header_index,
            HeaderIteratorDirection::Right,
            include_start,
        )
        .map(|h| unsafe { (*h).name.clone() })
        .collect();

    let mut exp = vec![O("1".to_string()), O("2".to_string()), O("3".to_string())];
    if include_start {
        exp.insert(0, F);
    }

    assert_eq!(actual, exp);
}

#[test_matrix([true, false]; "include_start")]
fn test_header_cell_iterator_right(include_start: bool) {
    let matrix = build_matrix();
    let index = matrix.locate_header("1").unwrap();

    let actual: Vec<HeaderName<_>> = matrix
        .iterate_headers(index, HeaderIteratorDirection::Right, include_start)
        .map(|h| unsafe { (*h).name.clone() })
        .collect();

    let mut exp = vec![O("2".to_string()), O("3".to_string()), F];
    if include_start {
        exp.insert(0, O("1".to_string()));
    }

    assert_eq!(actual, exp);
}

#[test_matrix([true, false]; "include_start")]
fn test_header_cell_iterator_left(include_start: bool) {
    let matrix = build_matrix();
    let index = matrix.locate_header("1").unwrap();

    let actual: Vec<HeaderName<_>> = matrix
        .iterate_headers(index, HeaderIteratorDirection::Left, include_start)
        .map(|h| unsafe { (*h).name.clone() })
        .collect();
    let mut exp = vec![F, O("3".to_string()), O("2".to_string())];
    if include_start {
        exp.insert(0, O("1".to_string()));
    }

    assert_eq!(actual, exp);
}

#[test_matrix([true, false]; "include_start")]
fn test_cell_iterator_left(include_start: bool) {
    let matrix = build_matrix();

    let index = matrix.locate_cell(4, "2").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(index, CellIteratorDirection::Left, include_start)
        .map(|h| unsafe { (*h).index })
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
    let matrix = build_matrix();

    let index = matrix.locate_cell(4, "2").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(index, CellIteratorDirection::Right, include_start)
        .map(|h| unsafe { (*h).index })
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
    let matrix = build_matrix();

    let index = matrix.locate_cell(2, "1").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(index, CellIteratorDirection::Up, include_start)
        .map(|h| unsafe { (*h).index })
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
    let matrix = build_matrix();

    let index = matrix.locate_cell(1, "2").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(index, CellIteratorDirection::Down, include_start)
        .map(|h| unsafe { (*h).index })
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

fn build_matrix() -> DancingLinksMatrix<String> {
    MatrixBuilder
        .add_column(1.to_string())
        .add_column(2.to_string())
        .add_column(3.to_string())
        .end_columns()
        .add_sorted_row(create_row(["1", "2"]))
        .add_sorted_row(create_row(["1", "3"]))
        .add_sorted_row(create_row(["2", "3"]))
        .add_sorted_row(create_row(["1", "2", "3"]))
        .build()
}
