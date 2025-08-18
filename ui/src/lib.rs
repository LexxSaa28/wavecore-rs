//! # WaveCore UI Module
//! 
//! User interface for marine hydrodynamics.
//! 
//! This module provides comprehensive user interface functionality for marine
//! hydrodynamics analysis, including web interface, CLI, and API endpoints.
//! 
//! ## Features
//! 
//! - **Web Interface**: REST API and real-time visualization
//! - **CLI Interface**: Command-line tools for batch processing
//! - **API Endpoints**: Complete REST API for all operations
//! - **Real-time Updates**: Live data streaming and visualization
//! - **Interactive Controls**: Web-based parameter adjustment
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_ui::{WebServer, CLIServer, ServerConfig};
//! 
//! // Create web server
//! let config = ServerConfig {
//!     host: "127.0.0.1".to_string(),
//!     port: 8080,
//!     enable_cors: true,
//! };
//! 
//! let server = WebServer::new(config);
//! 
//! // Start server
//! server.start().await?;
//! 
//! println!("Server running on http://127.0.0.1:8080");
//! ```

pub mod web;
pub mod cli;

pub use web::*;
pub use cli::*;

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Error types for UI operations
#[derive(Error, Debug)]
pub enum UIError {
    #[error("Server error: {message}")]
    ServerError { message: String },
    
    #[error("API error: {message}")]
    APIError { message: String },
    
    #[error("Authentication error: {message}")]
    AuthError { message: String },
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
    
    #[error("WebSocket error: {message}")]
    WebSocketError { message: String },
    
    #[error("CLI error: {message}")]
    CLIError { message: String },
    
    #[error("BEM error: {0}")]
    BEMError(#[from] wavecore_bem::BEMError),
    
    #[error("IO error: {0}")]
    IOError(#[from] wavecore_io::IOError),
    
    #[error("Post-processing error: {0}")]
    PostProError(#[from] wavecore_post_pro::PostProError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for UI operations
pub type Result<T> = std::result::Result<T, UIError>;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// Enable WebSocket
    pub enable_websocket: bool,
    /// Max request size (bytes)
    pub max_request_size: usize,
    /// Request timeout (seconds)
    pub request_timeout: u64,
    /// Enable logging
    pub enable_logging: bool,
    /// Log level
    pub log_level: String,
    /// Verbose output
    pub verbose: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
            enable_websocket: true,
            max_request_size: 10 * 1024 * 1024, // 10MB
            request_timeout: 30,
            enable_logging: true,
            log_level: "info".to_string(),
            verbose: false,
        }
    }
}

/// API request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum APIRequest {
    /// BEM solver request
    BEMSolver {
        /// Problem type
        problem_type: String,
        /// Parameters
        parameters: serde_json::Value,
    },
    /// File upload request
    FileUpload {
        /// File name
        filename: String,
        /// File content
        content: Vec<u8>,
    },
    /// Analysis request
    Analysis {
        /// Analysis type
        analysis_type: String,
        /// Parameters
        parameters: serde_json::Value,
    },
    /// Status request
    Status,
}

/// API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum APIResponse {
    /// Success response
    Success {
        /// Response data
        data: serde_json::Value,
        /// Message
        message: String,
    },
    /// Error response
    Error {
        /// Error code
        code: u16,
        /// Error message
        message: String,
        /// Error details
        details: Option<serde_json::Value>,
    },
    /// Progress response
    Progress {
        /// Progress percentage
        progress: f64,
        /// Status message
        message: String,
        /// Estimated time remaining
        eta: Option<f64>,
    },
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    /// Status update
    Status {
        /// Status type
        status_type: String,
        /// Status data
        data: serde_json::Value,
    },
    /// Progress update
    Progress {
        /// Progress percentage
        progress: f64,
        /// Status message
        message: String,
    },
    /// Error message
    Error {
        /// Error message
        message: String,
        /// Error code
        code: Option<u16>,
    },
    /// Data update
    Data {
        /// Data type
        data_type: String,
        /// Data content
        content: serde_json::Value,
    },
}

/// CLI command types
#[derive(Debug, Clone)]
pub enum CLICommand {
    /// Solve BEM problem
    Solve {
        /// Input file
        input: String,
        /// Output file
        output: String,
        /// Configuration file
        config: Option<String>,
    },
    /// Analyze results
    Analyze {
        /// Input file
        input: String,
        /// Analysis type
        analysis_type: String,
        /// Output file
        output: String,
    },
    /// Convert file format
    Convert {
        /// Input file
        input: String,
        /// Output file
        output: String,
        /// Input format
        input_format: String,
        /// Output format
        output_format: String,
    },
    /// Validate mesh
    Validate {
        /// Mesh file
        mesh: String,
        /// Output report
        report: Option<String>,
    },
    /// Benchmark performance
    Benchmark {
        /// Test cases
        test_cases: Vec<String>,
        /// Output file
        output: String,
    },
}

/// CLI configuration
#[derive(Debug, Clone)]
pub struct CLIConfig {
    /// Verbose output
    pub verbose: bool,
    /// Quiet output
    pub quiet: bool,
    /// Use parallel processing
    pub parallel: bool,
    /// Number of threads
    pub threads: Option<usize>,
    /// Memory limit (MB)
    pub memory_limit: Option<usize>,
    /// Timeout (seconds)
    pub timeout: Option<u64>,
}

impl Default for CLIConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            quiet: false,
            parallel: true,
            threads: None,
            memory_limit: None,
            timeout: None,
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Processing time (seconds)
    pub processing_time: f64,
    /// Memory usage (MB)
    pub memory_usage: f64,
    /// CPU usage (%)
    pub cpu_usage: f64,
    /// Throughput (operations/second)
    pub throughput: f64,
    /// Error rate (%)
    pub error_rate: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            processing_time: 0.0,
            memory_usage: 0.0,
            cpu_usage: 0.0,
            throughput: 0.0,
            error_rate: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.enable_cors);
        assert!(config.enable_websocket);
        assert_eq!(config.max_request_size, 10 * 1024 * 1024);
        assert_eq!(config.request_timeout, 30);
        assert!(config.enable_logging);
        assert_eq!(config.log_level, "info");
        assert!(!config.verbose);
    }
    
    #[test]
    fn test_cli_config_default() {
        let config = CLIConfig::default();
        assert!(!config.verbose);
        assert!(!config.quiet);
        assert!(config.parallel);
        assert!(config.threads.is_none());
        assert!(config.memory_limit.is_none());
        assert!(config.timeout.is_none());
    }
    
    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.processing_time, 0.0);
        assert_eq!(metrics.memory_usage, 0.0);
        assert_eq!(metrics.cpu_usage, 0.0);
        assert_eq!(metrics.throughput, 0.0);
        assert_eq!(metrics.error_rate, 0.0);
    }
    
    #[test]
    fn test_api_request_serialization() {
        let request = APIRequest::Status;
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: APIRequest = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, APIRequest::Status));
    }
    
    #[test]
    fn test_api_response_serialization() {
        let response = APIResponse::Success {
            data: serde_json::json!({"result": "success"}),
            message: "Operation completed".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: APIResponse = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, APIResponse::Success { .. }));
    }
} 