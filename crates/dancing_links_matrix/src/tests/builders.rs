use std::collections::HashMap;

use crate::{
    cells::{HeaderCell, HeaderName, MatrixCell},
    index::{Index, IndexOps, VecIndex},
    DancingLinksMatrix, MatrixBuilder,
};

use super::utils::create_row;

/// Matrix:
/// ```md
/// | 0   | 1   | 2   | 3   |
/// | --- | --- | --- | --- |
/// |     | 4   | 5   |     |
/// |     | 6   |     | 7   |
/// |     |     | 8   | 9   |
/// |     | 10  | 11  | 12  |
/// ```
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

#[test]
fn test_builder_headers() {
    let matrix = build_matrix();

    assert_eq!(matrix.rows, 4);
    assert_eq!(matrix.columns, 3);

    let cell_map = index_map(&matrix.cells);
    let headers_map = index_map(&matrix.headers);

    assert_eq!(cell_map.len(), 13);
    assert_eq!(headers_map.len(), 4);

    check_header(&headers_map, &cell_map, 0, HeaderName::First);
    check_header(
        &headers_map,
        &cell_map,
        1,
        HeaderName::Other("1".to_string()),
    );
    check_header(
        &headers_map,
        &cell_map,
        2,
        HeaderName::Other("2".to_string()),
    );
    check_header(
        &headers_map,
        &cell_map,
        3,
        HeaderName::Other("3".to_string()),
    );
}

#[test]
fn test_builder_cells() {
    let matrix = build_matrix();

    assert_eq!(matrix.rows, 4);
    assert_eq!(matrix.columns, 3);

    let cell_map = index_map(&matrix.cells);
    let headers_map = index_map(&matrix.headers);

    assert_eq!(cell_map.len(), 13);
    assert_eq!(headers_map.len(), 4);

    check_cell(&cell_map, &headers_map, 0, 0, 0, 3, 1, 0);
    check_cell(&cell_map, &headers_map, 1, 10, 4, 0, 2, 1);
    check_cell(&cell_map, &headers_map, 2, 11, 5, 1, 3, 2);
    check_cell(&cell_map, &headers_map, 3, 12, 7, 2, 0, 3);

    check_cell(&cell_map, &headers_map, 4, 1, 6, 5, 5, 1);
    check_cell(&cell_map, &headers_map, 5, 2, 8, 4, 4, 2);

    check_cell(&cell_map, &headers_map, 6, 4, 10, 7, 7, 1);
    check_cell(&cell_map, &headers_map, 7, 3, 9, 6, 6, 3);

    check_cell(&cell_map, &headers_map, 8, 5, 11, 9, 9, 2);
    check_cell(&cell_map, &headers_map, 9, 7, 12, 8, 8, 3);

    check_cell(&cell_map, &headers_map, 10, 6, 1, 12, 11, 1);
    check_cell(&cell_map, &headers_map, 11, 8, 2, 10, 12, 2);
    check_cell(&cell_map, &headers_map, 12, 9, 3, 11, 10, 3);
}

fn index_map<T>(index: &VecIndex<T>) -> HashMap<usize, *mut T> {
    let mut map = HashMap::new();
    unsafe {
        for i in 0..index.len() {
            let ptr = index.get_mut_ptr(i);
            map.insert(i, ptr);
        }
    }
    map
}

#[allow(clippy::too_many_arguments)]
fn check_cell(
    cell_map: &HashMap<usize, *mut MatrixCell<String>>,
    headers_map: &HashMap<usize, *mut HeaderCell<String>>,
    index: usize,
    up: usize,
    down: usize,
    left: usize,
    right: usize,
    header: usize,
) {
    let cell = *cell_map
        .get(&index)
        .unwrap_or_else(|| panic!("Cannot find cell with index {index}"));

    unsafe {
        assert_eq!((*cell).up, *cell_map.get(&up).unwrap());
        assert_eq!((*cell).down, *cell_map.get(&down).unwrap());
        assert_eq!((*cell).left, *cell_map.get(&left).unwrap());
        assert_eq!((*cell).right, *cell_map.get(&right).unwrap());
        assert_eq!((*cell).header, *headers_map.get(&header).unwrap());
    }
}

fn check_header(
    headers_map: &HashMap<usize, *mut HeaderCell<String>>,
    cell_map: &HashMap<usize, *mut MatrixCell<String>>,
    index: usize,
    name: HeaderName<String>,
) {
    let header = *headers_map
        .get(&index)
        .unwrap_or_else(|| panic!("Cannot find header with index {index}"));

    unsafe {
        assert_eq!((*header).name, name);
    }

    let cell = *cell_map
        .get(&index)
        .unwrap_or_else(|| panic!("Cannot find cell with index {index}"));

    unsafe {
        assert_eq!((*cell).header, header);
        assert_eq!((*header).cell, cell);
    }
}