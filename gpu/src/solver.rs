use crate::{GpuDevice, GpuError, GpuResult, GpuMemoryPool, GpuKernels, CpuFallback};
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
    pub fn solve_gpu(&self, mesh: &Mesh, green_function: &Method) -> GpuResult<Vec<f64>> {
        // Check if GPU acceleration is beneficial
        if let Some(ref fallback) = self.fallback {
            if self.should_use_cpu_fallback(mesh) {
                return fallback.solve_cpu(mesh, green_function)
                    .map_err(|e| GpuError::FallbackError(e.to_string()));
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
    pub fn assemble_matrix_gpu(&self, mesh: &Mesh, green_function: &Method) -> GpuResult<Matrix> {
        // Get mesh information
        let n_panels = mesh.panels().map_err(|e| GpuError::MeshError(format!("Failed to get panels: {}", e)))?.len();
        
        // Allocate GPU memory for matrix
        let matrix_size = n_panels * n_panels;
        let mut gpu_matrix = self.memory_pool.allocate_matrix(n_panels, n_panels)?;
        
        // Upload mesh data to GPU
        let gpu_mesh = self.upload_mesh_to_gpu(mesh)?;
        
        // Launch matrix assembly kernel
        self.kernels.launch_matrix_assembly(&gpu_mesh, green_function, &mut gpu_matrix)?;
        
        // Download result from GPU
        let cpu_matrix = self.memory_pool.download_matrix(&gpu_matrix)?;
        
        Ok(cpu_matrix)
    }

    /// Solve linear system on GPU
    pub fn solve_linear_system_gpu(&self, matrix: &Matrix, rhs: &Vec<f64>) -> GpuResult<Vec<f64>> {
        // Upload matrix and RHS to GPU
        let gpu_matrix = self.memory_pool.upload_matrix(matrix)?;
        let gpu_rhs = self.memory_pool.upload_vector(rhs)?;
        
        // Allocate solution vector
        let mut gpu_solution = self.memory_pool.allocate_vector(rhs.len())?;
        
        // Launch linear solver kernel
        self.kernels.launch_linear_solver(&gpu_matrix, &gpu_rhs, &mut gpu_solution)?;
        
        // Download solution from GPU
        let solution = self.memory_pool.download_vector(&gpu_rhs)?;
        
        Ok(solution)
    }

    /// Create right-hand side vector for BEM problem
    fn create_rhs_vector(&self, mesh: &Mesh) -> GpuResult<Vec<f64>> {
        let n_panels = mesh.panels().map_err(|e| GpuError::MeshError(format!("Failed to get panels: {}", e)))?.len();
        let mut rhs = vec![0.0; n_panels];
        
        // For now, use unit normal boundary condition
        for (i, panel) in mesh.panels().map_err(|e| GpuError::MeshError(format!("Failed to get panels: {}", e)))?.iter().enumerate() {
            rhs[i] = 1.0; // Unit normal velocity
        }
        
        Ok(rhs)
    }

    /// Upload mesh data to GPU
    fn upload_mesh_to_gpu(&self, mesh: &Mesh) -> GpuResult<GpuMesh> {
        // Convert mesh to GPU-friendly format
        let vertices = self.flatten_vertices(mesh);
        let panels = self.flatten_panels(mesh);
        
        // Upload to GPU memory
        let gpu_vertices = self.memory_pool.upload_vertices(&vertices)?;
        let gpu_panels = self.memory_pool.upload_panels(&panels)?;
        
        Ok(GpuMesh {
            vertices: gpu_vertices,
            panels: gpu_panels,
            n_panels: mesh.panels().map_err(|e| GpuError::MeshError(format!("Failed to get panels: {}", e)))?.len(),
        })
    }

    /// Flatten mesh vertices for GPU upload
    fn flatten_vertices(&self, mesh: &Mesh) -> Vec<f64> {
        let mut vertices = Vec::new();
        
        for panel in mesh.panels().map_err(|e| GpuError::MeshError(format!("Failed to get panels: {}", e)))? {
            for vertex in &panel.vertices {
                vertices.push(vertex.x);
                vertices.push(vertex.y);
                vertices.push(vertex.z);
            }
        }
        
        vertices
    }

    /// Flatten mesh panels for GPU upload
    fn flatten_panels(&self, mesh: &Mesh) -> Vec<u32> {
        let mut panels = Vec::new();
        let mut vertex_index = 0;
        
        for panel in mesh.panels().map_err(|e| GpuError::MeshError(format!("Failed to get panels: {}", e)))? {
            panels.push(panel.vertices.len() as u32); // Number of vertices
            for _ in &panel.vertices {
                panels.push(vertex_index);
                vertex_index += 1;
            }
        }
        
        panels
    }

    /// Check if CPU fallback should be used
    fn should_use_cpu_fallback(&self, mesh: &Mesh) -> bool {
        let n_panels = mesh.panels().map_err(|e| GpuError::MeshError(format!("Failed to get panels: {}", e)))?.len();
        
        // Use CPU for very small problems
        if n_panels < 100 {
            return true;
        }
        
        // Check if GPU is available
        if !self.device.is_available() {
            return true;
        }
        
        // Check memory requirements
        let memory_required = self.estimate_memory_usage(n_panels);
        if let Ok((free_memory, _)) = self.device.memory_usage() {
            if memory_required > free_memory {
                return true;
            }
        }
        
        false
    }

    /// Estimate GPU memory usage for problem
    fn estimate_memory_usage(&self, n_panels: usize) -> u64 {
        let matrix_size = n_panels * n_panels * 8; // 8 bytes per double
        let vector_size = n_panels * 8;
        let mesh_size = n_panels * 12 * 8; // Assume 4 vertices per panel, 3 coords each
        
        (matrix_size + vector_size * 3 + mesh_size) as u64
    }

    /// Get device information
    pub fn device_info(&self) -> &crate::DeviceInfo {
        &self.device.info
    }

    /// Get configuration
    pub fn config(&self) -> &GpuSolverConfig {
        &self.config
    }

    /// Check if GPU is available
    pub fn is_gpu_available(&self) -> bool {
        self.device.is_available()
    }

    /// Benchmark GPU vs CPU performance
    pub fn benchmark(&self, mesh: &Mesh, green_function: &Method) -> GpuResult<BenchmarkResults> {
        let start_time = std::time::Instant::now();
        
        // Time GPU execution
        let gpu_start = std::time::Instant::now();
        let _gpu_result = self.solve_gpu(mesh, green_function)?;
        let gpu_time = gpu_start.elapsed();
        
        // Time CPU execution if fallback is available
        let cpu_time = if let Some(ref fallback) = self.fallback {
            let cpu_start = std::time::Instant::now();
            let _cpu_result = fallback.solve_cpu(mesh, green_function)
                .map_err(|e| GpuError::FallbackError(e.to_string()))?;
            Some(cpu_start.elapsed())
        } else {
            None
        };
        
        let total_time = start_time.elapsed();
        
        Ok(BenchmarkResults {
            gpu_time,
            cpu_time,
            speedup: cpu_time.map(|ct| ct.as_secs_f64() / gpu_time.as_secs_f64()),
            total_time,
            n_panels: mesh.panels().map_err(|e| GpuError::MeshError(format!("Failed to get panels: {}", e)))?.len(),
        })
    }
}

/// GPU mesh representation
pub struct GpuMesh {
    pub vertices: Vec<f64>, // Flattened vertex coordinates
    pub panels: Vec<u32>,   // Panel connectivity
    pub n_panels: usize,
}

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub gpu_time: std::time::Duration,
    pub cpu_time: Option<std::time::Duration>,
    pub speedup: Option<f64>,
    pub total_time: std::time::Duration,
    pub n_panels: usize,
}

impl std::fmt::Display for BenchmarkResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Benchmark Results for {} panels:\n", self.n_panels)?;
        write!(f, "GPU Time: {:.3}s\n", self.gpu_time.as_secs_f64())?;
        
        if let Some(cpu_time) = self.cpu_time {
            write!(f, "CPU Time: {:.3}s\n", cpu_time.as_secs_f64())?;
        }
        
        if let Some(speedup) = self.speedup {
            write!(f, "Speedup: {:.2}x\n", speedup)?;
        }
        
        write!(f, "Total Time: {:.3}s", self.total_time.as_secs_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wavecore_meshes::PredefinedMesh;

    #[test]
    fn test_solver_config_default() {
        let config = GpuSolverConfig::default();
        assert_eq!(config.block_size, (16, 16, 1));
        assert!(config.enable_fallback);
        assert!(config.use_double_precision);
    }

    #[test]
    fn test_memory_estimation() {
        // Create a mock device for testing
        if let Ok(device) = GpuDevice::default() {
            if let Ok(solver) = GpuBemSolver::new(Arc::new(device)) {
                let memory = solver.estimate_memory_usage(100);
                assert!(memory > 0);
                
                // Memory should scale quadratically with problem size
                let memory_1000 = solver.estimate_memory_usage(1000);
                assert!(memory_1000 > memory * 50); // Should be much larger
            }
        }
    }

    #[test]
    fn test_cpu_fallback_decision() {
        if let Ok(device) = GpuDevice::default() {
            if let Ok(solver) = GpuBemSolver::new(Arc::new(device)) {
                // Create small mesh
                let small_mesh = PredefinedMesh::sphere(1.0, 8).expect("Failed to create sphere");
                assert!(solver.should_use_cpu_fallback(&small_mesh));
                
                // Create larger mesh
                let large_mesh = PredefinedMesh::sphere(1.0, 64).expect("Failed to create sphere");
                // Decision depends on GPU availability
            }
        }
    }

    #[test]
    fn test_vertex_flattening() {
        if let Ok(device) = GpuDevice::default() {
            if let Ok(solver) = GpuBemSolver::new(Arc::new(device)) {
                let mesh = PredefinedMesh::sphere(1.0, 8).expect("Failed to create sphere");
                let vertices = solver.flatten_vertices(&mesh);
                
                // Each vertex has 3 coordinates
                let expected_length = mesh.panels().map_err(|e| GpuError::MeshError(format!("Failed to get panels: {}", e)))?.iter().map(|p| p.vertices.len() * 3).sum::<usize>();
                assert_eq!(vertices.len(), expected_length);
            }
        }
    }
} 