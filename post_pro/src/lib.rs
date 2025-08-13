//! # WaveCore Post-Processing Module
//! 
//! Post-processing tools for marine hydrodynamics.
//! 
//! This module provides comprehensive post-processing functionality for marine
//! hydrodynamics analysis, including RAO calculations, Kochin functions,
//! free surface elevation, and result analysis.
//! 
//! ## Features
//! 
//! - **RAO Analysis**: Response Amplitude Operator calculations
//! - **Kochin Functions**: Far-field wave analysis
//! - **Free Surface**: Free surface elevation calculations
//! - **Result Analysis**: Statistical analysis and visualization
//! - **Export Tools**: Multiple format export capabilities
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_post_pro::{RAOAnalyzer, KochinAnalyzer, AnalysisResult};
//! 
//! // Create RAO analyzer
//! let rao_analyzer = RAOAnalyzer::new();
//! 
//! // Calculate RAOs
//! let raos = rao_analyzer.calculate_raos(&bem_results)?;
//! 
//! // Create Kochin analyzer
//! let kochin_analyzer = KochinAnalyzer::new();
//! 
//! // Calculate Kochin functions
//! let kochin = kochin_analyzer.calculate_kochin(&bem_results)?;
//! 
//! println!("RAO analysis complete: {:?}", raos);
//! ```

pub mod analysis;

pub use analysis::*;

use thiserror::Error;
use num_complex::Complex64;
use nalgebra::{Point3, Vector3};

/// Error types for post-processing operations
#[derive(Error, Debug)]
pub enum PostProError {
    #[error("Invalid analysis parameters: {message}")]
    InvalidParameters { message: String },
    
    #[error("Analysis failed: {message}")]
    AnalysisError { message: String },
    
    #[error("Data not found: {name}")]
    DataNotFound { name: String },
    
    #[error("Calculation error: {message}")]
    CalculationError { message: String },
    
    #[error("Export error: {message}")]
    ExportError { message: String },
    
    #[error("BEM error: {0}")]
    BEMError(#[from] wavecore_bem::BEMError),
    
    #[error("Matrix error: {0}")]
    MatrixError(#[from] wavecore_matrices::MatrixError),
    
    #[error("IO error: {0}")]
    IOError(#[from] wavecore_io::IOError),
    
    #[error("Memory allocation failed")]
    MemoryError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for post-processing operations
pub type Result<T> = std::result::Result<T, PostProError>;

/// 3D point type
pub type Point = Point3<f64>;

/// 3D vector type
pub type Vector = Vector3<f64>;

/// Analysis types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalysisType {
    /// RAO analysis
    RAO,
    /// Kochin function analysis
    Kochin,
    /// Free surface elevation
    FreeSurface,
    /// Statistical analysis
    Statistics,
    /// Sensitivity analysis
    Sensitivity,
    /// Optimization analysis
    Optimization,
}

/// RAO (Response Amplitude Operator) data
#[derive(Debug, Clone)]
pub struct RAOData {
    /// Frequencies (rad/s)
    pub frequencies: Vec<f64>,
    /// Wave directions (radians)
    pub directions: Vec<f64>,
    /// RAO values [frequency][direction][dof]
    pub rao_values: Vec<Vec<Vec<Complex64>>>,
    /// Degrees of freedom
    pub dofs: Vec<String>,
}

impl Default for RAOData {
    fn default() -> Self {
        Self {
            frequencies: Vec::new(),
            directions: Vec::new(),
            rao_values: Vec::new(),
            dofs: vec!["Surge".to_string(), "Sway".to_string(), "Heave".to_string(),
                      "Roll".to_string(), "Pitch".to_string(), "Yaw".to_string()],
        }
    }
}

/// Kochin function data
#[derive(Debug, Clone)]
pub struct KochinData {
    /// Frequencies (rad/s)
    pub frequencies: Vec<f64>,
    /// Wave directions (radians)
    pub directions: Vec<f64>,
    /// Kochin function values [frequency][direction]
    pub kochin_values: Vec<Vec<Complex64>>,
    /// Far-field distance
    pub far_field_distance: f64,
}

impl Default for KochinData {
    fn default() -> Self {
        Self {
            frequencies: Vec::new(),
            directions: Vec::new(),
            kochin_values: Vec::new(),
            far_field_distance: 100.0,
        }
    }
}

/// Free surface elevation data
#[derive(Debug, Clone)]
pub struct FreeSurfaceData {
    /// Time points (s)
    pub time_points: Vec<f64>,
    /// Spatial points [x, y]
    pub spatial_points: Vec<Point>,
    /// Elevation values [time][point]
    pub elevation_values: Vec<Vec<f64>>,
    /// Wave height (m)
    pub wave_height: f64,
    /// Wave period (s)
    pub wave_period: f64,
}

impl Default for FreeSurfaceData {
    fn default() -> Self {
        Self {
            time_points: Vec::new(),
            spatial_points: Vec::new(),
            elevation_values: Vec::new(),
            wave_height: 1.0,
            wave_period: 10.0,
        }
    }
}

/// Statistical analysis data
#[derive(Debug, Clone)]
pub struct StatisticsData {
    /// Mean values
    pub mean: Vec<f64>,
    /// Standard deviations
    pub std_dev: Vec<f64>,
    /// Maximum values
    pub max: Vec<f64>,
    /// Minimum values
    pub min: Vec<f64>,
    /// Percentiles (95th, 99th)
    pub percentiles: Vec<Vec<f64>>,
    /// Variable names
    pub variable_names: Vec<String>,
}

impl Default for StatisticsData {
    fn default() -> Self {
        Self {
            mean: Vec::new(),
            std_dev: Vec::new(),
            max: Vec::new(),
            min: Vec::new(),
            percentiles: Vec::new(),
            variable_names: Vec::new(),
        }
    }
}

/// Analysis configuration
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Analysis type
    pub analysis_type: AnalysisType,
    /// Frequency range [min, max] (rad/s)
    pub frequency_range: Option<(f64, f64)>,
    /// Direction range [min, max] (radians)
    pub direction_range: Option<(f64, f64)>,
    /// Number of frequency points
    pub num_frequencies: usize,
    /// Number of direction points
    pub num_directions: usize,
    /// Use parallel processing
    pub parallel: bool,
    /// Tolerance for calculations
    pub tolerance: f64,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            analysis_type: AnalysisType::RAO,
            frequency_range: None,
            direction_range: None,
            num_frequencies: 50,
            num_directions: 36,
            parallel: true,
            tolerance: 1e-6,
        }
    }
}

