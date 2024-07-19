use itertools::Itertools;
use test_case::test_matrix;

use crate::{cells::HeaderName, matrix::RowIterator, DancingLinksMatrix, MatrixBuilder};
use HeaderName::{First as F, Other as O};

#[test]
fn test_locate_cell() {
    let matrix = build_matrix();
    assert_eq!(matrix.locate_cell(1, "1").unwrap(), 4.into());
    assert_eq!(matrix.locate_cell(1, "2").unwrap(), 5.into());
    assert_eq!(matrix.locate_cell(1, "3"), None);
    assert_eq!(matrix.locate_cell(2, "1").unwrap(), 6.into());
    assert_eq!(matrix.locate_cell(2, "2"), None);
    assert_eq!(matrix.locate_cell(2, "3").unwrap(), 7.into());
}

#[test]
fn test_locate_header() {
    let matrix = build_matrix();
    assert_eq!(matrix.locate_header("1").unwrap(), 1.into());
    assert_eq!(matrix.locate_header("2").unwrap(), 2.into());
    assert_eq!(matrix.locate_header("6"), None);
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
        .iterate_headers(matrix.header_key, |cell| cell.right, include_start)
        .map(|h| h.name.clone())
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
    let key = matrix.locate_header("1").unwrap();

    let actual: Vec<HeaderName<_>> = matrix
        .iterate_headers(key, |cell| cell.right, include_start)
        .map(|h| h.name.clone())
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
    let key = matrix.locate_header("1").unwrap();

    let actual: Vec<HeaderName<_>> = matrix
        .iterate_headers(key, |cell| cell.left, include_start)
        .map(|h| h.name.clone())
        .collect();
    let mut exp = vec![F, O("3".to_string()), O("2".to_string())];
    if include_start {
        exp.insert(0, O("1".to_string()));
    }

    assert_eq!(actual, exp);
}

#[test_matrix([true, false]; "include_start")]
fn test_header_cell_iterator_up(include_start: bool) {
    let matrix = build_matrix();
    let key = matrix.locate_header("1").unwrap();

    let actual: Vec<HeaderName<_>> = matrix
        .iterate_headers(key, |c| c.up, include_start)
        .map(|h| h.name.clone())
        .collect();

    let mut exp = vec![];
    if include_start {
        exp.insert(0, O("1".to_string()));
    }

    assert_eq!(actual, exp);
}

#[test_matrix([true, false]; "include_start")]
fn test_header_cell_iterator_down(include_start: bool) {
    let matrix = build_matrix();
    let key = matrix.locate_header("1").unwrap();

    let actual: Vec<HeaderName<_>> = matrix
        .iterate_headers(key, |c| c.down, include_start)
        .map(|h| h.name.clone())
        .collect();

    let mut exp = vec![];
    if include_start {
        exp.insert(0, O("1".to_string()));
    }

    assert_eq!(actual, exp);
}

#[test_matrix([true, false]; "include_start")]
fn test_cell_iterator_left(include_start: bool) {
    let matrix = build_matrix();

    let key = matrix.locate_cell(4, "2").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(key, |cell| cell.left, include_start)
        .map(|h| h.key)
        .collect();

    let mut exp = vec!["1", "3"];

    if include_start {
        exp.insert(0, "2");
    }

    assert_eq!(
        actual,
        exp.into_iter()
            .map(|v| matrix.locate_cell(4, v).unwrap())
            .collect::<Vec<_>>()
    );
}

#[test_matrix([true, false]; "include_start")]
fn test_cell_iterator_right(include_start: bool) {
    let matrix = build_matrix();

    let key = matrix.locate_cell(4, "2").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(key, |cell| cell.right, include_start)
        .map(|h| h.key)
        .collect();

    let mut exp = vec!["3", "1"];

    if include_start {
        exp.insert(0, "2");
    }

    assert_eq!(
        actual,
        exp.into_iter()
            .map(|v| matrix.locate_cell(4, v).unwrap())
            .collect::<Vec<_>>()
    );
}

#[test_matrix([true, false]; "include_start")]
fn test_cell_iterator_up(include_start: bool) {
    let matrix = build_matrix();

    let key = matrix.locate_cell(2, "1").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(key, |cell| cell.up, include_start)
        .map(|h| h.key)
        .collect();

    let mut exp = vec![1, 0, 4];

    if include_start {
        exp.insert(0, 2);
    }

    assert_eq!(
        actual,
        exp.into_iter()
            .map(|v| matrix.locate_cell(v, "1").unwrap())
            .collect::<Vec<_>>()
    );
}

#[test_matrix([true, false]; "include_start")]
fn test_cell_iterator_down(include_start: bool) {
    let matrix = build_matrix();

    let key = matrix.locate_cell(1, "2").unwrap();

    let actual: Vec<_> = matrix
        .iterate_cells(key, |cell| cell.down, include_start)
        .map(|h| h.key)
        .collect();

    let mut exp = vec![3, 4, 0];

    if include_start {
        exp.insert(0, 1);
    }

    assert_eq!(
        actual,
        exp.into_iter()
            .map(|v| matrix.locate_cell(v, "2").unwrap())
            .collect::<Vec<_>>()
    );
}

fn build_matrix() -> DancingLinksMatrix<String> {
    MatrixBuilder
        .add_column(1.to_string())
        .add_column(2.to_string())
        .add_column(3.to_string())
        .end_columns()
        .add_sorted_row(r(["1", "2"]))
        .add_sorted_row(r(["1", "3"]))
        .add_sorted_row(r(["2", "3"]))
        .add_sorted_row(r(["1", "2", "3"]))
        .build()
}

fn r<const N: usize>(v: [&str; N]) -> Vec<String> {
    v.iter().map(|v| v.to_string()).collect()
}
