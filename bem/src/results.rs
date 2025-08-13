//! BEM result structures

use super::*;

/// BEM result structure
pub struct BEMResult {
    pub problem: BEMProblem,
    pub solution: Vec<f64>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl BEMResult {
    /// Create a new BEM result
    pub fn new(problem: BEMProblem, solution: Vec<f64>) -> Self {
        Self {
            problem,
            solution,
            metadata: std::collections::HashMap::new(),
        }
    }
} 