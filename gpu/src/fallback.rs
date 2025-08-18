use crate::{GpuError, GpuResult};
use wavecore_matrices::Matrix;
use wavecore_meshes::Mesh;
use wavecore_green_functions::Method;
use std::time::Instant;

/// Configuration for CPU fallback
#[derive(Debug, Clone)]
pub struct CpuFallbackConfig {
    /// Number of CPU threads to use
    pub num_threads: usize,
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Memory limit for CPU operations (in bytes)
    pub memory_limit: u64,
}

impl Default for CpuFallbackConfig {
    fn default() -> Self {
        Self {
            num_threads: num_cpus::get(),
            enable_parallel: true,
            memory_limit: 4 * 1024 * 1024 * 1024, // 4 GB
        }
    }
}

/// CPU fallback implementation for GPU operations
pub struct CpuFallback {
    config: CpuFallbackConfig,
}

impl CpuFallback {
    /// Create new CPU fallback
    pub fn new() -> Self {
        Self::with_config(CpuFallbackConfig::default())
    }

    /// Create CPU fallback with custom configuration
    pub fn with_config(config: CpuFallbackConfig) -> Self {
        // Set rayon thread pool size
        if config.enable_parallel {
            rayon::ThreadPoolBuilder::new()
                .num_threads(config.num_threads)
                .build_global()
                .unwrap_or_else(|_| {
                    tracing::warn!("Failed to set global thread pool, using default");
                });
        }

        Self { config }
    }

    /// Solve BEM problem on CPU
    pub fn solve_cpu(&self, mesh: &Mesh, _green_function: &Method) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        tracing::info!("Starting CPU fallback BEM solution");

        // Assemble influence matrix
        let influence_matrix = self.assemble_matrix_cpu(mesh)?;
        tracing::debug!("Matrix assembly completed in {:?}", start_time.elapsed());

        // Create right-hand side vector
        let rhs = self.create_rhs_vector(mesh)?;

        // Solve linear system
        let solution = self.solve_linear_system_cpu(&influence_matrix, &rhs)?;

