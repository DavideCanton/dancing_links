use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    cells::{CellRow, HeaderName},
    keys::Key,
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

    fn create_sol(&self, k: u32, sol_dict: &HashMap<u32, Key>) -> Solution<T> {
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

    fn search(&mut self, k: u32, sol_dict: &mut HashMap<u32, Key>) {
        let header = self.matrix.header(self.matrix.header_key);
        let header_cell = self.matrix.cell(header.cell);

        if header_cell.right == header_cell.index {
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
        let mut col = start_col.index;

        self.matrix.cover(col);

        let rows: Vec<_> = self
            .matrix
            .iterate_cells(col_cell, |cell| cell.down, false)
            .map(|v| v.index)
            .collect();

        for row in rows {
            sol_dict.insert(k, row);

            self.matrix
                .iterate_cells(row, |cell| cell.right, false)
                .map(|v| v.header)
                .collect_vec()
                .into_iter()
                .for_each(|j| self.matrix.cover(j));

            self.search(k + 1, sol_dict);

            if self.stop {
                return;
            }

            col = self.matrix.cell(row).header;

            self.matrix
                .iterate_cells(row, |c| c.left, false)
                .map(|v| v.header)
                .collect_vec()
                .into_iter()
                .for_each(|j| self.matrix.uncover(j));
        }

        self.matrix.uncover(col)
    }
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
            let k = elem.k();

            let header = self.matrix.header(self.matrix.header_key);
            let header_cell = self.matrix.cell(header.cell);

            if header_cell.right == header_cell.index {
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

                    self.matrix
                        .iterate_cells(current_row, |c| c.left, false)
                        .map(|v| v.header)
                        .collect_vec()
                        .into_iter()
                        .for_each(|j| self.matrix.uncover(j));

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
                        sol_dict.insert(k - 1, next_row);
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
                    let col_cell = start_col.cell;
                    let col = start_col.index;

                    self.matrix.cover(col);

                    let next_row = self.matrix.cell(col_cell).down;
                    stack.push(Iteration {
                        k: k + 1,
                        current_row: next_row,
                        start_row: col_cell,
                    });
                    advance = false;
                    sol_dict.insert(k, next_row);

                    next_row
                }
            };

            self.matrix
                .iterate_cells(next_row, |cell| cell.right, false)
                .map(|v| v.header)
                .collect_vec()
                .into_iter()
                .for_each(|j| self.matrix.cover(j));
        }

        solutions
    }
}
