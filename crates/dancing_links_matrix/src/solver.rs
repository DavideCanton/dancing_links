use std::{collections::HashMap, fmt::Debug};

use itertools::Itertools;
use log::debug;

use crate::{
    cells::{CellRow, HeaderName},
    keys::{HeaderKey, Key},
    DancingLinksMatrix,
};

pub struct RecursiveAlgorithmXSolver<F, T> {
    matrix: DancingLinksMatrix<T>,
    choose_min: bool,
    callback: F,
    stop: bool,
    solution_found: bool,
}

pub struct Solution<T> {
    pub solution_map: HashMap<usize, Vec<T>>,
}

impl<F: FnMut(&Solution<T>) -> bool, T: Eq + Clone> RecursiveAlgorithmXSolver<F, T> {
    pub fn new(matrix: DancingLinksMatrix<T>, callback: F, choose_min: bool) -> Self {
        Self {
            matrix,
            choose_min,
            callback,
            stop: false,
            solution_found: false,
        }
    }

    pub fn solve(&mut self) -> bool {
        let mut sol_dict = HashMap::new();
        self.search(0, &mut sol_dict);
        self.solution_found
    }

    fn create_sol(&self, k: usize, sol_dict: &HashMap<usize, Key>) -> Solution<T> {
        let mut sol = HashMap::new();

        for (key, row) in sol_dict.iter() {
            if *key >= k {
                continue;
            }

            let mut tmp_list = Vec::new();
            for r in self.matrix.iterate_cells(*row, |c| c.right, true) {
                let r = self.matrix.header(r.header);

                if let HeaderName::Other(ref name) = r.name {
                    tmp_list.push(name.clone());
                }
            }

            let r = self.matrix.cell(*row);
            if let CellRow::Data(row) = r.row {
                sol.insert(row, tmp_list);
            }
        }

        Solution { solution_map: sol }
    }

    fn search(&mut self, k: usize, sol_dict: &mut HashMap<usize, Key>) {
        // trace!("matrix:\n{}", self.matrix);

        let header = self.matrix.header(self.matrix.header_key);
        let header_cell = self.matrix.cell(header.cell);

        if header_cell.right == header_cell.key {
            self.solution_found = true;
            let sol = self.create_sol(k, sol_dict);

            if (self.callback)(&sol) {
                self.stop = true;
            }

            return;
        }

        let start_col = if self.choose_min {
            self.matrix.min_column().unwrap()
        } else {
            self.matrix.random_column().unwrap()
        };

        let col_cell = start_col.cell;
        let col = start_col.key;

        self.matrix.cover(col);

        let rows: Vec<_> = self
            .matrix
            .iterate_cells(col_cell, |cell| cell.down, false)
            .map(|v| v.key)
            .collect();

        if rows.is_empty() {
            debug!("rows is empty");
        }

        for row in rows {
            add_to_sol(&self.matrix, sol_dict, k, row, col);

            cover_row(&mut self.matrix, row);

            self.search(k + 1, sol_dict);

            if self.stop {
                return;
            }

            uncover_row(&mut self.matrix, row);
        }

        self.matrix.uncover(col)
    }
}

fn cover_row<T: Eq>(matrix: &mut DancingLinksMatrix<T>, row: Key) {
    matrix
        .iterate_cells(row, |cell| cell.right, false)
        .map(|v| v.header)
        .collect_vec()
        .into_iter()
        .for_each(|j| matrix.cover(j));
}

fn uncover_row<T: Eq>(matrix: &mut DancingLinksMatrix<T>, row: Key) {
    matrix
        .iterate_cells(row, |c| c.left, false)
        .map(|v| v.header)
        .collect_vec()
        .into_iter()
        .for_each(|j| matrix.uncover(j));
}

pub struct IterativeAlgorithmXSolver<T> {
    matrix: DancingLinksMatrix<T>,
    choose_min: bool,
    return_first: bool,
}

#[derive(Debug, Clone)]
enum StackElem {
    Root,
    Iteration {
        k: usize,
        current_row: Key,
        start_row: Key,
    },
}

impl StackElem {
    fn k(&self) -> usize {
        match self {
            StackElem::Root => 0,
            StackElem::Iteration { k, .. } => *k,
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

    fn create_sol(&self, k: usize, sol_dict: &HashMap<usize, Key>) -> Solution<T> {
        let mut sol = HashMap::new();

        for (key, row) in sol_dict.iter() {
            if *key >= k {
                continue;
            }

            let mut tmp_list = Vec::new();
            for r in self.matrix.iterate_cells(*row, |c| c.right, true) {
                let r = self.matrix.header(r.header);

                if let HeaderName::Other(ref name) = r.name {
                    tmp_list.push(name.clone());
                }
            }

            let r = self.matrix.cell(*row);
            if let CellRow::Data(row) = r.row {
                sol.insert(row, tmp_list);
            }
        }

        Solution { solution_map: sol }
    }

    pub fn solve(&mut self) -> Vec<Solution<T>> {
        let mut solutions = Vec::new();
        let mut sol_dict = HashMap::new();

        let mut advance = false;

        use StackElem::*;
        let mut stack = vec![Root];

        while let Some(elem) = stack.last().cloned() {
            debug!("elem: {elem:?}, advance: {advance}");
            // trace!("matrix:\n{}", self.matrix);

            let k = elem.k();

            let header = self.matrix.header(self.matrix.header_key);
            let header_cell = self.matrix.cell(header.cell);

            if header_cell.right == header_cell.key {
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

                    let next_row = self.matrix.cell(current_row).down;
                    if next_row == start_row {
                        let col = self.matrix.cell(next_row).header;
                        self.matrix.uncover(col);
                        advance = true;
                        continue;
                    } else {
                        stack.push(Iteration {
                            k,
                            current_row: next_row,
                            start_row,
                        });
                        let col = self.matrix.cell(start_row).header;
                        add_to_sol(&self.matrix, &mut sol_dict, k - 1, next_row, col);
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
                    if start_col.size == 0 {
                        advance = true;
                        continue;
                    }

                    let col_cell = start_col.cell;
                    let col = start_col.key;

                    self.matrix.cover(col);

                    let next_row = self.matrix.cell(col_cell).down;
                    stack.push(Iteration {
                        k: k + 1,
                        current_row: next_row,
                        start_row: col_cell,
                    });
                    advance = false;
                    add_to_sol(&self.matrix, &mut sol_dict, k, next_row, col);

                    next_row
                }
            };

            cover_row(&mut self.matrix, next_row);
        }

        solutions
    }
}

fn add_to_sol<T: Eq>(
    matrix: &DancingLinksMatrix<T>,
    sol_dict: &mut HashMap<usize, Key>,
    k: usize,
    next_row: Key,
    current_col: HeaderKey,
) {
    let row = matrix.cell(next_row).row;
    debug!("inserting cell {next_row:?} of row {row} at {k}, column = {current_col}");

    if cfg!(debug_assertions) {
        let h = matrix.cell(next_row).header;
        let c = matrix.header(h).cell;
        debug_assert!(c != next_row);
    }

    sol_dict.insert(k, next_row);
}
