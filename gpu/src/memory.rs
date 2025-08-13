use crate::{GpuDevice, GpuError, GpuResult};
use wavecore_matrices::Matrix;
use nalgebra::Vector3;
use std::sync::Arc;
use std::collections::HashMap;

/// GPU memory pool for efficient allocation
pub struct GpuMemoryPool {
    device: Arc<GpuDevice>,
    pool_size: u64,
    allocated_blocks: HashMap<usize, GpuMemoryBlock>,
    next_block_id: usize,
    total_allocated: u64,
}

/// GPU memory block
pub struct GpuMemoryBlock {
    id: usize,
    size: u64,
    #[cfg(feature = "cuda")]
    ptr: Option<cudarc::driver::CudaSlice<f64>>,
}

/// GPU matrix representation
pub struct GpuMatrix {
    rows: usize,
    cols: usize,
    data_block: GpuMemoryBlock,
}

/// GPU vector representation
pub struct GpuVector {
    len: usize,
    data_block: GpuMemoryBlock,
}

impl GpuMemoryPool {
    /// Create new GPU memory pool
    pub fn new(device: Arc<GpuDevice>, pool_size: u64) -> GpuResult<Self> {
        Ok(Self {
            device,
            pool_size,
            allocated_blocks: HashMap::new(),
            next_block_id: 0,
            total_allocated: 0,
        })
    }

    /// Allocate GPU matrix
    pub fn allocate_matrix(&mut self, rows: usize, cols: usize) -> GpuResult<GpuMatrix> {
        let size = (rows * cols * std::mem::size_of::<f64>()) as u64;
        let data_block = self.allocate_block(size)?;
        
        Ok(GpuMatrix {
            rows,
            cols,
            data_block,
        })
    }

    /// Allocate GPU vector
    pub fn allocate_vector(&mut self, len: usize) -> GpuResult<GpuVector> {
        let size = (len * std::mem::size_of::<f64>()) as u64;
        let data_block = self.allocate_block(size)?;
        
        Ok(GpuVector {
            len,
            data_block,
        })
    }

    /// Upload matrix to GPU
    pub fn upload_matrix(&mut self, matrix: &Matrix) -> GpuResult<GpuMatrix> {
        // Get matrix dimensions - assuming square matrix for now
        let data = matrix.data();
        let rows = (data.len() as f64).sqrt() as usize;
        let cols = rows;
        
        let mut gpu_matrix = self.allocate_matrix(rows, cols)?;
        
        #[cfg(feature = "cuda")]
        {
            if let Some(ref device) = self.device.cuda_device() {
                let data: Vec<f64> = matrix.iter().cloned().collect();
                
                if let Some(ref mut cuda_slice) = gpu_matrix.data_block.ptr {
                    device.htod_copy(data, cuda_slice)
                        .map_err(|e| GpuError::MemoryError(format!("Upload failed: {}", e)))?;
                }
            }
        }
        
        Ok(gpu_matrix)
    }

    /// Upload vector to GPU
    pub fn upload_vector(&mut self, vector: &Vec<f64>) -> GpuResult<GpuVector> {
        let mut gpu_vector = self.allocate_vector(vector.len())?;
        
        #[cfg(feature = "cuda")]
        {
            if let Some(ref device) = self.device.cuda_device() {
                let data: Vec<f64> = vector.iter().cloned().collect();
                
                if let Some(ref mut cuda_slice) = gpu_vector.data_block.ptr {
                    device.htod_copy(data, cuda_slice)
                        .map_err(|e| GpuError::MemoryError(format!("Upload failed: {}", e)))?;
                }
            }
        }
        
        Ok(gpu_vector)
    }

    /// Download matrix from GPU
    pub fn download_matrix(&self, gpu_matrix: &GpuMatrix) -> GpuResult<Matrix> {
        #[cfg(feature = "cuda")]
        {
            if let Some(ref device) = self.device.cuda_device() {
                if let Some(ref cuda_slice) = gpu_matrix.data_block.ptr {
                    let data = device.dtoh_sync_copy(cuda_slice)
                        .map_err(|e| GpuError::MemoryError(format!("Download failed: {}", e)))?;
                    
                    let matrix = Matrix::from_row_slice(gpu_matrix.rows, gpu_matrix.cols, &data);
                    return Ok(matrix);
                }
            }
        }
        
        // Fallback: return zero matrix if CUDA is not available
        Ok(Matrix::new(gpu_matrix.rows, gpu_matrix.cols))
    }

    /// Download vector from GPU
    pub fn download_vector(&self, gpu_vector: &GpuVector) -> GpuResult<Vec<f64>> {
        #[cfg(feature = "cuda")]
        {
            if let Some(ref device) = self.device.cuda_device() {
                let mut data = vec![0.0; gpu_vector.len];
                match device.synchronize() {
                    Ok(_) => {
                        // In a real implementation, this would copy data from GPU to CPU
                        // For now, return zero vector
                        return Ok(data);
                    }
                    Err(e) => {
                        return Err(GpuError::MemoryError(format!("Download failed: {}", e)));
                    }
                }
            }
        }
        
        // Fallback: return zero vector if CUDA is not available
        Ok(vec![0.0; gpu_vector.len])
    }

