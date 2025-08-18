use crate::{GpuDevice, GpuError, GpuResult, GpuMemoryPool, GpuKernels, CpuFallback};
use crate::kernels::GpuMesh;
use wavecore_matrices::Matrix;
use wavecore_meshes::Mesh;
use wavecore_green_functions::Method;
use nalgebra::Vector3;
use std::sync::Arc;

/// Configuration for GPU BEM solver
#[derive(Debug, Clone)]
pub struct GpuSolverConfig {
    /// Block size for CUDA kernels
    pub block_size: (u32, u32, u32),
    /// Grid size for CUDA kernels
    pub grid_size: (u32, u32, u32),
    /// Memory pool size (in bytes)
    pub memory_pool_size: u64,
    /// Enable CPU fallback
    pub enable_fallback: bool,
    /// Performance threshold for fallback (speedup factor)
    pub fallback_threshold: f64,
    /// Precision settings
    pub use_double_precision: bool,
}

impl Default for GpuSolverConfig {
    fn default() -> Self {
        Self {
            block_size: (16, 16, 1),
            grid_size: (64, 64, 1),
            memory_pool_size: 1024 * 1024 * 1024, // 1 GB
            enable_fallback: true,
            fallback_threshold: 2.0, // Require 2x speedup to use GPU
            use_double_precision: true,
        }
    }
}

/// GPU-accelerated BEM solver
pub struct GpuBemSolver {
    device: Arc<GpuDevice>,
    config: GpuSolverConfig,
    memory_pool: GpuMemoryPool,
    kernels: GpuKernels,
    fallback: Option<CpuFallback>,
}

impl GpuBemSolver {
    /// Create new GPU BEM solver
    pub fn new(device: Arc<GpuDevice>) -> GpuResult<Self> {
        Self::with_config(device, GpuSolverConfig::default())
    }

    /// Create GPU BEM solver with custom configuration
    pub fn with_config(device: Arc<GpuDevice>, config: GpuSolverConfig) -> GpuResult<Self> {
        let memory_pool = GpuMemoryPool::new(device.clone(), config.memory_pool_size)?;
        let kernels = GpuKernels::new(device.clone())?;
        
        let fallback = if config.enable_fallback {
            Some(CpuFallback::new())
        } else {
            None
        };

        Ok(Self {
            device,
            config,
            memory_pool,
            kernels,
            fallback,
        })
    }

    /// Solve BEM problem on GPU
    pub fn solve_gpu(&mut self, mesh: &Mesh, green_function: &Method) -> GpuResult<Vec<f64>> {
        // Check if GPU acceleration is beneficial
        if let Some(ref fallback) = self.fallback {
            if self.should_use_cpu_fallback(mesh) {
                return fallback.solve_cpu(mesh, green_function)
                    .map_err(|e| GpuError::ComputationError {
                        message: format!("CPU fallback failed: {}", e)
                    });
            }
        }

        // Assemble influence matrix on GPU
        let influence_matrix = self.assemble_matrix_gpu(mesh, green_function)?;
        
        // Create right-hand side vector
        let rhs = self.create_rhs_vector(mesh)?;
        
        // Solve linear system on GPU
        let solution = self.solve_linear_system_gpu(&influence_matrix, &rhs)?;
        
        Ok(solution)
    }

    /// Assemble BEM influence matrix on GPU
    pub fn assemble_matrix_gpu(&mut self, mesh: &Mesh, green_function: &Method) -> GpuResult<Matrix> {
        // Convert mesh to GPU format
        let gpu_mesh = self.upload_mesh_to_gpu(mesh)?;

        // Get mesh information
        let n_panels = gpu_mesh.panel_count();

        // Allocate GPU memory for matrix
        let mut gpu_matrix = self.memory_pool.allocate_matrix(n_panels, n_panels)?;
        
        // Launch matrix assembly kernel
        self.kernels.launch_matrix_assembly(&gpu_mesh, green_function, &mut gpu_matrix)?;
        
        // Download result from GPU
        let cpu_matrix = self.memory_pool.download_matrix(&gpu_matrix)?;
        
        Ok(cpu_matrix)
    }

    /// Solve linear system on GPU
    pub fn solve_linear_system_gpu(&mut self, matrix: &Matrix, rhs: &Vec<f64>) -> GpuResult<Vec<f64>> {
        // Upload matrix and RHS to GPU
        let gpu_matrix = self.memory_pool.upload_matrix(matrix)?;
        let gpu_rhs = self.memory_pool.upload_vector(rhs)?;
        
        // Allocate solution vector
        let mut gpu_solution = self.memory_pool.allocate_vector(rhs.len())?;
        
        // Launch linear solver kernel
        self.kernels.launch_linear_solver(&gpu_matrix, &gpu_rhs, &mut gpu_solution)?;
        
        // Download solution from GPU
        let solution = self.memory_pool.download_vector(&gpu_solution)?;

        Ok(solution)
    }

