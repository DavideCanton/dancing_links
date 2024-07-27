use std::ptr::addr_of_mut;

use crate::cells::{CellRow, Header, HeaderName, MatrixCell, ProtoCell, ProtoHeader};

use HeaderName::{First as F, Other as O};

#[test]
fn test_matrix_cell_new() {
    let mut h = Header::new(O(1), 3, 5);
    let ptr = addr_of_mut!(h);

    let cell = MatrixCell::new(42, ptr, CellRow::Data(3));
    assert_eq!(cell.index, 42);
    assert!(cell.up.is_null());
    assert!(cell.down.is_null());
    assert!(cell.left.is_null());
    assert!(cell.right.is_null());
    assert_eq!(cell.header, ptr);
    assert_eq!(cell.row, CellRow::Data(3));
}

#[test]
fn test_matrix_cell_update_pointers() {
    let mut h = Header::new(O(1), 3, 5);
    let ptr = addr_of_mut!(h);

    let cell = MatrixCell::new(42, ptr, CellRow::Data(3));
    let cell_up = MatrixCell::new(43, ptr, CellRow::Data(2));
    let cell_down = MatrixCell::new(44, ptr, CellRow::Data(4));
    let cell_left = MatrixCell::new(45, ptr, CellRow::Data(1));
    let cell_right = MatrixCell::new(46, ptr, CellRow::Data(5));

    let mut cells = vec![cell, cell_up, cell_down, cell_left, cell_right];

    let base = cells.as_mut_ptr();
    cells[0].update_pointers(base, 1, 2, 3, 4);

    unsafe {
        assert_eq!(cells[0].up, base.add(1));
        assert_eq!(cells[0].down, base.add(2));
        assert_eq!(cells[0].left, base.add(3));
        assert_eq!(cells[0].right, base.add(4));

        assert_eq!((*cells[0].up).index, 43);
        assert_eq!((*cells[0].down).index, 44);
        assert_eq!((*cells[0].left).index, 45);
        assert_eq!((*cells[0].right).index, 46);
    }
}

#[test]
fn test_proto_cell_new() {
    let proto = ProtoCell::new(42, 3, CellRow::Header);
    assert_eq!(proto.index, 42);
    assert_eq!(proto.header, 3);
    assert_eq!(proto.row, CellRow::Header);
    assert_eq!(proto.up, 42);
    assert_eq!(proto.down, 42);
    assert_eq!(proto.left, 42);
    assert_eq!(proto.right, 42);
}

#[test]
fn test_proto_header_new() {
    let proto = ProtoHeader::<()>::new(42, F, 3);
    assert_eq!(proto.index, 42);
    assert_eq!(proto.name, F);
    assert_eq!(proto.size, 3);
}

#[test]
fn test_header_new() {
    let header = Header::new(O(1), 3, 2);
    assert_eq!(header.name, O(1));
    assert_eq!(header.size, 2);
    assert_eq!(header.index, 3);
    assert!(!header.is_first());
    assert!(header.cell.is_null());

    let header = Header::<u32>::new(F, 10, 8);
    assert_eq!(header.name, F);
    assert_eq!(header.size, 8);
    assert_eq!(header.index, 10);
    assert!(header.is_first());
    assert!(header.cell.is_null());
}

#[test]
fn test_header_from_proto() {
    let proto = ProtoHeader::new(42, O(1), 3);
    let header = Header::from_proto(proto);
    assert_eq!(header.name, O(1));
    assert_eq!(header.size, 3);
    assert_eq!(header.index, 42);
    assert!(header.cell.is_null());
}

#[test]
fn test_header_update_pointer() {
    let mut header = Header::<u32>::new(F, 10, 8);

    let mut cell = MatrixCell::new(42, addr_of_mut!(header), CellRow::Header);
    let cell_ptr = addr_of_mut!(cell);

    header.update_pointer(cell_ptr);

    assert_eq!(header.cell, cell_ptr);
}
