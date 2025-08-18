//! # WaveCore Matrices Module
//! 
//! High-performance linear algebra and matrix operations for marine hydrodynamics.
//! 
//! This module provides efficient matrix operations, linear solvers, and block matrix
//! functionality optimized for boundary element method (BEM) computations.
//! 
//! ## Features
//! 
//! - **Matrix Operations**: Addition, multiplication, inversion, decomposition
//! - **Linear Solvers**: LU decomposition, GMRES, iterative methods
//! - **Block Matrices**: Efficient large matrix handling
//! - **Parallel Processing**: Multi-threaded computations
//! - **Memory Optimization**: Efficient data structures
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_matrices::{Matrix, LinearSolver, SolverType};
//! 
//! // Create a matrix
//! let matrix = Matrix::from_vec(3, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0])?;
//! 
//! // Solve linear system
//! let solver = LinearSolver::new(SolverType::LU);
//! let b = vec![1.0, 2.0, 3.0];
//! let x = solver.solve(&matrix, &b)?;
//! 
//! println!("Solution: {:?}", x);
//! ```

pub mod operations;
pub mod solvers;
pub mod block;
pub mod types;

pub use operations::*;
pub use solvers::*;
pub use block::*;
pub use types::*;

use thiserror::Error;

/// Error types for matrix operations
#[derive(Error, Debug)]
pub enum MatrixError {
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("Singular matrix encountered")]
    SingularMatrix,
    
    #[error("Invalid matrix dimensions: {rows}x{cols}")]
    InvalidDimensions { rows: usize, cols: usize },
    
    #[error("Linear solver failed: {message}")]
    SolverError { message: String },
    
    #[error("Memory allocation failed")]
    MemoryError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for matrix operations
pub type Result<T> = std::result::Result<T, MatrixError>;

/// Matrix representation optimized for BEM computations
#[derive(Debug, Clone)]
pub struct Matrix {
    /// Number of rows
    pub rows: usize,
    /// Number of columns
    pub cols: usize,
    /// Matrix data stored in row-major order
    pub data: Vec<f64>,
}

impl Matrix {
    /// Create a new matrix with the given dimensions
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }
    
    /// Create a matrix from a vector of data
    pub fn from_vec(rows: usize, cols: usize, data: Vec<f64>) -> Result<Self> {
        if data.len() != rows * cols {
            return Err(MatrixError::DimensionMismatch {
                expected: rows * cols,
                actual: data.len(),
            });
        }
        
        Ok(Self { rows, cols, data })
    }
    
    /// Get element at position (i, j)
    pub fn get(&self, i: usize, j: usize) -> Result<f64> {
        if i >= self.rows || j >= self.cols {
            return Err(MatrixError::InvalidDimensions {
                rows: self.rows,
                cols: self.cols,
            });
        }
        
        Ok(self.data[i * self.cols + j])
    }
    
    /// Set element at position (i, j)
    pub fn set(&mut self, i: usize, j: usize, value: f64) -> Result<()> {
        if i >= self.rows || j >= self.cols {
            return Err(MatrixError::InvalidDimensions {
                rows: self.rows,
                cols: self.cols,
            });
        }
        
        self.data[i * self.cols + j] = value;
        Ok(())
    }
    
    /// Get matrix dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }
    
    /// Check if matrix is square
    pub fn is_square(&self) -> bool {
        self.rows == self.cols
    }
    
    /// Check if matrix is symmetric
    pub fn is_symmetric(&self) -> bool {
        if !self.is_square() {
            return false;
        }
        
        for i in 0..self.rows {
            for j in (i + 1)..self.cols {
                if (self.get(i, j).unwrap() - self.get(j, i).unwrap()).abs() > 1e-10 {
                    return false;
                }
            }
        }
        true
    }
}

/// Linear solver types
#[derive(Debug, Clone, Copy)]
pub enum SolverType {
    /// LU decomposition
    LU,
    /// Cholesky decomposition (for symmetric positive definite matrices)
    Cholesky,
    /// GMRES iterative solver
    GMRES,
    /// Conjugate gradient (for symmetric positive definite matrices)
    ConjugateGradient,
    /// BiCGSTAB iterative solver (for general matrices)
    BiCGSTAB,
}

/// Linear solver interface
pub trait LinearSolverTrait {
    /// Solve the linear system Ax = b
    fn solve(&self, a: &Matrix, b: &[f64]) -> Result<Vec<f64>>;
    
    /// Get solver type
    fn solver_type(&self) -> SolverType;
}

/// Linear solver implementation
pub struct LinearSolver {
    solver_type: SolverType,
}

impl LinearSolver {
    /// Create a new linear solver
    pub fn new(solver_type: SolverType) -> Self {
        Self { solver_type }
    }
    
    /// Get solver type
    pub fn solver_type(&self) -> SolverType {
        self.solver_type
    }
}

impl LinearSolverTrait for LinearSolver {
    fn solve(&self, a: &Matrix, b: &[f64]) -> Result<Vec<f64>> {
        match self.solver_type {
            SolverType::LU => solvers::lu_solve(a, b),
            SolverType::Cholesky => solvers::cholesky_solve(a, b),
            SolverType::GMRES => solvers::gmres_solve(a, b),
            SolverType::ConjugateGradient => solvers::cg_solve(a, b),
            SolverType::BiCGSTAB => solvers::bicgstab_solve(a, b),
        }
    }
    
    fn solver_type(&self) -> SolverType {
        self.solver_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_matrix_creation() {
        let matrix = Matrix::new(3, 3);
        assert_eq!(matrix.dimensions(), (3, 3));
        assert_eq!(matrix.data.len(), 9);
    }
    
    #[test]
    fn test_matrix_from_vec() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let matrix = Matrix::from_vec(2, 3, data.clone()).unwrap();
        assert_eq!(matrix.dimensions(), (2, 3));
        assert_eq!(matrix.data, data);
    }
    
    #[test]
    fn test_matrix_get_set() {
        let mut matrix = Matrix::new(2, 2);
        matrix.set(0, 1, 5.0).unwrap();
        assert_eq!(matrix.get(0, 1).unwrap(), 5.0);
    }
    
    #[test]
    fn test_matrix_symmetry() {
        let mut matrix = Matrix::new(2, 2);
        matrix.set(0, 1, 5.0).unwrap();
        matrix.set(1, 0, 5.0).unwrap();
        assert!(matrix.is_symmetric());
    }
} 