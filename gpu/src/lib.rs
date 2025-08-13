//! # WaveCore GPU Acceleration Module
//! 
//! High-performance GPU computing for marine hydrodynamics.
//! 
//! This module provides GPU acceleration for boundary element method computations,
//! including CUDA integration, GPU-optimized linear algebra, and efficient
//! memory management with CPU fallback support.
//! 
//! ## Features
//! 
//! - **CUDA Integration**: High-performance GPU computing with CUDA
//! - **Matrix Operations**: GPU-accelerated BEM matrix assembly and solving
//! - **Memory Management**: Efficient GPU memory allocation and transfer
//! - **CPU Fallback**: Automatic fallback to CPU when GPU is unavailable
//! - **Performance Monitoring**: Benchmarking and profiling tools
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_gpu::{GpuBemSolver, GpuDevice};
//! 
//! // Initialize GPU device
//! let device = GpuDevice::new(0)?;
//! 
//! // Create GPU BEM solver
//! let solver = GpuBemSolver::new(device)?;
//! 
//! // Solve BEM problem on GPU
//! let solution = solver.solve_gpu(&mesh, &green_function)?;
//! ```

pub mod device;
pub mod solver;
pub mod memory;
pub mod kernels;
pub mod fallback;

use thiserror::Error;

/// GPU computation errors
#[derive(Error, Debug)]
pub enum GpuError {
    #[error("GPU device error: {message}")]
    DeviceError { message: String },
    
    #[error("Memory allocation failed: {message}")]
    MemoryError { message: String },
    
    #[error("Kernel execution failed: {message}")]
    KernelError { message: String },
    
    #[error("Computation error: {message}")]
    ComputationError { message: String },
    
    #[error("Mesh error: {message}")]
    MeshError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    #[error("CUDA error: {0}")]
    CudaError(String),
    
    #[error("Feature not available: {feature}")]
    FeatureNotAvailable { feature: String },
}

pub type GpuResult<T> = Result<T, GpuError>;

// Re-export main types
pub use device::{GpuDevice, DeviceInfo};
pub use solver::{GpuBemSolver, GpuSolverConfig};
pub use memory::{GpuMemoryPool, GpuMatrix, GpuVector};
pub use kernels::{GpuKernels, KernelType};
pub use fallback::{CpuFallback, CpuFallbackConfig};

/// GPU acceleration capabilities
#[derive(Debug, Clone)]
pub struct GpuCapabilities {
    pub cuda_available: bool,
    pub device_count: usize,
    pub total_memory: u64,
    pub compute_capability: (u32, u32),
    pub max_threads_per_block: u32,
    pub max_shared_memory: u32,
}

/// Initialize GPU subsystem and check capabilities
pub fn initialize() -> GpuResult<GpuCapabilities> {
    #[cfg(feature = "cuda")]
    {
        match device::check_cuda_availability() {
            Ok(capabilities) => Ok(capabilities),
            Err(_) => {
                // Fallback to CPU-only mode
                Ok(GpuCapabilities {
                    cuda_available: false,
                    device_count: 0,
                    total_memory: 0,
                    compute_capability: (0, 0),
                    max_threads_per_block: 0,
                    max_shared_memory: 0,
                })
            }
        }
    }
    
    #[cfg(not(feature = "cuda"))]
    {
        Ok(GpuCapabilities {
            cuda_available: false,
            device_count: 0,
            total_memory: 0,
            compute_capability: (0, 0),
            max_threads_per_block: 0,
            max_shared_memory: 0,
        })
    }
}

/// Check if GPU acceleration is available
pub fn is_gpu_available() -> bool {
    initialize().map(|caps| caps.cuda_available).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_initialization() {
        let result = initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_gpu_availability_check() {
        // Should not panic
        let _available = is_gpu_available();
    }

    #[test]
    fn test_error_types() {
        let cuda_error = GpuError::CudaError("test".to_string());
        assert!(cuda_error.to_string().contains("CUDA device error"));
        
        let memory_error = GpuError::MemoryError("test".to_string());
        assert!(memory_error.to_string().contains("GPU memory allocation failed"));
    }
} 