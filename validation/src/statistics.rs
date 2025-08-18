use serde::{Serialize, Deserialize};

/// Statistical analysis for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysis {
    pub mean_error: f64,
    pub std_error: f64,
    pub max_error: f64,
    pub correlation: f64,
}

/// Error metrics for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub relative_error: f64,
    pub absolute_error: f64,
    pub rms_error: f64,
}

/// Comparison report between computed and reference data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub statistics: StatisticalAnalysis,
    pub errors: ErrorMetrics,
    pub passed: bool,
    pub summary: String,
}

impl StatisticalAnalysis {
    pub fn new() -> Self {
        Self {
            mean_error: 0.0,
            std_error: 0.0,
            max_error: 0.0,
            correlation: 1.0,
        }
    }
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            relative_error: 0.0,
            absolute_error: 0.0,
            rms_error: 0.0,
        }
    }
} 