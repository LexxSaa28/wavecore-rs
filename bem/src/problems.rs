//! BEM problem definitions

use super::*;

/// BEM problem definition
pub struct BEMProblem {
    pub problem_type: ProblemType,
    pub parameters: std::collections::HashMap<String, f64>,
}

impl BEMProblem {
    /// Create a new BEM problem
    pub fn new(problem_type: ProblemType) -> Self {
        Self {
            problem_type,
            parameters: std::collections::HashMap::new(),
        }
    }
} 