//! # WaveCore I/O Module
//! 
//! Input/Output operations for marine hydrodynamics.
//! 
//! This module provides comprehensive I/O functionality for marine hydrodynamics
//! data, including file format support, data arrays, and serialization.
//! 
//! ## Features
//! 
//! - **File I/O**: Multiple format support (STL, OBJ, NEMOH, WAMIT)
//! - **Data Arrays**: XArray-like functionality for efficient data handling
//! - **Serialization**: JSON, YAML, binary formats
//! - **Memory Mapping**: Efficient large file handling
//! - **Format Conversion**: Between different file formats
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_io::{FileIO, DataArray, Format};
//! 
//! fn example() -> wavecore_io::Result<()> {
//!     // Create data array
//!     let data = DataArray::new(&[100, 50], &vec![1.0; 5000])?;
//!     
//!     // Save to JSON (corrected parameter order)
//!     FileIO::save_data(&data, "results.json", Format::JSON)?;
//!     
//!     println!("Created data array with shape {:?}", data.shape());
//!     Ok(())
//! }
//! ```

pub mod file_io;
pub mod xarray;
pub mod wamit;
pub mod nemoh;

pub use file_io::*;
pub use wamit::*;
pub use nemoh::*;
pub use xarray::*;

use thiserror::Error;
use ndarray::Array;
use serde::{Serialize, Deserialize};

/// Error types for I/O operations
#[derive(Error, Debug)]
pub enum IOError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Invalid file format: {format}")]
    InvalidFormat { format: String },
    
    #[error("Parse error: {message}")]
    ParseError { message: String },
    
    #[error("Write error: {message}")]
    WriteError { message: String },
    
    #[error("Data array error: {message}")]
    DataArrayError { message: String },
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    
    #[error("Memory mapping error: {0}")]
    MemoryMapError(#[from] std::io::Error),
    
    #[error("Mesh error: {0}")]
    MeshError(#[from] wavecore_meshes::MeshError),
}

/// Result type for I/O operations
pub type Result<T> = std::result::Result<T, IOError>;

/// Supported file formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    /// STL format (stereolithography)
    STL,
    /// OBJ format (Wavefront)
    OBJ,
    /// NEMOH format
    NEMOH,
    /// WAMIT format
    WAMIT,
    /// JSON format
    JSON,
    /// YAML format
    YAML,
    /// Binary format
    Binary,
    /// CSV format
    CSV,
    /// NetCDF format
    NetCDF,
}

impl Format {
    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            Format::STL => "stl",
            Format::OBJ => "obj",
            Format::NEMOH => "nemoh",
            Format::WAMIT => "wamit",
            Format::JSON => "json",
            Format::YAML => "yaml",
            Format::Binary => "bin",
            Format::CSV => "csv",
            Format::NetCDF => "nc",
        }
    }
    
    /// Get MIME type for format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Format::STL => "application/sla",
            Format::OBJ => "text/plain",
            Format::NEMOH => "text/plain",
            Format::WAMIT => "text/plain",
            Format::JSON => "application/json",
            Format::YAML => "application/x-yaml",
            Format::Binary => "application/octet-stream",
            Format::CSV => "text/csv",
            Format::NetCDF => "application/x-netcdf",
        }
    }
    
    /// Check if format is text-based
    pub fn is_text(&self) -> bool {
        matches!(self, Format::OBJ | Format::NEMOH | Format::WAMIT | Format::JSON | Format::YAML | Format::CSV)
    }
    
    /// Check if format is binary
    pub fn is_binary(&self) -> bool {
        matches!(self, Format::STL | Format::Binary | Format::NetCDF)
    }
}

/// Data array types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    /// 32-bit floating point
    Float32,
    /// 64-bit floating point
    Float64,
    /// 32-bit integer
    Int32,
    /// 64-bit integer
    Int64,
    /// Complex 64-bit floating point
    Complex64,
    /// Complex 128-bit floating point
    Complex128,
}

impl DataType {
    /// Get size in bytes
    pub fn size(&self) -> usize {
        match self {
            DataType::Float32 => 4,
            DataType::Float64 => 8,
            DataType::Int32 => 4,
            DataType::Int64 => 8,
            DataType::Complex64 => 8,
            DataType::Complex128 => 16,
        }
    }
    
    /// Get default value
    pub fn default_value(&self) -> String {
        match self {
            DataType::Float32 => "0.0f32".to_string(),
            DataType::Float64 => "0.0".to_string(),
            DataType::Int32 => "0i32".to_string(),
            DataType::Int64 => "0i64".to_string(),
            DataType::Complex64 => "Complex::new(0.0f32, 0.0f32)".to_string(),
            DataType::Complex128 => "Complex::new(0.0, 0.0)".to_string(),
        }
    }
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File format
    pub format: Format,
    /// File size in bytes
    pub size: u64,
    /// Creation time
    pub created: Option<String>,
    /// Modification time
    pub modified: Option<String>,
    /// Additional metadata
    pub attributes: std::collections::HashMap<String, String>,
}

impl Default for FileMetadata {
    fn default() -> Self {
        Self {
            format: Format::Binary,
            size: 0,
            created: None,
            modified: None,
            attributes: std::collections::HashMap::new(),
        }
    }
}

/// I/O statistics
#[derive(Debug, Clone)]
pub struct IOStats {
    /// Number of files read
    pub files_read: usize,
    /// Number of files written
    pub files_written: usize,
    /// Total bytes read
    pub bytes_read: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Average read time (ms)
    pub avg_read_time: f64,
    /// Average write time (ms)
    pub avg_write_time: f64,
}

impl Default for IOStats {
    fn default() -> Self {
        Self {
            files_read: 0,
            files_written: 0,
            bytes_read: 0,
            bytes_written: 0,
            avg_read_time: 0.0,
            avg_write_time: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_extension() {
        assert_eq!(Format::STL.extension(), "stl");
        assert_eq!(Format::JSON.extension(), "json");
        assert_eq!(Format::YAML.extension(), "yaml");
    }
    
    #[test]
    fn test_format_mime_type() {
        assert_eq!(Format::STL.mime_type(), "application/sla");
        assert_eq!(Format::JSON.mime_type(), "application/json");
        assert_eq!(Format::CSV.mime_type(), "text/csv");
    }
    
    #[test]
    fn test_format_text_binary() {
        assert!(Format::JSON.is_text());
        assert!(Format::CSV.is_text());
        assert!(Format::STL.is_binary());
        assert!(Format::Binary.is_binary());
    }
    
    #[test]
    fn test_data_type_size() {
        assert_eq!(DataType::Float32.size(), 4);
        assert_eq!(DataType::Float64.size(), 8);
        assert_eq!(DataType::Complex64.size(), 8);
        assert_eq!(DataType::Complex128.size(), 16);
    }
    
    #[test]
    fn test_file_metadata_default() {
        let metadata = FileMetadata::default();
        assert_eq!(metadata.format, Format::Binary);
        assert_eq!(metadata.size, 0);
        assert!(metadata.created.is_none());
        assert!(metadata.modified.is_none());
        assert!(metadata.attributes.is_empty());
    }
    
    #[test]
    fn test_io_stats_default() {
        let stats = IOStats::default();
        assert_eq!(stats.files_read, 0);
        assert_eq!(stats.files_written, 0);
        assert_eq!(stats.bytes_read, 0);
        assert_eq!(stats.bytes_written, 0);
        assert_eq!(stats.avg_read_time, 0.0);
        assert_eq!(stats.avg_write_time, 0.0);
    }
} 