/// Analysis result
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Analysis type
    pub analysis_type: AnalysisType,
    /// RAO data (if applicable)
    pub rao_data: Option<RAOData>,
    /// Kochin data (if applicable)
    pub kochin_data: Option<KochinData>,
    /// Free surface data (if applicable)
    pub free_surface_data: Option<FreeSurfaceData>,
    /// Statistics data (if applicable)
    pub statistics_data: Option<StatisticsData>,
    /// Analysis metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Processing time (seconds)
    pub processing_time: f64,
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            analysis_type: AnalysisType::RAO,
            rao_data: None,
            kochin_data: None,
            free_surface_data: None,
            statistics_data: None,
            metadata: std::collections::HashMap::new(),
            processing_time: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rao_data_default() {
        let rao_data = RAOData::default();
        assert_eq!(rao_data.dofs.len(), 6);
        assert_eq!(rao_data.dofs[0], "Surge");
        assert_eq!(rao_data.dofs[1], "Sway");
        assert_eq!(rao_data.dofs[2], "Heave");
    }
    
    #[test]
    fn test_kochin_data_default() {
        let kochin_data = KochinData::default();
        assert_eq!(kochin_data.far_field_distance, 100.0);
        assert!(kochin_data.frequencies.is_empty());
        assert!(kochin_data.directions.is_empty());
    }
    
    #[test]
    fn test_free_surface_data_default() {
        let free_surface_data = FreeSurfaceData::default();
        assert_eq!(free_surface_data.wave_height, 1.0);
        assert_eq!(free_surface_data.wave_period, 10.0);
        assert!(free_surface_data.time_points.is_empty());
        assert!(free_surface_data.spatial_points.is_empty());
    }
    
    #[test]
    fn test_statistics_data_default() {
        let stats_data = StatisticsData::default();
        assert!(stats_data.mean.is_empty());
        assert!(stats_data.std_dev.is_empty());
        assert!(stats_data.max.is_empty());
        assert!(stats_data.min.is_empty());
        assert!(stats_data.variable_names.is_empty());
    }
    
    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();
        assert_eq!(config.analysis_type, AnalysisType::RAO);
        assert_eq!(config.num_frequencies, 50);
        assert_eq!(config.num_directions, 36);
        assert!(config.parallel);
        assert_eq!(config.tolerance, 1e-6);
    }
    
    #[test]
    fn test_analysis_result_default() {
        let result = AnalysisResult::default();
        assert_eq!(result.analysis_type, AnalysisType::RAO);
        assert!(result.rao_data.is_none());
        assert!(result.kochin_data.is_none());
        assert!(result.free_surface_data.is_none());
        assert!(result.statistics_data.is_none());
        assert_eq!(result.processing_time, 0.0);
    }
} 