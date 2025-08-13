//! BEM solver engines

use super::*;

/// BEM solver engine trait
pub trait SolverEngineTrait {
    /// Solve the BEM problem
    fn solve(&self, problem: &BEMProblem) -> Result<BEMResult>;
}

/// Standard BEM solver engine
pub struct StandardSolverEngine {
    config: BEMConfig,
}

impl StandardSolverEngine {
    /// Create a new standard solver engine
    pub fn new(config: BEMConfig) -> Self {
        Self { config }
    }
}

impl SolverEngineTrait for StandardSolverEngine {
    fn solve(&self, _problem: &BEMProblem) -> Result<BEMResult> {
        // TODO: Implement standard solver
        Err(BEMError::SolverError {
            message: "Standard solver not implemented yet".to_string(),
        })
    }
} 