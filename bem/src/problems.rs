//! BEM problem definitions

use super::*;

/// Boundary condition types for BEM problems
#[derive(Debug, Clone)]
pub enum BoundaryCondition {
    /// Neumann boundary condition (normal velocity specified)
    Neumann { value: f64 },
    /// Dirichlet boundary condition (potential specified)
    Dirichlet { value: f64 },
    /// Robin boundary condition (mixed)
    Robin { alpha: f64, beta: f64, value: f64 },
    /// Radiation condition
    Radiation { frequency: f64, mode: usize },
    /// Free surface condition
    FreeSurface,
}

/// BEM problem definition
#[derive(Debug, Clone)]
pub struct BEMProblem {
    pub problem_type: ProblemType,
    pub parameters: std::collections::HashMap<String, f64>,
    pub boundary_conditions: Vec<BoundaryCondition>,
}

impl BEMProblem {
    /// Create a new BEM problem
    pub fn new(problem_type: ProblemType) -> Self {
        Self {
            problem_type,
            parameters: std::collections::HashMap::new(),
            boundary_conditions: Vec::new(),
        }
    }

    /// Add boundary condition
    pub fn add_boundary_condition(&mut self, bc: BoundaryCondition) {
        self.boundary_conditions.push(bc);
    }
}
