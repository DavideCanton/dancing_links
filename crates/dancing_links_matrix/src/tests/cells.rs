use std::ptr::addr_of_mut;

use crate::cells::{CellRow, HeaderCell, HeaderName, MatrixCell, ProtoHeaderCell};

use HeaderName::{First as F, Other as O};

#[test]
fn test_cell_new() {
    let mut h = HeaderCell::new(O(1), 3, 5);
    let ptr = addr_of_mut!(h);

    let cell = MatrixCell::new(42, ptr, CellRow::Data(3));
    assert_eq!(cell.key, 42);
    assert!(cell.up.is_null());
    assert!(cell.down.is_null());
    assert!(cell.left.is_null());
    assert!(cell.right.is_null());
    assert_eq!(cell.header, ptr);
    assert_eq!(cell.row, CellRow::Data(3));
}

#[test]
fn test_header_cell_new() {
    let header_cell = HeaderCell::new(O(1), 3, 2);
    assert_eq!(header_cell.name, O(1));
    assert_eq!(header_cell.size, 2);
    assert_eq!(header_cell.key, 3);
    assert!(!header_cell.is_first());
    assert!(header_cell.cell.is_null());

    let header_cell = HeaderCell::<u32>::new(F, 10, 8);
    assert_eq!(header_cell.name, F);
    assert_eq!(header_cell.size, 8);
    assert_eq!(header_cell.key, 10);
    assert!(header_cell.is_first());
    assert!(header_cell.cell.is_null());
}

#[test]
fn test_header_cell_from_proto() {
    let proto = ProtoHeaderCell {
        key: 42,
        name: O(1),
        size: 3,
        cell: 55,
    };
    let header_cell = HeaderCell::from_proto(proto);
    assert_eq!(header_cell.name, O(1));
    assert_eq!(header_cell.size, 3);
    assert_eq!(header_cell.key, 42);

    let cell_ptr = header_cell.cell as usize;
    assert_eq!(cell_ptr, 55);
}
