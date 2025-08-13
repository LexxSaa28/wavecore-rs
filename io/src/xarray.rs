//! Data array operations

use super::*;
use serde::{Serialize, Deserialize};

/// Data array (XArray-like functionality)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataArray {
    pub shape: Vec<usize>,
    pub data: Vec<f64>,
    pub data_type: DataType,
}

impl DataArray {
    /// Create a new data array
    pub fn new(shape: &[usize], data: &[f64]) -> Result<Self> {
        let expected_size: usize = shape.iter().product();
        if data.len() != expected_size {
            return Err(IOError::DataArrayError {
                message: format!("Data size {} does not match shape {:?}", data.len(), shape),
            });
        }
        
        Ok(Self {
            shape: shape.to_vec(),
            data: data.to_vec(),
            data_type: DataType::Float64,
        })
    }
    
    /// Get array dimensions
    pub fn dimensions(&self) -> &[usize] {
        &self.shape
    }
    
    /// Get array size
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Get data as slice
    pub fn as_slice(&self) -> &[f64] {
        &self.data
    }
    
    /// Get data as mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [f64] {
        &mut self.data
    }
    
    /// Get value at index
    pub fn get(&self, index: usize) -> Option<f64> {
        self.data.get(index).copied()
    }
    
    /// Set value at index
    pub fn set(&mut self, index: usize, value: f64) -> Result<()> {
        if index >= self.data.len() {
            return Err(IOError::DataArrayError {
                message: format!("Index {} out of bounds for array of size {}", index, self.data.len()),
            });
        }
        self.data[index] = value;
        Ok(())
    }
    
    /// Reshape array
    pub fn reshape(&mut self, new_shape: &[usize]) -> Result<()> {
        let expected_size: usize = new_shape.iter().product();
        if self.data.len() != expected_size {
            return Err(IOError::DataArrayError {
                message: format!("Cannot reshape array of size {} to shape {:?}", self.data.len(), new_shape),
            });
        }
        self.shape = new_shape.to_vec();
        Ok(())
    }
    
    /// Create zero array
    pub fn zeros(shape: &[usize]) -> Self {
        let size: usize = shape.iter().product();
        Self {
            shape: shape.to_vec(),
            data: vec![0.0; size],
            data_type: DataType::Float64,
        }
    }
    
    /// Create ones array
    pub fn ones(shape: &[usize]) -> Self {
        let size: usize = shape.iter().product();
        Self {
            shape: shape.to_vec(),
            data: vec![1.0; size],
            data_type: DataType::Float64,
        }
    }
} 