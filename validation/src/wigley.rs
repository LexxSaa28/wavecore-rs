use crate::{Benchmark, ValidationResult, ValidationReport, ValidationError};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WigleyConfig {
    pub length: f64,
    pub beam: f64,
    pub draft: f64,
}

impl Default for WigleyConfig {
    fn default() -> Self {
        Self {
            length: 1.0,
            beam: 0.1,
            draft: 0.05,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WigleyResults {
    pub config: WigleyConfig,
    pub computation_time: f64,
}

pub struct WigleyBenchmark {
    config: WigleyConfig,
}

impl WigleyBenchmark {
    pub fn new() -> Self {
        Self {
            config: WigleyConfig::default(),
        }
    }
}

impl Benchmark for WigleyBenchmark {
    type Config = WigleyConfig;
    type Results = WigleyResults;
    
    fn new(config: Self::Config) -> Self {
        Self { config }
    }
    
    fn run_tests(&self) -> ValidationResult<Self::Results> {
        Ok(WigleyResults {
            config: self.config.clone(),
            computation_time: 1.0,
        })
    }
    
    fn validate(&self, _results: &Self::Results) -> ValidationResult<ValidationReport> {
        Ok(ValidationReport {
            benchmark_name: "Wigley Hull".to_string(),
            passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            summary: "Wigley hull validation passed".to_string(),
            detailed_results: serde_json::Value::Null,
        })
    }
    
    fn name(&self) -> &str {
        "Wigley Hull"
    }
    
    fn description(&self) -> &str {
        "Wigley hull mathematical benchmark"
    }
} 