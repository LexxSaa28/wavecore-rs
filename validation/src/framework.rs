use crate::{
    ValidationResult, ValidationError, ValidationCriteria,
    DTMB5415Benchmark, WigleyBenchmark, SphereBenchmark,
    Benchmark
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Comprehensive validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub benchmark_name: String,
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub summary: String,
    pub detailed_results: serde_json::Value,
}

/// Validation framework for managing benchmarks
pub struct ValidationFramework {
    criteria: ValidationCriteria,
    benchmarks: HashMap<String, Box<dyn BenchmarkRunner>>,
}

/// Trait for benchmark runners to enable dynamic dispatch
trait BenchmarkRunner {
    fn run_and_validate(&self) -> ValidationResult<ValidationReport>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

impl ValidationFramework {
    /// Create new validation framework
    pub fn new() -> ValidationResult<Self> {
        let criteria = ValidationCriteria::default();
        let mut benchmarks: HashMap<String, Box<dyn BenchmarkRunner>> = HashMap::new();
        
        // Add standard benchmarks
        benchmarks.insert("dtmb5415".to_string(), Box::new(DTMB5415Runner::new()));
        benchmarks.insert("wigley".to_string(), Box::new(WigleyRunner::new()));
        benchmarks.insert("sphere".to_string(), Box::new(SphereRunner::new()));
        
        Ok(Self {
            criteria,
            benchmarks,
        })
    }

    /// Create validation framework with custom criteria
    pub fn with_criteria(criteria: ValidationCriteria) -> ValidationResult<Self> {
        let mut framework = Self::new()?;
        framework.criteria = criteria;
        Ok(framework)
    }

    /// Run all validation benchmarks
    pub fn run_all_validations(&self) -> ValidationResult<HashMap<String, ValidationReport>> {
        let mut reports = HashMap::new();
        
        for (name, benchmark) in &self.benchmarks {
            println!("Running validation: {}", benchmark.name());
            
            match benchmark.run_and_validate() {
                Ok(report) => {
                    println!("✓ {} completed", benchmark.name());
                    reports.insert(name.clone(), report);
                },
                Err(e) => {
                    println!("✗ {} failed: {}", benchmark.name(), e);
                    reports.insert(name.clone(), ValidationReport {
                        benchmark_name: benchmark.name().to_string(),
                        passed: false,
                        errors: vec![e.to_string()],
                        warnings: Vec::new(),
                        summary: format!("Benchmark execution failed: {}", e),
                        detailed_results: serde_json::Value::Null,
                    });
                }
            }
        }
        
        Ok(reports)
    }

    /// Run specific validation benchmark
    pub fn run_validation(&self, name: &str) -> ValidationResult<ValidationReport> {
        match self.benchmarks.get(name) {
            Some(benchmark) => benchmark.run_and_validate(),
            None => Err(ValidationError::BenchmarkError(format!("Benchmark '{}' not found", name))),
        }
    }

    /// List available benchmarks
    pub fn list_benchmarks(&self) -> Vec<(String, String)> {
        self.benchmarks.iter()
            .map(|(name, benchmark)| (name.clone(), benchmark.description().to_string()))
            .collect()
    }

    /// Generate summary report
    pub fn generate_summary(&self, reports: &HashMap<String, ValidationReport>) -> ValidationSummary {
        let total_benchmarks = reports.len();
        let passed_benchmarks = reports.values().filter(|r| r.passed).count();
        let failed_benchmarks = total_benchmarks - passed_benchmarks;
        
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        
        for report in reports.values() {
            all_errors.extend(report.errors.clone());
            all_warnings.extend(report.warnings.clone());
        }
        
        ValidationSummary {
            total_benchmarks: reports.len(),
            passed_benchmarks: passed_benchmarks,
            failed_benchmarks: failed_benchmarks,
            overall_passed: failed_benchmarks == 0,
            errors: all_errors,
            warnings: all_warnings,
            timestamp: "2024-01-01 00:00:00 UTC".to_string(), // Fixed timestamp
        }
    }

