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
    assert_eq!(header.size(), 2);
    assert_eq!(header.index, 3);
    assert!(!header.is_first());
    assert!(!header.has_cell());

    let header = Header::<u32>::new(F, 10, 8);
    assert_eq!(header.name, F);
    assert_eq!(header.size(), 8);
    assert_eq!(header.index, 10);
    assert!(header.is_first());
    assert!(!header.has_cell());
}

#[test]
fn test_header_from_proto() {
    let proto = ProtoHeader::new(42, O(1), 3);
    let header = Header::from_proto(proto);
    assert_eq!(header.name, O(1));
    assert_eq!(header.size(), 3);
    assert_eq!(header.index, 42);
    assert!(!header.has_cell());
}
