use crate::cells::{CellRow, Header, HeaderName, MatrixCell, ProtoCell, ProtoHeader};

use HeaderName::{First as F, Other as O};

#[test]
fn test_matrix_cell_new() {
    let cell: MatrixCell<()> = MatrixCell::new(42, 3.into());
    assert_eq!(cell.index, 42);
    assert!(cell.up.get().is_none());
    assert!(cell.down.get().is_none());
    assert!(cell.left.get().is_none());
    assert!(cell.right.get().is_none());
    assert!(cell.header.get().is_none());
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
    assert_eq!(header.size.get(), 2);
    assert_eq!(header.index, 3);
    assert!(!header.is_first());
    assert!(header.cell.get().is_none());

    let header = Header::<u32>::new(F, 10, 8);
    assert_eq!(header.name, F);
    assert_eq!(header.size.get(), 8);
    assert_eq!(header.index, 10);
    assert!(header.is_first());
    assert!(header.cell.get().is_none());
}

#[test]
fn test_header_from_proto() {
    let proto = ProtoHeader::new(42, O(1), 3);
    let header = Header::from_proto(proto);
    assert_eq!(header.name, O(1));
    assert_eq!(header.size.get(), 3);
    assert_eq!(header.index, 42);
    assert!(header.cell.get().is_none());
}
