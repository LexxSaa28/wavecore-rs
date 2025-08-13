use chrono::{DateTime, Local};
use serde::Serialize;

/// Comprehensive test results for WaveCore library
#[derive(Debug, Clone, Serialize)]
pub struct WaveCoreTestResult {
    pub test_name: String,
    pub timestamp: DateTime<Local>,
    pub duration_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_ops_per_sec: f64,
    pub memory_usage_mb: u64,
    pub num_panels: usize,
    pub matrix_size: usize,
    pub num_frequencies: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Test categories for organizing tests
#[derive(Debug, Clone, PartialEq)]
pub enum TestCategory {
    Core,
    Performance,
    Extensions,
    Validation,
    All,
}

impl TestCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "test_core_functional" => Some(TestCategory::Core),
            "test_performance" => Some(TestCategory::Performance),
            "test_extensions" => Some(TestCategory::Extensions),
            "test_numerical_validation" => Some(TestCategory::Validation),
            "test_all" => Some(TestCategory::All),
            _ => None,
        }
    }
} 