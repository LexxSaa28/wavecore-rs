//! Matrix operations

use super::*;

/// Matrix operations trait
pub trait MatrixOperations {
    /// Add two matrices
    fn add(&self, other: &Matrix) -> Result<Matrix>;
    
    /// Multiply two matrices
    fn multiply(&self, other: &Matrix) -> Result<Matrix>;
    
    /// Transpose matrix
    fn transpose(&self) -> Matrix;
}

impl MatrixOperations for Matrix {
    fn add(&self, other: &Matrix) -> Result<Matrix> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err(MatrixError::DimensionMismatch {
                expected: self.rows * self.cols,
                actual: other.rows * other.cols,
            });
        }
        
        let mut result = Matrix::new(self.rows, self.cols);
        for i in 0..self.data.len() {
            result.data[i] = self.data[i] + other.data[i];
        }
        
        Ok(result)
    }
    
    fn multiply(&self, other: &Matrix) -> Result<Matrix> {
        if self.cols != other.rows {
            return Err(MatrixError::DimensionMismatch {
                expected: self.cols,
                actual: other.rows,
            });
        }
        
        let mut result = Matrix::new(self.rows, other.cols);
        
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self.get(i, k).unwrap() * other.get(k, j).unwrap();
                }
                result.set(i, j, sum).unwrap();
            }
        }
        
        Ok(result)
    }
    
    fn transpose(&self) -> Matrix {
        let mut result = Matrix::new(self.cols, self.rows);
        
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.set(j, i, self.get(i, j).unwrap()).unwrap();
            }
        }
        
        result
    }
} 