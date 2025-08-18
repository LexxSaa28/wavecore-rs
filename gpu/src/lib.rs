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
pub use solver::{GpuBemSolver, GpuSolverConfig, GpuSolverStatistics};
pub use memory::{GpuMemoryPool, GpuMatrix, GpuVector};
pub use kernels::{GpuKernels, KernelType, GpuMesh};
pub use fallback::{CpuFallback, CpuFallbackConfig, CpuFallbackStats};

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

/// Create GPU device with automatic fallback
pub fn create_device() -> GpuResult<GpuDevice> {
    match GpuDevice::new(0) {
        Ok(device) => {
            tracing::info!("GPU device initialized: {}", device.info.name);
            Ok(device)
        },
        Err(e) => {
            tracing::warn!("GPU initialization failed: {}, falling back to CPU", e);
            Err(e)
        }
    }
}

/// Create GPU BEM solver with automatic configuration
pub fn create_solver() -> GpuResult<GpuBemSolver> {
    let device = std::sync::Arc::new(create_device()?);
    GpuBemSolver::new(device)
}

/// Get system GPU information
pub fn get_gpu_info() -> GpuResult<Vec<DeviceInfo>> {
    GpuDevice::list_devices()
}

/// Performance benchmarking utilities
pub mod benchmark {
    use super::*;
    use std::time::Instant;

    /// Benchmark GPU vs CPU performance
    pub fn compare_performance(
        mesh: &wavecore_meshes::Mesh,
        green_fn: &wavecore_green_functions::Method
    ) -> GpuResult<PerformanceComparison> {
        let start_time = Instant::now();

        // Try GPU first
        let gpu_time = match create_solver() {
            Ok(mut solver) => {
                let gpu_start = Instant::now();
                let _result = solver.solve_gpu(mesh, green_fn)?;
                Some(gpu_start.elapsed())
            },
            Err(_) => None,
        };

        // CPU fallback
        let cpu_start = Instant::now();
        let fallback = CpuFallback::new();
        let _cpu_result = fallback.solve_cpu(mesh, green_fn)
            .map_err(|e| GpuError::ComputationError {
                message: format!("CPU benchmark failed: {}", e)
            })?;
        let cpu_time = cpu_start.elapsed();

        Ok(PerformanceComparison {
            gpu_time,
            cpu_time,
            speedup: gpu_time.map(|gt| cpu_time.as_secs_f64() / gt.as_secs_f64()),
            total_time: start_time.elapsed(),
        })
    }

    #[derive(Debug, Clone)]
    pub struct PerformanceComparison {
        pub gpu_time: Option<std::time::Duration>,
        pub cpu_time: std::time::Duration,
        pub speedup: Option<f64>,
        pub total_time: std::time::Duration,
    }

    impl std::fmt::Display for PerformanceComparison {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match (self.gpu_time, self.speedup) {
                (Some(gpu_time), Some(speedup)) => {
                    write!(f,
                        "Performance Comparison:\n\
                         GPU Time: {:.3}s\n\
                         CPU Time: {:.3}s\n\
                         Speedup: {:.2}x\n\
                         Total Time: {:.3}s",
                        gpu_time.as_secs_f64(),
                        self.cpu_time.as_secs_f64(),
                        speedup,
                        self.total_time.as_secs_f64()
                    )
                },
                _ => {
                    write!(f,
                        "Performance Comparison:\n\
                         GPU: Not Available\n\
                         CPU Time: {:.3}s\n\
                         Total Time: {:.3}s",
                        self.cpu_time.as_secs_f64(),
                        self.total_time.as_secs_f64()
                    )
                }
            }
        }
    }
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
        assert!(cuda_error.to_string().contains("CUDA error"));

        let memory_error = GpuError::MemoryError { message: "test".to_string() };
        assert!(memory_error.to_string().contains("Memory allocation failed"));
    }

    #[test]
    fn test_device_creation_fallback() {
        // This should handle gracefully even if no GPU is available
        match create_device() {
            Ok(_) => {
                // GPU available
            },
            Err(GpuError::FeatureNotAvailable { .. }) |
            Err(GpuError::DeviceError { .. }) |
            Err(GpuError::CudaError(_)) => {
                // Expected when no GPU available
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_gpu_info() {
        let result = get_gpu_info();
        assert!(result.is_ok());
        // Should return empty list if no GPU available
    }
}
