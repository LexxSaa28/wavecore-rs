//! Matrix types and traits

use super::*;

/// Matrix type trait
pub trait MatrixType {
    /// Get matrix type name
    fn type_name(&self) -> &'static str;
    
    /// Check if matrix is sparse
    fn is_sparse(&self) -> bool;
    
    /// Check if matrix is dense
    fn is_dense(&self) -> bool;
}

impl MatrixType for Matrix {
    fn type_name(&self) -> &'static str {
        "DenseMatrix"
    }
    
    fn is_sparse(&self) -> bool {
        false
    }
    
    fn is_dense(&self) -> bool {
        true
    }
}

/// Matrix format trait
pub trait MatrixFormat {
    /// Get matrix format
    fn format(&self) -> &'static str;
}

impl MatrixFormat for Matrix {
    fn format(&self) -> &'static str {
        "RowMajor"
    }
} 