        tracing::info!("CPU fallback solution completed in {:?}", start_time.elapsed());
        Ok(solution)
    }

    /// Assemble BEM influence matrix on CPU
    fn assemble_matrix_cpu(&self, mesh: &Mesh) -> Result<Matrix, Box<dyn std::error::Error>> {
        // Get mesh information without borrowing mutably
        let n_panels = mesh.vertices.len() / 4; // Estimate panels from vertices
        
        // Check memory requirements
        let matrix_size = n_panels * n_panels * std::mem::size_of::<f64>();
        if matrix_size as u64 > self.config.memory_limit {
            return Err(format!("Matrix size {} bytes exceeds memory limit {} bytes", 
                matrix_size, self.config.memory_limit).into());
        }

        let mut matrix = Matrix::new(n_panels, n_panels);

        if self.config.enable_parallel {
            self.assemble_matrix_parallel_simplified(n_panels, &mut matrix)?;
        } else {
            self.assemble_matrix_sequential_simplified(n_panels, &mut matrix)?;
        }

        Ok(matrix)
    }

    /// Create right-hand side vector for BEM problem
    fn create_rhs_vector(&self, mesh: &Mesh) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        let n_panels = mesh.vertices.len() / 4; // Estimate panels from vertices
        let mut rhs = vec![0.0; n_panels];

        // For now, use unit normal boundary condition
        for i in 0..n_panels {
            rhs[i] = 1.0; // Unit normal velocity
        }

        Ok(rhs)
    }

    /// Simplified parallel matrix assembly without panel access
    fn assemble_matrix_parallel_simplified(
        &self,
        n_panels: usize,
        matrix: &mut Matrix
    ) -> Result<(), Box<dyn std::error::Error>> {
        use rayon::prelude::*;

        // Process matrix elements in parallel with simplified computation
        let matrix_data: Vec<f64> = (0..n_panels).into_par_iter().flat_map(|i| {
            (0..n_panels).into_par_iter().map(|j| {
                if i == j {
                    -0.5 + (1.0 / (4.0 * std::f64::consts::PI)) // Simplified diagonal
                } else {
                    0.1 / (1.0 + (i as f64 - j as f64).abs()) // Simplified off-diagonal
                }
            }).collect::<Vec<f64>>()
        }).collect();

        // Fill matrix
        for (idx, value) in matrix_data.iter().enumerate() {
            let i = idx / n_panels;
            let j = idx % n_panels;
            matrix.set(i, j, *value)?;
        }

        Ok(())
    }

    /// Simplified sequential matrix assembly without panel access
    fn assemble_matrix_sequential_simplified(
        &self,
        n_panels: usize,
        matrix: &mut Matrix
    ) -> Result<(), Box<dyn std::error::Error>> {
        for i in 0..n_panels {
            for j in 0..n_panels {
                let value = if i == j {
                    -0.5 + (1.0 / (4.0 * std::f64::consts::PI)) // Simplified diagonal
                } else {
                    0.1 / (1.0 + (i as f64 - j as f64).abs()) // Simplified off-diagonal
                };
                matrix.set(i, j, value)?;
            }
        }

        Ok(())
    }

    /// Solve linear system on CPU
    fn solve_linear_system_cpu(&self, matrix: &Matrix, rhs: &Vec<f64>) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        let n = matrix.rows;

        if n != rhs.len() {
            return Err(format!("Matrix size {} doesn't match RHS size {}", n, rhs.len()).into());
        }

        // Simple iterative solver (in real implementation would use LU decomposition)
        let mut solution = vec![0.0; n];
        let max_iterations = 1000;
        let tolerance = 1e-8;

        for iteration in 0..max_iterations {
            let mut max_change = 0.0f64;

            for i in 0..n {
                let mut sum = 0.0;
                for j in 0..n {
                    if i != j {
                        sum += matrix.get(i, j)? * solution[j];
                    }
                }

                let old_value = solution[i];
                let diagonal = matrix.get(i, i)?;
                if diagonal.abs() > 1e-12 {
                    solution[i] = (rhs[i] - sum) / diagonal;
                }

                let change = (solution[i] - old_value).abs();
                max_change = max_change.max(change);
            }

            if max_change < tolerance {
                tracing::info!("Iterative solver converged in {} iterations", iteration + 1);
                break;
            }
        }

        Ok(solution)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: CpuFallbackConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &CpuFallbackConfig {
        &self.config
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> CpuFallbackStats {
        CpuFallbackStats {
            num_threads: self.config.num_threads,
            parallel_enabled: self.config.enable_parallel,
            memory_limit: self.config.memory_limit,
        }
    }
}

/// CPU fallback statistics
#[derive(Debug, Clone)]
pub struct CpuFallbackStats {
    pub num_threads: usize,
    pub parallel_enabled: bool,
    pub memory_limit: u64,
}

impl std::fmt::Display for CpuFallbackStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "CPU Fallback Stats:\n\
             Threads: {}\n\
             Parallel: {}\n\
             Memory Limit: {:.1} GB",
            self.num_threads,
            self.parallel_enabled,
            self.memory_limit as f64 / 1024.0 / 1024.0 / 1024.0
        )
    }
}

impl Default for CpuFallback {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_fallback_creation() {
        let fallback = CpuFallback::new();
        assert!(fallback.config.num_threads > 0);
        assert!(fallback.config.enable_parallel);
    }

    #[test]
    fn test_cpu_fallback_config() {
        let config = CpuFallbackConfig {
            num_threads: 4,
            enable_parallel: false,
            memory_limit: 1024 * 1024 * 1024,
        };

        let fallback = CpuFallback::with_config(config.clone());
        assert_eq!(fallback.config.num_threads, 4);
        assert!(!fallback.config.enable_parallel);
    }

    #[test]
    fn test_stats_display() {
        let stats = CpuFallbackStats {
            num_threads: 8,
            parallel_enabled: true,
            memory_limit: 4 * 1024 * 1024 * 1024,
        };

        let display = format!("{}", stats);
        assert!(display.contains("8"));
        assert!(display.contains("4.0 GB"));
    }
}
