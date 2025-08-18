//! BEM solver results

use super::*;
use crate::problems::BEMProblem;

/// BEM solution results
#[derive(Debug, Clone)]
pub struct BEMResult {
    pub problem: BEMProblem,
    pub solution: Vec<f64>,
    pub residual: f64,
    pub iterations: usize,
    pub computation_time: f64,
}

impl BEMResult {
    pub fn new(problem: BEMProblem, solution: Vec<f64>) -> Self {
        Self {
            problem,
            solution,
            residual: 0.0,
            iterations: 0,
            computation_time: 0.0,
        }
    }

    /// Get computation time
    pub fn computation_time(&self) -> f64 {
        self.computation_time
    }

    /// Get potential solution (alias for solution)
    pub fn potential(&self) -> &Vec<f64> {
        &self.solution
    }

    /// Get solution vector
    pub fn solution(&self) -> &Vec<f64> {
        &self.solution
    }

    /// Get residual
    pub fn residual(&self) -> f64 {
        self.residual
    }

    /// Get number of iterations
    pub fn iterations(&self) -> usize {
        self.iterations
    }
}

/// Legacy BEM result structure (deprecated - use solver::BEMResult instead)
pub struct LegacyBEMResult {
    pub problem: BEMProblem,
    pub solution: Vec<f64>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl LegacyBEMResult {
    /// Create a new legacy BEM result
    pub fn new(problem: BEMProblem, solution: Vec<f64>) -> Self {
        Self {
            problem,
            solution,
            metadata: std::collections::HashMap::new(),
        }
    }
}