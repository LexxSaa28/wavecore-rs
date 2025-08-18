use crate::{GpuDevice, GpuError, GpuResult};
use wavecore_matrices::Matrix;
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

        if self.total_allocated + size > self.pool_size {
            return Err(GpuError::MemoryError {
                message: format!("Not enough memory: requested {} bytes, available {} bytes",
                    size, self.pool_size - self.total_allocated)
            });
        }

        let block = self.allocate_block(size)?;

        Ok(GpuMatrix {
            rows,
            cols,
            data_block: block,
        })
    }

    /// Allocate GPU vector
    pub fn allocate_vector(&mut self, len: usize) -> GpuResult<GpuVector> {
        let size = (len * std::mem::size_of::<f64>()) as u64;

        if self.total_allocated + size > self.pool_size {
            return Err(GpuError::MemoryError {
                message: format!("Not enough memory: requested {} bytes, available {} bytes",
                    size, self.pool_size - self.total_allocated)
            });
        }

        let block = self.allocate_block(size)?;

        Ok(GpuVector {
            len,
            data_block: block,
        })
    }

    /// Upload matrix to GPU
    pub fn upload_matrix(&mut self, matrix: &Matrix) -> GpuResult<GpuMatrix> {
        let rows = matrix.rows;
        let cols = matrix.cols;
        let mut gpu_matrix = self.allocate_matrix(rows, cols)?;
        
        #[cfg(feature = "cuda")]
        {
            if let Some(ref device) = self.device.cuda_device() {
                let matrix_data: Vec<f64> = matrix.data().to_vec();
                
                if let Some(ref mut ptr) = gpu_matrix.data_block.ptr {
                    device.htod_copy(matrix_data.into(), ptr)
                        .map_err(|e| GpuError::MemoryError { 
                            message: format!("Failed to upload matrix: {}", e) 
                        })?;
                }
            }
        }
        
        Ok(gpu_matrix)
    }

    /// Upload vector to GPU
    pub fn upload_vector(&mut self, vector: &Vec<f64>) -> GpuResult<GpuVector> {
        let len = vector.len();
        let gpu_vector = self.allocate_vector(len)?;

        #[cfg(feature = "cuda")]
        {
            if let Some(ref device) = self.device.cuda_device() {
                if let Some(ref mut ptr) = gpu_vector.data_block.ptr {
                    device.htod_copy(vector.clone().into(), ptr)
                        .map_err(|e| GpuError::MemoryError {
                            message: format!("Failed to upload vector: {}", e)
                        })?;
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
                if let Some(ref ptr) = gpu_matrix.data_block.ptr {
                    let data: Vec<f64> = device.dtoh_sync_copy(ptr)
                        .map_err(|e| GpuError::MemoryError {
                            message: format!("Failed to download matrix: {}", e)
                        })?;

                    let matrix = Matrix::from_vec(gpu_matrix.rows, gpu_matrix.cols, data)
                        .map_err(|e| GpuError::MemoryError {
                            message: format!("Failed to create matrix: {}", e)
                        })?;
                    return Ok(matrix);
                }
            }
        }
        
        // Fallback for non-CUDA builds
        Ok(Matrix::new(gpu_matrix.rows, gpu_matrix.cols))
    }

    /// Download vector from GPU
    pub fn download_vector(&self, gpu_vector: &GpuVector) -> GpuResult<Vec<f64>> {
        #[cfg(feature = "cuda")]
        {
            if let Some(ref device) = self.device.cuda_device() {
                if let Some(ref ptr) = gpu_vector.data_block.ptr {
                    let data: Vec<f64> = device.dtoh_sync_copy(ptr)
                        .map_err(|e| GpuError::MemoryError {
                            message: format!("Failed to download vector: {}", e)
                        })?;

                    return Ok(data);
                }
            }
        }
        
        // Fallback for non-CUDA builds
        Ok(vec![0.0; gpu_vector.len])
    }

    /// Deallocate GPU memory block
    pub fn deallocate(&mut self, block_id: usize) -> GpuResult<()> {
        if let Some(block) = self.allocated_blocks.remove(&block_id) {
            self.total_allocated -= block.size;
            // CUDA memory is automatically freed when CudaSlice is dropped
        }
        Ok(())
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> (u64, u64, usize) {
        (self.total_allocated, self.pool_size, self.allocated_blocks.len())
    }

    /// Internal method to allocate memory block
    fn allocate_block(&mut self, size: u64) -> GpuResult<GpuMemoryBlock> {
        let block_id = self.next_block_id;
        self.next_block_id += 1;

        #[cfg(feature = "cuda")]
        let ptr = {
            if let Some(ref device) = self.device.cuda_device() {
                let elements = size / std::mem::size_of::<f64>() as u64;
                match device.alloc_zeros::<f64>(elements as usize) {
                    Ok(slice) => Some(slice),
                    Err(e) => return Err(GpuError::MemoryError {
                        message: format!("CUDA allocation failed: {}", e)
                    }),
                }
            } else {
                None
            }
        };

        #[cfg(not(feature = "cuda"))]
        let ptr: Option<()> = None;

        let block = GpuMemoryBlock {
            id: block_id,
            size,
            #[cfg(feature = "cuda")]
            ptr,
        };

        self.allocated_blocks.insert(block_id, block);
        self.total_allocated += size;

        Ok(self.allocated_blocks.get(&block_id).unwrap().clone())
    }
}

impl Clone for GpuMemoryBlock {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            size: self.size,
            #[cfg(feature = "cuda")]
            ptr: None, // Cannot clone CUDA pointers directly
        }
    }
}

