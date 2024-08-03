use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    ptr,
};

use log::debug;

use crate::{
    cells::{CellRow, ColumnName, ColumnRef, MatrixCellRef},
    matrix::CellIteratorDir,
    DancingLinksMatrix,
};

pub struct Solution<'a, T> {
    pub solution_map: HashMap<usize, Vec<&'a T>>,
}

fn cover_row<'a, T>(matrix: &DancingLinksMatrix<'a, T>, row: MatrixCellRef<'a, T>) {
    for j in matrix.iterate_cells(row, CellIteratorDir::Right, false) {
        matrix.cover(j.column())
    }
}

fn uncover_row<'a, T>(matrix: &DancingLinksMatrix<'a, T>, row: MatrixCellRef<'a, T>) {
    for j in matrix.iterate_cells(row, CellIteratorDir::Left, false) {
        matrix.uncover(j.column())
    }
}

pub struct IterativeAlgorithmXSolver<'a, T> {
    matrix: DancingLinksMatrix<'a, T>,
    choose_min: bool,
    return_first: bool,
}

enum StackElem<'a, T> {
    Root,
    Iteration {
        k: usize,
        current_row: MatrixCellRef<'a, T>,
        start_row: MatrixCellRef<'a, T>,
    },
}

impl<T> Copy for StackElem<'_, T> {}

impl<T> Clone for StackElem<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> StackElem<'a, T> {
    fn k(&self) -> usize {
        match self {
            StackElem::Root => 0,
            StackElem::Iteration { k, .. } => *k,
        }
    }
}

impl<'a, T> Debug for StackElem<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StackElem::Root => write!(f, "Root"),
            StackElem::Iteration {
                k,
                current_row,
                start_row,
            } => write!(
                f,
                "Iteration({:?} {:?} {:?})",
                k, current_row.index, start_row.index
            ),
        }
    }
}

impl<'a, T> IterativeAlgorithmXSolver<'a, T> {
    pub fn new(matrix: DancingLinksMatrix<'a, T>, choose_min: bool, return_first: bool) -> Self {
        Self {
            matrix,
            choose_min,
            return_first,
        }
    }

    fn create_sol(
        &self,
        k: usize,
        sol_dict: &hashbrown::HashMap<usize, MatrixCellRef<'a, T>>,
    ) -> Solution<T> {
        let mut sol = HashMap::new();

        for (key, row) in sol_dict.iter() {
            if *key >= k {
                continue;
            }

            let mut tmp_list = Vec::new();
            for r in self
                .matrix
                .iterate_cells(*row, CellIteratorDir::Right, true)
            {
                if let ColumnName::Other(ref name) = r.name() {
                    tmp_list.push(name);
                }
            }

            if let CellRow::Data(row) = row.row {
                sol.insert(row.into(), tmp_list);
            }
        }

        Solution { solution_map: sol }
    }

    pub fn solve(&self) -> Vec<Solution<T>> {
        let mut solutions = Vec::new();
        let mut sol_dict = hashbrown::HashMap::new();

        let mut advance = false;

        use StackElem::*;
        let mut stack = vec![Root];

        while let Some(elem) = stack.last().copied() {
            debug!("elem: {elem:?}, advance: {advance}");
            // trace!("matrix:\n{}", self.matrix);

            let k = elem.k();

            let column = self.matrix.first_column();
            let column_cell = column.cell();

            if ptr::eq(column_cell.right(), column_cell) {
                let sol = self.create_sol(k, &sol_dict);
                solutions.push(sol);
                if self.return_first {
                    return solutions;
                }
                advance = true;
            }

            let next_row = match elem {
                Root if advance => {
                    stack.pop();
                    continue;
                }
                Iteration {
                    current_row,
                    start_row,
                    ..
                } if advance => {
                    stack.pop();

                    uncover_row(&self.matrix, current_row);

                    let next_row = current_row.down();
                    if ptr::eq(next_row, start_row) {
                        let col = next_row.column();
                        self.matrix.uncover(col);
                        advance = true;
                        continue;
                    } else {
                        stack.push(Iteration {
                            k,
                            current_row: next_row,
                            start_row,
                        });
                        let col = start_row.column();
                        add_to_sol(&mut sol_dict, k - 1, next_row, col);
                        advance = false;
                    }
                    next_row
                }
                _ => {
                    let start_col = if self.choose_min {
                        self.matrix.min_column()
                    } else {
                        self.matrix.random_column()
                    };
                    if start_col.empty() {
                        advance = true;
                        continue;
                    }

                    let col_cell = start_col.cell();

                    self.matrix.cover(start_col);

                    let next_row = col_cell.down();
                    stack.push(Iteration {
                        k: k + 1,
                        current_row: next_row,
                        start_row: col_cell,
                    });
                    advance = false;
                    add_to_sol(&mut sol_dict, k, next_row, start_col);

                    next_row
                }
            };

            cover_row(&self.matrix, next_row);
        }

        solutions
    }
}

fn add_to_sol<'a, T>(
    sol_dict: &mut hashbrown::HashMap<usize, MatrixCellRef<'a, T>>,
    k: usize,
    next_row: MatrixCellRef<'a, T>,
    current_col: ColumnRef<'a, T>,
) {
    let row = next_row.row;
    let cur = current_col.index;
    debug!(
        "inserting cell {} of row {row} at {k}, column = {cur}",
        next_row.index
    );

    if cfg!(debug_assertions) {
        debug_assert!(next_row.column().cell().index != next_row.index);
    }

    sol_dict.insert(k, next_row);
}