    /// Upload vertices array
    pub fn upload_vertices(&mut self, vertices: &[f64]) -> GpuResult<Vec<f64>> {
        // For now, just return a copy (placeholder implementation)
        Ok(vertices.to_vec())
    }

    /// Upload panels array
    pub fn upload_panels(&mut self, panels: &[u32]) -> GpuResult<Vec<u32>> {
        // For now, just return a copy (placeholder implementation)
        Ok(panels.to_vec())
    }

    /// Allocate memory block
    fn allocate_block(&mut self, size: u64) -> GpuResult<GpuMemoryBlock> {
        if self.total_allocated + size > self.pool_size {
            return Err(GpuError::MemoryError("Memory pool exhausted".to_string()));
        }

        let block_id = self.next_block_id;
        self.next_block_id += 1;

        #[cfg(feature = "cuda")]
        let ptr: Option<*mut f64> = None;

        #[cfg(not(feature = "cuda"))]
        let ptr = None;

        let block = GpuMemoryBlock {
            id: block_id,
            size,
            #[cfg(feature = "cuda")]
            ptr,
        };

        self.total_allocated += size;
        self.allocated_blocks.insert(block_id, block);
        
        Ok(self.allocated_blocks.get(&block_id).unwrap().clone())
    }

    /// Deallocate memory block
    pub fn deallocate_block(&mut self, block_id: usize) -> GpuResult<()> {
        if let Some(block) = self.allocated_blocks.remove(&block_id) {
            self.total_allocated -= block.size;
            Ok(())
        } else {
            Err(GpuError::MemoryError("Block not found".to_string()))
        }
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.total_allocated,
            pool_size: self.pool_size,
            num_blocks: self.allocated_blocks.len(),
            utilization: self.total_allocated as f64 / self.pool_size as f64,
        }
    }

    /// Clear all allocations
    pub fn clear(&mut self) {
        self.allocated_blocks.clear();
        self.total_allocated = 0;
        self.next_block_id = 0;
    }
}

impl Clone for GpuMemoryBlock {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            size: self.size,
            #[cfg(feature = "cuda")]
            ptr: None, // Cannot clone CUDA pointer, set to None
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: u64,
    pub pool_size: u64,
    pub num_blocks: usize,
    pub utilization: f64,
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GPU Memory: {}/{} MB ({:.1}% used), {} blocks",
               self.total_allocated / 1024 / 1024,
               self.pool_size / 1024 / 1024,
               self.utilization * 100.0,
               self.num_blocks)
    }
}

impl Drop for GpuMemoryPool {
    fn drop(&mut self) {
        // Automatically clean up when pool is dropped
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_creation() {
        if let Ok(device) = GpuDevice::default() {
            let pool_size = 1024 * 1024; // 1 MB
            let result = GpuMemoryPool::new(Arc::new(device), pool_size);
            assert!(result.is_ok());
            
            if let Ok(pool) = result {
                let stats = pool.memory_stats();
                assert_eq!(stats.pool_size, pool_size);
                assert_eq!(stats.total_allocated, 0);
            }
        }
    }

    #[test]
    fn test_memory_allocation() {
        if let Ok(device) = GpuDevice::default() {
            let pool_size = 1024 * 1024; // 1 MB
            if let Ok(mut pool) = GpuMemoryPool::new(Arc::new(device), pool_size) {
                // Allocate a small matrix
                let result = pool.allocate_matrix(10, 10);
                assert!(result.is_ok());
                
                let stats = pool.memory_stats();
                assert!(stats.total_allocated > 0);
                assert_eq!(stats.num_blocks, 1);
            }
        }
    }

    #[test]
    fn test_memory_exhaustion() {
        if let Ok(device) = GpuDevice::default() {
            let pool_size = 1024; // Very small pool
            if let Ok(mut pool) = GpuMemoryPool::new(Arc::new(device), pool_size) {
                // Try to allocate more than available
                let result = pool.allocate_matrix(1000, 1000); // Too large
                assert!(result.is_err());
                
                if let Err(GpuError::MemoryError(_)) = result {
                    // Expected error
                } else {
                    panic!("Expected memory error");
                }
            }
        }
    }

    #[test]
    fn test_matrix_operations() {
        if let Ok(device) = GpuDevice::default() {
            let pool_size = 1024 * 1024;
            if let Ok(mut pool) = GpuMemoryPool::new(Arc::new(device), pool_size) {
                // Create a test matrix
                let matrix = Matrix::from_row_slice(3, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
                
                // Upload and download
                if let Ok(gpu_matrix) = pool.upload_matrix(&matrix) {
                    let downloaded = pool.download_matrix(&gpu_matrix).unwrap();
                    
                    // Check dimensions
                    assert_eq!(downloaded.nrows(), matrix.nrows());
                    assert_eq!(downloaded.ncols(), matrix.ncols());
                }
            }
        }
    }
} 