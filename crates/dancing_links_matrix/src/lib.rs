mod builders;
mod cells;
mod index;
mod matrix;
mod solver;

pub use builders::MatrixBuilder;
pub use matrix::{ColumnSpec, DancingLinksMatrix};
pub use solver::{IterativeAlgorithmXSolver, Solution};

#[cfg(test)]
mod tests;
