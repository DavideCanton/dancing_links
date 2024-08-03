use std::ptr;

use crate::cells::{CellRow, ColumnInfo, ColumnName, MatrixCell, ProtoCell, ProtoColumn};

use ColumnName::{First as F, Other as O};

#[test]
fn test_matrix_cell_new() {
    let cell: MatrixCell<()> = MatrixCell::new(42, 3.into());
    assert_eq!(cell.index, 42);
    assert!(!cell.has_up());
    assert!(!cell.has_down());
    assert!(!cell.has_left());
    assert!(!cell.has_right());
    assert!(!cell.has_column());
    assert_eq!(cell.row, 3.into());
}

#[test]
fn test_matrix_cell_update_pointers() {
    let column = ColumnInfo::new(O(1), 3, 5, true);

    let cell = MatrixCell::new(42, 3.into());
    let up = MatrixCell::new(43, 2.into());
    let down = MatrixCell::new(44, 4.into());
    let left = MatrixCell::new(45, 1.into());
    let right = MatrixCell::new(46, 5.into());

    cell.update_pointers(&up, &down, &left, &right, &column);

    assert!(ptr::eq(cell.up(), &up));
    assert!(ptr::eq(cell.down(), &down));
    assert!(ptr::eq(cell.left(), &left));
    assert!(ptr::eq(cell.right(), &right));
    assert!(ptr::eq(cell.column(), &column));

    assert!(cell.has_up());
    assert!(cell.has_down());
    assert!(cell.has_left());
    assert!(cell.has_right());
    assert!(cell.has_column());
}

#[test]
fn test_proto_cell_new() {
    let proto = ProtoCell::new(42, 3, CellRow::Header);
    assert_eq!(proto.index, 42);
    assert_eq!(proto.column, 3);
    assert_eq!(proto.row, CellRow::Header);
    assert_eq!(proto.up, 42);
    assert_eq!(proto.down, 42);
    assert_eq!(proto.left, 42);
    assert_eq!(proto.right, 42);
}

#[test]
fn test_proto_column_new() {
    let proto = ProtoColumn::<()>::new(42, F, true);
    assert_eq!(proto.index, 42);
    assert_eq!(proto.name, F);
    assert_eq!(proto.size, 0);
    assert!(proto.primary);

    let proto2 = ProtoColumn::new(42, O(1), false);
    assert_eq!(proto2.name, O(1));
    assert!(!proto2.primary);
}

#[test]
fn test_column_new() {
    let column = ColumnInfo::new(O(1), 3, 2, true);
    assert_eq!(column.name, O(1));
    assert_eq!(column.size(), 2);
    assert_eq!(column.index, 3);
    assert!(column.primary);
    assert!(!column.has_cell());

    let column = ColumnInfo::<u32>::new(F, 10, 8, false);
    assert_eq!(column.name, F);
    assert_eq!(column.size(), 8);
    assert_eq!(column.index, 10);
    assert!(!column.primary);
    assert!(!column.has_cell());
}

#[test]
fn test_column_from_proto() {
    let mut proto = ProtoColumn::new(42, O(1), true);
    let column = ColumnInfo::from_proto(proto.clone());
    assert_eq!(column.name, O(1));
    assert_eq!(column.size(), 0);
    assert_eq!(column.index, 42);
    assert!(!column.has_cell());
    assert!(column.primary);

    proto.primary = false;
    let column = ColumnInfo::from_proto(proto);
    assert!(!column.primary);
}

#[test]
fn test_column_update_pointer() {
    let column = ColumnInfo::<u32>::new(F, 10, 8, true);
    let cell = MatrixCell::new(42, CellRow::Header);

    column.update_pointer(&cell);

    assert!(ptr::eq(column.cell(), &cell));
    assert!(column.has_cell());
}

#[test]
fn test_column_size() {
    let column = ColumnInfo::new(O(1), 3, 1, true);
    assert_eq!(column.size(), 1);

    assert_eq!(column.increase_size(), 2);
    assert_eq!(column.size(), 2);

    assert_eq!(column.decrease_size(), 1);
    assert_eq!(column.size(), 1);

    column.increase_size();
    assert_eq!(column.size(), 2);

    column.decrease_size();
    assert_eq!(column.size(), 1);

    column.decrease_size();
    assert_eq!(column.size(), 0);

    assert!(column.empty());

    column.increase_size();
    assert_eq!(column.size(), 1);
}
