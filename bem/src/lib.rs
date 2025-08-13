//! # WaveCore BEM Module
//! 
//! Boundary Element Method (BEM) solver for marine hydrodynamics.
//! 
//! This module provides the core BEM solver functionality for wave-body interactions,
//! including radiation and diffraction problems, linear solvers, and solver engines.
//! 
//! ## Features
//! 
//! - **BEM Solver**: Complete boundary element method implementation
//! - **Problem Types**: Radiation and diffraction problems
//! - **Solver Engines**: Multiple solver strategies
//! - **Linear Solvers**: Integration with matrix solvers
//! - **Wave Theory**: Airy wave theory implementation
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_bem::{BEMSolver, ProblemType, SolverEngine};
//! 
//! // Create a BEM solver
//! let solver = BEMSolver::new(SolverEngine::Standard);
//! 
//! // Create radiation problem
//! let problem = ProblemType::Radiation {
//!     frequency: 1.0,
//!     mode: 0,
//! };
//! 
//! println!("Solver created with engine: {:?}", solver.config().engine);
//! println!("Problem type: {:?}", problem);
//! ```

pub mod solver;
pub mod problems;
pub mod results;
pub mod linear_solver;
pub mod engines;
pub mod airy_waves;

pub use solver::*;
pub use problems::*;
pub use results::*;
pub use linear_solver::*;
pub use engines::*;
pub use airy_waves::*;

use thiserror::Error;

/// Error types for BEM operations
#[derive(Error, Debug)]
pub enum BEMError {
    #[error("Invalid problem definition: {message}")]
    InvalidProblem { message: String },
    
    #[error("Solver failed: {message}")]
    SolverError { message: String },
    
    #[error("Matrix error: {0}")]
    MatrixError(#[from] wavecore_matrices::MatrixError),
    
    #[error("Green function error: {0}")]
    GreenFunctionError(#[from] wavecore_green_functions::GreenFunctionError),
    
    #[error("Mesh error: {0}")]
    MeshError(#[from] wavecore_meshes::MeshError),
    
    #[error("Body error: {0}")]
    BodyError(#[from] wavecore_bodies::BodyError),
    
    #[error("Numerical error: {message}")]
    NumericalError { message: String },
    
    #[error("Memory allocation failed")]
    MemoryError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for BEM operations
pub type Result<T> = std::result::Result<T, BEMError>;

/// Problem types for BEM solver
#[derive(Debug, Clone)]
pub enum ProblemType {
    /// Radiation problem
    Radiation {
        /// Wave frequency (rad/s)
        frequency: f64,
        /// Motion mode (0-5 for 6 DOF)
        mode: usize,
    },
    /// Diffraction problem
    Diffraction {
        /// Wave frequency (rad/s)
        frequency: f64,
        /// Wave direction (radians)
        direction: f64,
    },
    /// Combined radiation-diffraction problem
    Combined {
        /// Wave frequency (rad/s)
        frequency: f64,
        /// Wave direction (radians)
        direction: f64,
        /// Motion modes to solve
        modes: Vec<usize>,
    },
}

/// Solver engine types
#[derive(Debug, Clone, Copy)]
pub enum SolverEngine {
    /// Standard BEM solver
    Standard,
    /// Fast multipole method
    FastMultipole,
    /// Hierarchical matrix method
    HierarchicalMatrix,
    /// Adaptive solver
    Adaptive,
}

/// BEM solver configuration
#[derive(Debug, Clone)]
pub struct BEMConfig {
    /// Solver engine to use
    pub engine: SolverEngine,
    /// Tolerance for convergence
    pub tolerance: f64,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Use parallel processing
    pub parallel: bool,
    /// Memory limit (bytes)
    pub memory_limit: Option<usize>,
}

impl Default for BEMConfig {
    fn default() -> Self {
        Self {
            engine: SolverEngine::Standard,
            tolerance: 1e-6,
            max_iterations: 1000,
            parallel: true,
            memory_limit: None,
        }
    }
}

/// Main BEM solver
pub struct BEMSolver {
    config: BEMConfig,
}

impl BEMSolver {
    /// Create a new BEM solver with default configuration
    pub fn new(engine: SolverEngine) -> Self {
        Self {
            config: BEMConfig {
                engine,
                ..Default::default()
            },
        }
    }
    
    /// Create a new BEM solver with custom configuration
    pub fn with_config(config: BEMConfig) -> Self {
        Self { config }
    }
    
    /// Get solver configuration
    pub fn config(&self) -> &BEMConfig {
        &self.config
    }
    
    /// Update solver configuration
    pub fn update_config(&mut self, config: BEMConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bem_solver_creation() {
        let solver = BEMSolver::new(SolverEngine::Standard);
        assert_eq!(solver.config().engine as usize, SolverEngine::Standard as usize);
    }
    
    #[test]
    fn test_problem_types() {
        let radiation = ProblemType::Radiation {
            frequency: 1.0,
            mode: 0,
        };
        
        let diffraction = ProblemType::Diffraction {
            frequency: 1.0,
            direction: 0.0,
        };
        
        let combined = ProblemType::Combined {
            frequency: 1.0,
            direction: 0.0,
            modes: vec![0, 1, 2],
        };
        
        // Just test that they can be created
        assert!(matches!(radiation, ProblemType::Radiation { .. }));
        assert!(matches!(diffraction, ProblemType::Diffraction { .. }));
        assert!(matches!(combined, ProblemType::Combined { .. }));
    }
} 