impl GpuMatrix {
    /// Get matrix dimensions
    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Get number of rows
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Get number of columns
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Get total number of elements
    pub fn len(&self) -> usize {
        self.rows * self.cols
    }

    /// Check if matrix is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get mutable access to data block (for kernels)
    pub fn data_mut(&mut self) -> &mut GpuMemoryBlock {
        &mut self.data_block
    }

    /// Get immutable access to data block
    pub fn data(&self) -> &GpuMemoryBlock {
        &self.data_block
    }
}

impl GpuVector {
    /// Get vector length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if vector is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get mutable access to data block (for kernels)
    pub fn data_mut(&mut self) -> &mut GpuMemoryBlock {
        &mut self.data_block
    }

    /// Get immutable access to data block
    pub fn data(&self) -> &GpuMemoryBlock {
        &self.data_block
    }
}

impl Drop for GpuMemoryPool {
    fn drop(&mut self) {
        // Clean up all allocated blocks
        self.allocated_blocks.clear();
        self.total_allocated = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_creation() {
        // This test should work even without GPU
        let device = Arc::new(GpuDevice {
            info: crate::device::DeviceInfo {
                id: 0,
                name: "Test".to_string(),
                total_memory: 1024 * 1024 * 1024,
                free_memory: 1024 * 1024 * 1024,
                compute_capability: (7, 5),
                max_threads_per_block: 1024,
                max_shared_memory: 49152,
                multiprocessor_count: 108,
                clock_rate: 1500000,
            },
            #[cfg(feature = "cuda")]
            cuda_device: None,
        });

        let pool = GpuMemoryPool::new(device, 1024 * 1024);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_memory_stats() {
        let device = Arc::new(GpuDevice {
            info: crate::device::DeviceInfo {
                id: 0,
                name: "Test".to_string(),
                total_memory: 1024 * 1024 * 1024,
                free_memory: 1024 * 1024 * 1024,
                compute_capability: (7, 5),
                max_threads_per_block: 1024,
                max_shared_memory: 49152,
                multiprocessor_count: 108,
                clock_rate: 1500000,
            },
            #[cfg(feature = "cuda")]
            cuda_device: None,
        });

        let pool = GpuMemoryPool::new(device, 1024 * 1024).unwrap();
        let (allocated, total, blocks) = pool.memory_stats();
        assert_eq!(allocated, 0);
        assert_eq!(total, 1024 * 1024);
        assert_eq!(blocks, 0);
    }
}
