use std::ptr;

use crate::cells::{CellRow, Header, HeaderName, MatrixCell, ProtoCell, ProtoHeader};

use HeaderName::{First as F, Other as O};

#[test]
fn test_matrix_cell_new() {
    let cell: MatrixCell<()> = MatrixCell::new(42, 3.into());
    assert_eq!(cell.index, 42);
    assert!(!cell.has_up());
    assert!(!cell.has_down());
    assert!(!cell.has_left());
    assert!(!cell.has_right());
    assert!(!cell.has_header());
    assert_eq!(cell.row, 3.into());
}

#[test]
fn test_matrix_cell_update_pointers() {
    let header = Header::new(O(1), 3, 5, true);

    let cell = MatrixCell::new(42, 3.into());
    let up = MatrixCell::new(43, 2.into());
    let down = MatrixCell::new(44, 4.into());
    let left = MatrixCell::new(45, 1.into());
    let right = MatrixCell::new(46, 5.into());

    cell.update_pointers(&up, &down, &left, &right, &header);

    assert!(ptr::eq(cell.up(), &up));
    assert!(ptr::eq(cell.down(), &down));
    assert!(ptr::eq(cell.left(), &left));
    assert!(ptr::eq(cell.right(), &right));
    assert!(ptr::eq(cell.header(), &header));

    assert!(cell.has_up());
    assert!(cell.has_down());
    assert!(cell.has_left());
    assert!(cell.has_right());
    assert!(cell.has_header());
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
    let proto = ProtoHeader::<()>::new(42, F, 3, true);
    assert_eq!(proto.index, 42);
    assert_eq!(proto.name, F);
    assert_eq!(proto.size, 3);
    assert!(proto.primary);

    let proto2 = ProtoHeader::new(42, O(1), 3, false);
    assert_eq!(proto2.name, O(1));
    assert!(!proto2.primary);
}

#[test]
fn test_header_new() {
    let header = Header::new(O(1), 3, 2, true);
    assert_eq!(header.name, O(1));
    assert_eq!(header.size(), 2);
    assert_eq!(header.index, 3);
    assert!(header.primary);
    assert!(!header.is_first());
    assert!(!header.has_cell());

    let header = Header::<u32>::new(F, 10, 8, false);
    assert_eq!(header.name, F);
    assert_eq!(header.size(), 8);
    assert_eq!(header.index, 10);
    assert!(!header.primary);
    assert!(header.is_first());
    assert!(!header.has_cell());
}

#[test]
fn test_header_from_proto() {
    let mut proto = ProtoHeader::new(42, O(1), 3, true);
    let header = Header::from_proto(proto.clone());
    assert_eq!(header.name, O(1));
    assert_eq!(header.size(), 3);
    assert_eq!(header.index, 42);
    assert!(!header.has_cell());
    assert!(header.primary);

    proto.primary = false;
    let header = Header::from_proto(proto);
    assert!(!header.primary);
}

#[test]
fn test_header_update_pointer() {
    let header = Header::<u32>::new(F, 10, 8, true);
    let cell = MatrixCell::new(42, CellRow::Header);

    header.update_pointer(&cell);

    assert!(ptr::eq(header.cell(), &cell));
    assert!(header.has_cell());
}

#[test]
fn test_header_size() {
    let header = Header::new(O(1), 3, 1, true);
    assert_eq!(header.size(), 1);

    assert_eq!(header.increase_size(), 2);
    assert_eq!(header.size(), 2);

    assert_eq!(header.decrease_size(), 1);
    assert_eq!(header.size(), 1);

    header.increase_size();
    assert_eq!(header.size(), 2);

    header.decrease_size();
    assert_eq!(header.size(), 1);

    header.decrease_size();
    assert_eq!(header.size(), 0);

    assert!(header.empty());

    header.increase_size();
    assert_eq!(header.size(), 1);
}
