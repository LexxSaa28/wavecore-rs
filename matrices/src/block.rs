//! Block matrix operations

use super::*;

/// Block matrix representation
pub struct BlockMatrix {
    pub blocks: Vec<Vec<Matrix>>,
    pub block_rows: usize,
    pub block_cols: usize,
}

impl BlockMatrix {
    /// Create a new block matrix
    pub fn new(block_rows: usize, block_cols: usize) -> Self {
        Self {
            blocks: vec![vec![Matrix::new(0, 0); block_cols]; block_rows],
            block_rows,
            block_cols,
        }
    }
    
    /// Set a block in the matrix
    pub fn set_block(&mut self, i: usize, j: usize, block: Matrix) -> Result<()> {
        if i >= self.block_rows || j >= self.block_cols {
            return Err(MatrixError::InvalidDimensions {
                rows: self.block_rows,
                cols: self.block_cols,
            });
        }
        
        self.blocks[i][j] = block;
        Ok(())
    }
    
    /// Get a block from the matrix
    pub fn get_block(&self, i: usize, j: usize) -> Result<&Matrix> {
        if i >= self.block_rows || j >= self.block_cols {
            return Err(MatrixError::InvalidDimensions {
                rows: self.block_rows,
                cols: self.block_cols,
            });
        }
        
        Ok(&self.blocks[i][j])
    }
} 