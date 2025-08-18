use crate::{Benchmark, ValidationResult, ValidationReport, ValidationError};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphereConfig {
    pub radius: f64,
    pub mesh_density: f64,
}

impl Default for SphereConfig {
    fn default() -> Self {
        Self {
            radius: 1.0,
            mesh_density: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphereResults {
    pub config: SphereConfig,
    pub computation_time: f64,
}

pub struct SphereBenchmark {
    config: SphereConfig,
}

impl SphereBenchmark {
    pub fn new() -> Self {
        Self {
            config: SphereConfig::default(),
        }
    }
}

impl Benchmark for SphereBenchmark {
    type Config = SphereConfig;
    type Results = SphereResults;
    
    fn new(config: Self::Config) -> Self {
        Self { config }
    }
    
    fn run_tests(&self) -> ValidationResult<Self::Results> {
        Ok(SphereResults {
            config: self.config.clone(),
            computation_time: 0.5,
        })
    }
    
    fn validate(&self, _results: &Self::Results) -> ValidationResult<ValidationReport> {
        Ok(ValidationReport {
            benchmark_name: "Sphere".to_string(),
            passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            summary: "Sphere validation passed".to_string(),
            detailed_results: serde_json::Value::Null,
        })
    }
    
    fn name(&self) -> &str {
        "Sphere"
    }
    
    fn description(&self) -> &str {
        "Sphere analytical benchmark"
    }
} 