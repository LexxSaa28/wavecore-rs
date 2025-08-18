//! Error types for resistance calculations

use thiserror::Error;

/// Result type for resistance calculations
pub type Result<T> = std::result::Result<T, ResistanceError>;

/// Comprehensive error types for resistance calculations
#[derive(Error, Debug)]
pub enum ResistanceError {
    #[error("Invalid vessel parameters: {message}")]
    InvalidVesselParameters { message: String },

    #[error("Invalid operating conditions: {message}")]
    InvalidOperatingConditions { message: String },

    #[error("Holtrop-Mennen method not applicable: {reason}")]
    HoltropMennenNotApplicable { reason: String },

    #[error("RAO data not available or invalid: {message}")]
    InvalidRAOData { message: String },

    #[error("Wave spectrum data invalid: {message}")]
    InvalidWaveSpectrum { message: String },

    #[error("Wind conditions invalid: {message}")]
    InvalidWindConditions { message: String },

    #[error("Mathematical calculation error: {message}")]
    CalculationError { message: String },

    #[error("Validation failed: {issues:?}")]
    ValidationFailed { issues: Vec<String> },

    #[error("Benchmark data loading error: {message}")]
    BenchmarkError { message: String },

    #[error("Interpolation error: {message}")]
    InterpolationError { message: String },

    #[error("Convergence failure: {iterations} iterations, error: {error}")]
    ConvergenceFailure { iterations: u32, error: f64 },

    #[error("Data serialization error: {source}")]
    SerializationError { #[from] source: serde_json::Error },

    #[error("IO error: {source}")]
    IoError { #[from] source: std::io::Error },

    #[error("Generic error: {message}")]
    Generic { message: String },
}

impl ResistanceError {
    /// Create a new invalid vessel parameters error
    pub fn invalid_vessel_parameters(message: impl Into<String>) -> Self {
        Self::InvalidVesselParameters {
            message: message.into(),
        }
    }

    /// Create a new invalid operating conditions error
    pub fn invalid_operating_conditions(message: impl Into<String>) -> Self {
        Self::InvalidOperatingConditions {
            message: message.into(),
        }
    }

    /// Create a new Holtrop-Mennen not applicable error
    pub fn holtrop_mennen_not_applicable(reason: impl Into<String>) -> Self {
        Self::HoltropMennenNotApplicable {
            reason: reason.into(),
        }
    }

    /// Create a new calculation error
    pub fn calculation_error(message: impl Into<String>) -> Self {
        Self::CalculationError {
            message: message.into(),
        }
    }

    /// Create a new validation error
    pub fn validation_failed(issues: Vec<String>) -> Self {
        Self::ValidationFailed { issues }
    }

    /// Create a new convergence failure error
    pub fn convergence_failure(iterations: u32, error: f64) -> Self {
        Self::ConvergenceFailure { iterations, error }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::InvalidVesselParameters { .. } => false,
            Self::InvalidOperatingConditions { .. } => false,
            Self::HoltropMennenNotApplicable { .. } => true, // Can try other methods
            Self::InvalidRAOData { .. } => true, // Can fall back to empirical methods
            Self::InvalidWaveSpectrum { .. } => true,
            Self::InvalidWindConditions { .. } => true,
            Self::CalculationError { .. } => false,
            Self::ValidationFailed { .. } => false,
            Self::BenchmarkError { .. } => true,
            Self::InterpolationError { .. } => true,
            Self::ConvergenceFailure { .. } => true,
            Self::SerializationError { .. } => false,
            Self::IoError { .. } => true,
            Self::Generic { .. } => false,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::InvalidVesselParameters { .. } => ErrorSeverity::Critical,
            Self::InvalidOperatingConditions { .. } => ErrorSeverity::Critical,
            Self::HoltropMennenNotApplicable { .. } => ErrorSeverity::Warning,
            Self::InvalidRAOData { .. } => ErrorSeverity::Warning,
            Self::InvalidWaveSpectrum { .. } => ErrorSeverity::Warning,
            Self::InvalidWindConditions { .. } => ErrorSeverity::Warning,
            Self::CalculationError { .. } => ErrorSeverity::Error,
            Self::ValidationFailed { .. } => ErrorSeverity::Error,
            Self::BenchmarkError { .. } => ErrorSeverity::Warning,
            Self::InterpolationError { .. } => ErrorSeverity::Warning,
            Self::ConvergenceFailure { .. } => ErrorSeverity::Warning,
            Self::SerializationError { .. } => ErrorSeverity::Error,
            Self::IoError { .. } => ErrorSeverity::Warning,
            Self::Generic { .. } => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Warning,  // Non-fatal, calculation can continue with alternative methods
    Error,    // Error occurred but system remains stable
    Critical, // Critical error, calculation cannot proceed
}

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub value: f64,
    pub expected_range: (f64, f64),
    pub message: String,
}

impl ValidationError {
    pub fn new(field: &str, value: f64, range: (f64, f64), message: &str) -> Self {
        Self {
            field: field.to_string(),
            value,
            expected_range: range,
            message: message.to_string(),
        }
    }
}

/// Helper trait for error context
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<ResistanceError>,
{
    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| {
            let original_error = e.into();
            ResistanceError::Generic {
                message: format!("{}: {}", context, original_error),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = ResistanceError::invalid_vessel_parameters("Length out of range");
        assert!(matches!(error, ResistanceError::InvalidVesselParameters { .. }));
        assert!(!error.is_recoverable());
        assert_eq!(error.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_severity() {
        let warning_error = ResistanceError::holtrop_mennen_not_applicable("Outside CB range");
        assert_eq!(warning_error.severity(), ErrorSeverity::Warning);
        assert!(warning_error.is_recoverable());

        let critical_error = ResistanceError::invalid_vessel_parameters("Invalid length");
        assert_eq!(critical_error.severity(), ErrorSeverity::Critical);
        assert!(!critical_error.is_recoverable());
    }

    #[test]
    fn test_validation_error() {
        let validation_error = ValidationError::new(
            "block_coefficient",
            1.5,
            (0.4, 0.85),
            "Block coefficient out of valid range"
        );
        
        assert_eq!(validation_error.field, "block_coefficient");
        assert_eq!(validation_error.value, 1.5);
        assert_eq!(validation_error.expected_range, (0.4, 0.85));
    }
} 