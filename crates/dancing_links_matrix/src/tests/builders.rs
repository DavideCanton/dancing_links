use std::collections::HashMap;

use bumpalo::Bump;

use crate::{
    cells::{
        ColumnName::{self, First as F, Other as O},
        ColumnRef, MatrixCellRef,
    },
    tests::utils::BumpArena,
    Arena, DancingLinksMatrix, MatrixBuilder,
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

#[test]
fn test_builder_columns() {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);

    assert_eq!(matrix.row_count, 4);
    assert_eq!(matrix.column_count, 3);

    let cells_map = index_map(&matrix.cells);
    let columns_map = index_map(&matrix.columns);

    assert_eq!(cells_map.len(), 13);
    assert_eq!(columns_map.len(), 4);

    check_column(&columns_map, &cells_map, 0, F);
    check_column(&columns_map, &cells_map, 1, O("1".to_string()));
    check_column(&columns_map, &cells_map, 2, O("2".to_string()));
    check_column(&columns_map, &cells_map, 3, O("3".to_string()));
}

#[test]
fn test_builder_cells() {
    let arena: BumpArena = Bump::new().into();
    let matrix = build_matrix(&arena);

    assert_eq!(matrix.row_count, 4);
    assert_eq!(matrix.column_count, 3);

    let cells_map = index_map(&matrix.cells);
    let columns_map = index_map(&matrix.columns);

    assert_eq!(cells_map.len(), 13);
    assert_eq!(columns_map.len(), 4);

    check_cell(&cells_map, &columns_map, 0, 0, 0, 3, 1, 0);
    check_cell(&cells_map, &columns_map, 1, 10, 4, 0, 2, 1);
    check_cell(&cells_map, &columns_map, 2, 11, 5, 1, 3, 2);
    check_cell(&cells_map, &columns_map, 3, 12, 7, 2, 0, 3);

    check_cell(&cells_map, &columns_map, 4, 1, 6, 5, 5, 1);
    check_cell(&cells_map, &columns_map, 5, 2, 8, 4, 4, 2);

    check_cell(&cells_map, &columns_map, 6, 4, 10, 7, 7, 1);
    check_cell(&cells_map, &columns_map, 7, 3, 9, 6, 6, 3);

    check_cell(&cells_map, &columns_map, 8, 5, 11, 9, 9, 2);
    check_cell(&cells_map, &columns_map, 9, 7, 12, 8, 8, 3);

    check_cell(&cells_map, &columns_map, 10, 6, 1, 12, 11, 1);
    check_cell(&cells_map, &columns_map, 11, 8, 2, 10, 12, 2);
    check_cell(&cells_map, &columns_map, 12, 9, 3, 11, 10, 3);
}

fn index_map<'a, T>(index: &[&'a T]) -> HashMap<usize, &'a T> {
    let mut map = HashMap::new();
    for (i, ptr) in index.iter().enumerate() {
        map.insert(i, *ptr);
    }
    map
}

#[allow(clippy::too_many_arguments)]
fn check_cell<'a>(
    cells_map: &HashMap<usize, MatrixCellRef<'a, String>>,
    columns_map: &HashMap<usize, ColumnRef<'a, String>>,
    index: usize,
    up: usize,
    down: usize,
    left: usize,
    right: usize,
    column: usize,
) {
    let cell = cells_map
        .get(&index)
        .unwrap_or_else(|| panic!("Cannot find cell with index {index}"));

    assert_eq!(cell.up().index, cells_map.get(&up).unwrap().index);
    assert_eq!(cell.down().index, cells_map.get(&down).unwrap().index);
    assert_eq!(cell.left().index, cells_map.get(&left).unwrap().index);
    assert_eq!(cell.right().index, cells_map.get(&right).unwrap().index);
    assert_eq!(cell.column().index, columns_map.get(&column).unwrap().index);
}

fn check_column<'a>(
    columns_map: &HashMap<usize, ColumnRef<'a, String>>,
    cells_map: &HashMap<usize, MatrixCellRef<'a, String>>,
    index: usize,
    name: ColumnName<String>,
) {
    let column = *columns_map
        .get(&index)
        .unwrap_or_else(|| panic!("Cannot find column with index {index}"));

    assert_eq!(column.name, name);

    let cell = *cells_map
        .get(&index)
        .unwrap_or_else(|| panic!("Cannot find cell with index {index}"));

    assert_eq!(cell.column().index, column.index);
    assert_eq!(column.cell().index, cell.index);
}
