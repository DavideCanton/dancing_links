use crate::cells::{MatrixCell, CellRow, HeaderCell, HeaderName};

use HeaderName::{First as F, Other as O};

// #[test]
// fn test_cell_new() {
//     let cell = MatrixCell::new(42.into(), 2.into(), CellRow::Data(3));
//     assert_eq!(cell.key, 42.into());
//     assert_eq!(cell.up, 42.into());
//     assert_eq!(cell.down, 42.into());
//     assert_eq!(cell.left, 42.into());
//     assert_eq!(cell.right, 42.into());
//     assert_eq!(cell.header, 2.into());
//     assert_eq!(cell.row, CellRow::Data(3));
// }

// #[test]
// fn test_header_cell_new() {
//     let header_cell = HeaderCell::new(O(1), 3.into(), 2.into());
//     assert_eq!(header_cell.name, O(1));
//     assert_eq!(header_cell.size, 0);
//     assert_eq!(header_cell.key, 3.into());
//     assert!(!header_cell.is_first());
//     assert_eq!(header_cell.cell, 2.into());

//     let header_cell = HeaderCell::<u32>::new(F, 10.into(), 12.into());
//     assert_eq!(header_cell.name, F);
//     assert_eq!(header_cell.size, 0);
//     assert_eq!(header_cell.key, 10.into());
//     assert!(header_cell.is_first());
//     assert_eq!(header_cell.cell, 12.into());
// }
