//! # WaveCore Validation Module
//! 
//! Industry-standard validation benchmarks for marine hydrodynamics.
//! 
//! This module provides comprehensive validation against industry-standard
//! benchmarks including DTMB 5415 destroyer hull, Wigley hull, and sphere
//! test cases with reference data comparison and statistical analysis.
//! 
//! ## Features
//! 
//! - **DTMB 5415**: Standard destroyer hull validation
//! - **Wigley Hull**: Mathematical hull with analytical solutions
//! - **Sphere Benchmark**: Analytical validation case
//! - **Statistical Analysis**: Comprehensive error analysis and reporting
//! - **Reference Data**: Literature results for comparison
//! - **Automated Validation**: Continuous validation pipeline
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_validation::{ValidationFramework, DTMB5415Benchmark};
//! 
//! // Create validation framework
//! let framework = ValidationFramework::new();
//! 
//! // Run DTMB 5415 benchmark
//! let benchmark = DTMB5415Benchmark::new();
//! let results = benchmark.run_seakeeping_tests()?;
//! 
//! // Validate against reference data
//! let report = benchmark.validate_results(&results)?;
//! println!("Validation Report: {}", report);
//! ```

pub mod dtmb5415;
pub mod wigley;
pub mod sphere;
pub mod framework;
pub mod reference_data;
pub mod statistics;

use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Benchmark execution failed: {0}")]
    BenchmarkError(String),
    
    #[error("Reference data not found: {0}")]
    ReferenceDataError(String),
    
    #[error("Validation criteria not met: {0}")]
    ValidationFailed(String),
    
    #[error("Statistical analysis failed: {0}")]
    StatisticsError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

// Re-export main types
pub use framework::{ValidationFramework, ValidationReport};
pub use dtmb5415::{DTMB5415Benchmark, DTMB5415Config, DTMB5415Results};
pub use sphere::{SphereBenchmark, SphereConfig, SphereResults};
pub use wigley::{WigleyBenchmark, WigleyConfig, WigleyResults};
pub use reference_data::{ReferenceData, ReferenceDatabase};
pub use statistics::{StatisticalAnalysis, ErrorMetrics, ComparisonReport};

/// Test condition for validation
#[derive(Debug, Clone)]
pub struct TestCondition {
    /// Frequency (rad/s)
    pub frequency: f64,
    /// Wave direction (rad)
    pub direction: f64,
    /// Wave height (m)
    pub wave_height: f64,
    /// Water depth (m)
    pub water_depth: f64,
}

/// Seakeeping results
#[derive(Debug, Clone)]
pub struct SeakeepingResults {
    /// Added mass coefficients
    pub added_mass: HashMap<String, f64>,
    /// Damping coefficients
    pub damping: HashMap<String, f64>,
    /// Exciting forces
    pub exciting_forces: HashMap<String, f64>,
    /// RAOs (Response Amplitude Operators)
    pub raos: HashMap<String, f64>,
}

/// Validation metadata
#[derive(Debug, Clone)]
pub struct ValidationMetadata {
    /// Benchmark name
    pub benchmark_name: String,
    /// Timestamp
    pub timestamp: String,
    /// Version
    pub version: String,
    /// Description
    pub description: String,
}

/// Mesh information
#[derive(Debug, Clone)]
pub struct MeshInfo {
    /// Number of panels
    pub num_panels: usize,
    /// Number of vertices
    pub num_vertices: usize,
    /// Mesh quality score
    pub mesh_quality: f64,
    /// Coordinate system
    pub coordinate_system: String,
}

/// Benchmark trait for validation cases
pub trait Benchmark {
    type Config;
    type Results;
    
    /// Create new benchmark with configuration
    fn new(config: Self::Config) -> Self;
    
    /// Run benchmark tests
    fn run_tests(&self) -> ValidationResult<Self::Results>;
    
    /// Validate results against reference data
    fn validate(&self, results: &Self::Results) -> ValidationResult<ValidationReport>;
    
    /// Get benchmark name
    fn name(&self) -> &str;
    
    /// Get benchmark description
    fn description(&self) -> &str;
}

/// Validation criteria for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCriteria {
    /// Maximum acceptable relative error (%)
    pub max_relative_error: f64,
    /// Maximum acceptable absolute error
    pub max_absolute_error: f64,
    /// Minimum correlation coefficient
    pub min_correlation: f64,
    /// Statistical significance level
    pub significance_level: f64,
}

impl Default for ValidationCriteria {
    fn default() -> Self {
        Self {
            max_relative_error: 5.0,   // 5% maximum error
            max_absolute_error: 0.1,   // Problem-dependent
            min_correlation: 0.95,     // 95% correlation
            significance_level: 0.05,  // 5% significance
        }
    }
}

/// Initialize validation system
pub fn initialize() -> ValidationResult<ValidationFramework> {
    ValidationFramework::new()
}

/// Run all standard validation benchmarks
pub fn run_all_validations() -> ValidationResult<HashMap<String, ValidationReport>> {
    let framework = initialize()?;
    framework.run_all_validations()
}

/// Quick validation check
pub fn quick_validation() -> ValidationResult<bool> {
    let reports = run_all_validations()?;
    
    // Check if all validations passed
    for (name, report) in &reports {
        if !report.passed {
            println!("Validation failed for: {}", name);
            return Ok(false);
        }
    }
    
    println!("All validations passed!");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_criteria_default() {
        let criteria = ValidationCriteria::default();
        assert_eq!(criteria.max_relative_error, 5.0);
        assert_eq!(criteria.min_correlation, 0.95);
    }

    #[test]
    fn test_test_condition_creation() {
        let condition = TestCondition {
            frequency: 1.0,
            amplitude: 0.1,
            heading: 0.0,
            water_depth: None,
            description: "Test case".to_string(),
        };
        
        assert_eq!(condition.frequency, 1.0);
        assert!(condition.water_depth.is_none());
    }

    #[test]
    fn test_validation_framework_initialization() {
        let result = initialize();
        assert!(result.is_ok());
    }
} 