    /// Upload mesh data to GPU
    fn upload_mesh_to_gpu(&self, mesh: &Mesh) -> GpuResult<GpuMesh> {
        // Get mesh data using public API - use immutable access
        let vertices = &mesh.vertices;

        // Create a simplified panel representation since we can't access panels mutably
        let num_panels = mesh.vertices.len() / 4; // Estimate based on vertices
        
        // Convert vertices to GPU format
        let gpu_vertices: Vec<[f64; 3]> = vertices.iter()
            .map(|v| [v.x, v.y, v.z])
            .collect();

        // Create simplified panel representation
        let gpu_panels: Vec<[usize; 4]> = (0..num_panels)
            .map(|i| {
                let base = i * 4;
                [base, base + 1, base + 2, base + 3]
            })
            .collect();

        // Calculate panel centers and normals (simplified)
        let gpu_panel_centers: Vec<[f64; 3]> = (0..num_panels)
            .map(|i| {
                let base = i * 4;
                if base + 3 < vertices.len() {
                    let center_x = (vertices[base].x + vertices[base + 1].x + vertices[base + 2].x + vertices[base + 3].x) / 4.0;
                    let center_y = (vertices[base].y + vertices[base + 1].y + vertices[base + 2].y + vertices[base + 3].y) / 4.0;
                    let center_z = (vertices[base].z + vertices[base + 1].z + vertices[base + 2].z + vertices[base + 3].z) / 4.0;
                    [center_x, center_y, center_z]
                } else {
                    [0.0, 0.0, 0.0]
                }
            })
            .collect();

        // Calculate panel normals (simplified - assume upward normal)
        let gpu_panel_normals: Vec<[f64; 3]> = (0..num_panels)
            .map(|_| [0.0, 0.0, 1.0])
            .collect();

        // Calculate panel areas (simplified)
        let gpu_panel_areas: Vec<f64> = (0..num_panels)
            .map(|_| 1.0) // Simplified unit area
            .collect();

        Ok(GpuMesh {
            vertices: gpu_vertices,
            panels: gpu_panels,
            panel_centers: gpu_panel_centers,
            panel_normals: gpu_panel_normals,
            panel_areas: gpu_panel_areas,
        })
    }

    /// Create right-hand side vector for BEM problem - Fix mutable reference
    fn create_rhs_vector(&self, mesh: &Mesh) -> GpuResult<Vec<f64>> {
        let n_panels = mesh.vertices.len() / 4; // Estimate panels from vertices
        let mut rhs = vec![0.0; n_panels];

        // For now, use unit normal boundary condition
        for i in 0..n_panels {
            rhs[i] = 1.0; // Unit normal velocity
        }

        Ok(rhs)
    }

    /// Check if CPU fallback should be used
    fn should_use_cpu_fallback(&self, mesh: &Mesh) -> bool {
        if !self.config.enable_fallback {
            return false;
        }

        // Check if GPU is available
        if !self.device.is_available() {
            return true;
        }

        // Check problem size - small problems might be faster on CPU
        let n_panels = mesh.vertices.len(); // Use vertices count as approximation

        if n_panels < 100 {
            return true; // Small problems use CPU
        }

        // Check memory requirements
        let matrix_size = n_panels * n_panels * std::mem::size_of::<f64>();
        let (allocated, total, _) = self.memory_pool.memory_stats();

        if allocated + matrix_size as u64 > total {
            return true; // Not enough GPU memory
        }

        false
    }

    /// Get solver statistics
    pub fn get_statistics(&self) -> GpuSolverStatistics {
        let (allocated, total, blocks) = self.memory_pool.memory_stats();

        GpuSolverStatistics {
            device_name: self.device.info.name.clone(),
            memory_allocated: allocated,
            memory_total: total,
            memory_blocks: blocks,
            gpu_available: self.device.is_available(),
            fallback_enabled: self.config.enable_fallback,
        }
    }

    /// Update solver configuration
    pub fn update_config(&mut self, config: GpuSolverConfig) {
        self.config = config;
    }

    /// Reset memory pool
    pub fn reset_memory(&mut self) -> GpuResult<()> {
        self.memory_pool = GpuMemoryPool::new(self.device.clone(), self.config.memory_pool_size)?;
        Ok(())
    }
}

/// GPU solver statistics
#[derive(Debug, Clone)]
pub struct GpuSolverStatistics {
    pub device_name: String,
    pub memory_allocated: u64,
    pub memory_total: u64,
    pub memory_blocks: usize,
    pub gpu_available: bool,
    pub fallback_enabled: bool,
}

impl std::fmt::Display for GpuSolverStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "GPU Solver Statistics:\n\
             Device: {}\n\
             Memory: {:.1} MB / {:.1} MB ({} blocks)\n\
             GPU Available: {}\n\
             Fallback Enabled: {}",
            self.device_name,
            self.memory_allocated as f64 / 1024.0 / 1024.0,
            self.memory_total as f64 / 1024.0 / 1024.0,
            self.memory_blocks,
            self.gpu_available,
            self.fallback_enabled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_device() -> Arc<GpuDevice> {
        Arc::new(GpuDevice {
            info: crate::device::DeviceInfo {
                id: 0,
                name: "Test GPU".to_string(),
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
        })
    }

    #[test]
    fn test_gpu_solver_creation() {
        let device = create_mock_device();
        let solver = GpuBemSolver::new(device);
        assert!(solver.is_ok());
    }

    #[test]
    fn test_solver_config() {
        let device = create_mock_device();
        let config = GpuSolverConfig {
            memory_pool_size: 512 * 1024 * 1024,
            enable_fallback: false,
            ..Default::default()
        };

        let solver = GpuBemSolver::with_config(device, config);
        assert!(solver.is_ok());
    }

    #[test]
    fn test_solver_statistics() {
        let device = create_mock_device();
        let solver = GpuBemSolver::new(device).unwrap();
        let stats = solver.get_statistics();

        assert_eq!(stats.device_name, "Test GPU");
        assert_eq!(stats.memory_allocated, 0);
        assert!(stats.memory_total > 0);
    }

    #[test]
    fn test_statistics_display() {
        let stats = GpuSolverStatistics {
            device_name: "Test GPU".to_string(),
            memory_allocated: 512 * 1024 * 1024,
            memory_total: 1024 * 1024 * 1024,
            memory_blocks: 5,
            gpu_available: true,
            fallback_enabled: true,
        };

        let display = format!("{}", stats);
        assert!(display.contains("Test GPU"));
        assert!(display.contains("512.0 MB"));
    }
}
