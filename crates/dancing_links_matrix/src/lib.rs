mod allocator;
mod builders;
mod cells;
mod keys;
mod matrix;
mod solver;

pub use builders::MatrixBuilder;
pub use matrix::{ColumnSpec, DancingLinksMatrix};
pub use solver::{IterativeAlgorithmXSolver, RecursiveAlgorithmXSolver, Solution};

#[cfg(test)]
mod tests;