    /// Export reports to JSON
    pub fn export_reports(&self, reports: &HashMap<String, ValidationReport>, path: &str) -> ValidationResult<()> {
        let json = serde_json::to_string_pretty(reports)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Get validation criteria
    pub fn criteria(&self) -> &ValidationCriteria {
        &self.criteria
    }
}

/// Validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_benchmarks: usize,
    pub passed_benchmarks: usize,
    pub failed_benchmarks: usize,
    pub overall_passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub timestamp: String,
}

impl std::fmt::Display for ValidationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== WaveCore Validation Summary ===")?;
        writeln!(f, "Timestamp: {}", self.timestamp)?;
        writeln!(f, "Total Benchmarks: {}", self.total_benchmarks)?;
        writeln!(f, "Passed: {}", self.passed_benchmarks)?;
        writeln!(f, "Failed: {}", self.failed_benchmarks)?;
        writeln!(f, "Overall Status: {}", if self.overall_passed { "PASSED" } else { "FAILED" })?;
        
        if !self.errors.is_empty() {
            writeln!(f, "\nErrors:")?;
            for error in &self.errors {
                writeln!(f, "  - {}", error)?;
            }
        }
        
        if !self.warnings.is_empty() {
            writeln!(f, "\nWarnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  - {}", warning)?;
            }
        }
        
        Ok(())
    }
}

// Benchmark runners for dynamic dispatch
struct DTMB5415Runner {
    benchmark: DTMB5415Benchmark,
}

impl DTMB5415Runner {
    fn new() -> Self {
        Self {
            benchmark: DTMB5415Benchmark::new(),
        }
    }
}

impl BenchmarkRunner for DTMB5415Runner {
    fn run_and_validate(&self) -> ValidationResult<ValidationReport> {
        let results = self.benchmark.run_tests()?;
        self.benchmark.validate(&results)
    }
    
    fn name(&self) -> &str {
        self.benchmark.name()
    }
    
    fn description(&self) -> &str {
        self.benchmark.description()
    }
}

struct WigleyRunner {
    benchmark: WigleyBenchmark,
}

impl WigleyRunner {
    fn new() -> Self {
        Self {
            benchmark: WigleyBenchmark::new(),
        }
    }
}

impl BenchmarkRunner for WigleyRunner {
    fn run_and_validate(&self) -> ValidationResult<ValidationReport> {
        let results = self.benchmark.run_tests()?;
        self.benchmark.validate(&results)
    }
    
    fn name(&self) -> &str {
        self.benchmark.name()
    }
    
    fn description(&self) -> &str {
        self.benchmark.description()
    }
}

struct SphereRunner {
    benchmark: SphereBenchmark,
}

impl SphereRunner {
    fn new() -> Self {
        Self {
            benchmark: SphereBenchmark::new(),
        }
    }
}

impl BenchmarkRunner for SphereRunner {
    fn run_and_validate(&self) -> ValidationResult<ValidationReport> {
        let results = self.benchmark.run_tests()?;
        self.benchmark.validate(&results)
    }
    
    fn name(&self) -> &str {
        self.benchmark.name()
    }
    
    fn description(&self) -> &str {
        self.benchmark.description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_creation() {
        let framework = ValidationFramework::new();
        assert!(framework.is_ok());
        
        let framework = framework.unwrap();
        assert_eq!(framework.benchmarks.len(), 3); // DTMB, Wigley, Sphere
    }

    #[test]
    fn test_benchmark_listing() {
        let framework = ValidationFramework::new().unwrap();
        let benchmarks = framework.list_benchmarks();
        
        assert!(!benchmarks.is_empty());
        assert!(benchmarks.iter().any(|(name, _)| name == "dtmb5415"));
        assert!(benchmarks.iter().any(|(name, _)| name == "wigley"));
        assert!(benchmarks.iter().any(|(name, _)| name == "sphere"));
    }

    #[test]
    fn test_validation_summary_display() {
        let summary = ValidationSummary {
            total_benchmarks: 3,
            passed_benchmarks: 2,
            failed_benchmarks: 1,
            overall_passed: false,
            errors: vec!["Test error".to_string()],
            warnings: vec!["Test warning".to_string()],
            timestamp: "2024-01-01 00:00:00 UTC".to_string(),
        };
        
        let display = format!("{}", summary);
        assert!(display.contains("Total Benchmarks: 3"));
        assert!(display.contains("FAILED"));
        assert!(display.contains("Test error"));
    }
} 