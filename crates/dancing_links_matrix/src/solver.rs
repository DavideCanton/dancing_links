use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use log::debug;

use crate::{
    cells::{CellRow, Header, HeaderName, MatrixCell},
    matrix::CellIteratorDirection,
    DancingLinksMatrix,
};

pub struct Solution<T> {
    pub solution_map: HashMap<usize, Vec<T>>,
}

fn cover_row<T: Eq>(matrix: &mut DancingLinksMatrix<T>, row: *mut MatrixCell<T>) {
    matrix
        .iterate_cells(row, CellIteratorDirection::Right, false)
        .map(|v| unsafe { (*v).header })
        .for_each(|j| matrix.cover(j));
}

fn uncover_row<T: Eq>(matrix: &mut DancingLinksMatrix<T>, row: *mut MatrixCell<T>) {
    matrix
        .iterate_cells(row, CellIteratorDirection::Left, false)
        .map(|v| unsafe { (*v).header })
        .for_each(|j| matrix.uncover(j));
}

pub struct IterativeAlgorithmXSolver<T> {
    matrix: DancingLinksMatrix<T>,
    choose_min: bool,
    return_first: bool,
}

#[derive(Clone)]
enum StackElem<T> {
    Root,
    Iteration {
        k: usize,
        current_row: *mut MatrixCell<T>,
        start_row: *mut MatrixCell<T>,
    },
}

impl<T> StackElem<T> {
    fn k(&self) -> usize {
        match self {
            StackElem::Root => 0,
            StackElem::Iteration { k, .. } => *k,
        }
    }
}

impl<T: Eq + Clone> Debug for StackElem<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StackElem::Root => write!(f, "Root"),
            StackElem::Iteration {
                k,
                current_row,
                start_row,
            } => unsafe {
                write!(
                    f,
                    "Iteration({:?} {:?} {:?})",
                    k,
                    (*(*current_row)).index,
                    (*(*start_row)).index
                )
            },
        }
    }
}

impl<T: Eq + Clone> IterativeAlgorithmXSolver<T> {
    pub fn new(matrix: DancingLinksMatrix<T>, choose_min: bool, return_first: bool) -> Self {
        Self {
            matrix,
            choose_min,
            return_first,
        }
    }

    fn create_sol(&self, k: usize, sol_dict: &HashMap<usize, *mut MatrixCell<T>>) -> Solution<T> {
        let mut sol = HashMap::new();

        for (key, row) in sol_dict.iter() {
            if *key >= k {
                continue;
            }

            let mut tmp_list = Vec::new();
            for r in self
                .matrix
                .iterate_cells(*row, CellIteratorDirection::Right, true)
            {
                let name = unsafe { &(*(*r).header).name };
                if let HeaderName::Other(ref name) = name {
                    tmp_list.push(name.clone());
                }
            }

            let row = unsafe { &(*(*row)).row };
            if let CellRow::Data(row) = row {
                sol.insert(*row, tmp_list);
            }
        }

        Solution { solution_map: sol }
    }

    pub fn solve(&mut self) -> Vec<Solution<T>> {
        unsafe { self._solve() }
    }

    unsafe fn _solve(&mut self) -> Vec<Solution<T>> {
        let mut solutions = Vec::new();
        let mut sol_dict = HashMap::new();

        let mut advance = false;

        use StackElem::*;
        let mut stack = vec![Root];

        while let Some(elem) = stack.last().cloned() {
            debug!("elem: {elem:?}, advance: {advance}");
            // trace!("matrix:\n{}", self.matrix);

            let k = elem.k();

            let header = self.matrix.first_header();
            let header_cell = (*header).cell;

            if (*header_cell).right == header_cell {
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

                    uncover_row(&mut self.matrix, current_row);

                    let next_row = (*current_row).down;
                    if next_row == start_row {
                        let col = (*next_row).header;
                        self.matrix.uncover(col);
                        advance = true;
                        continue;
                    } else {
                        stack.push(Iteration {
                            k,
                            current_row: next_row,
                            start_row,
                        });
                        let col = (*start_row).header;
                        add_to_sol(&mut sol_dict, k - 1, next_row, col);
                        advance = false;
                    }
                    next_row
                }
                _ => {
                    let start_col = if self.choose_min {
                        self.matrix.min_column().unwrap()
                    } else {
                        self.matrix.random_column().unwrap()
                    };
                    if (*start_col).size == 0 {
                        advance = true;
                        continue;
                    }

                    let col_cell = (*start_col).cell;

                    self.matrix.cover(start_col);

                    let next_row = (*col_cell).down;
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

            cover_row(&mut self.matrix, next_row);
        }

        solutions
    }
}

unsafe fn add_to_sol<T: Eq>(
    sol_dict: &mut HashMap<usize, *mut MatrixCell<T>>,
    k: usize,
    next_row: *mut MatrixCell<T>,
    current_col: *mut Header<T>,
) {
    let row = (*next_row).row;
    let cur = (*current_col).index;
    debug!("inserting cell {next_row:?} of row {row} at {k}, column = {cur}");

    if cfg!(debug_assertions) {
        let h = (*next_row).header;
        let c = (*h).cell;
        debug_assert!(c != next_row);
    }

    sol_dict.insert(k, next_row);
}
