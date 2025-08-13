//! BEM linear solver

use super::*;

/// BEM linear solver
pub struct BEMLinearSolver {
    solver_type: wavecore_matrices::SolverType,
}

impl BEMLinearSolver {
    /// Create a new BEM linear solver
    pub fn new(solver_type: wavecore_matrices::SolverType) -> Self {
        Self { solver_type }
    }
} 