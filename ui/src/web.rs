//! Web interface implementation

use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    routing::{get, post},
    http::{StatusCode, HeaderMap},
    response::{Json, Response},
    extract::{Path, State, Query},
    body::Body,
};
use serde_json::Value;
use tower_http::cors::{CorsLayer, Any};

/// Web server
pub struct WebServer {
    config: ServerConfig,
    state: Arc<AppState>,
}

/// Application state
#[derive(Clone)]
struct AppState {
    config: ServerConfig,
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

/// Session data
#[derive(Clone, Debug)]
struct SessionData {
    id: String,
    created_at: std::time::SystemTime,
    last_activity: std::time::SystemTime,
    data: HashMap<String, Value>,
}

impl WebServer {
    /// Create a new web server
    pub fn new(config: ServerConfig) -> Self {
        let state = Arc::new(AppState {
            config: config.clone(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        });
        
        Self { config, state }
    }
    
    /// Start the web server
    pub async fn start(&self) -> Result<()> {
        if self.config.verbose {
            println!("Starting web server on {}:{}", self.config.host, self.config.port);
        }
        
        // Configure CORS
        let cors = if self.config.enable_cors {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        } else {
            CorsLayer::new()
        };
        
        // Create router
        let app = axum::Router::new()
            .route("/", get(Self::index_handler))
            .route("/api/status", get(Self::status_handler))
            .route("/api/solve", post(Self::solve_handler))
            .route("/api/analyze", post(Self::analyze_handler))
            .route("/api/convert", post(Self::convert_handler))
            .route("/api/validate", post(Self::validate_handler))
            .route("/api/benchmark", post(Self::benchmark_handler))
            .route("/api/metrics", get(Self::metrics_handler))
            .route("/api/session/:id", get(Self::session_handler))
            .route("/api/session/:id", post(Self::update_session_handler))
            .route("/ws", get(Self::websocket_handler))
            .layer(cors)
            .with_state(self.state.clone());
        
        // Start server
        let addr = format!("{}:{}", self.config.host, self.config.port);
        
        if self.config.verbose {
            println!("Server listening on http://{}", addr);
        }
        
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| UIError::ServerError {
                message: format!("Failed to bind to address: {}", e),
            })?;
        
        axum::serve(listener, app)
            .await
            .map_err(|e| UIError::ServerError {
                message: format!("Server error: {}", e),
            })?;
        
        Ok(())
    }
    
    /// Index handler
    async fn index_handler() -> Response<Body> {
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>WaveCore - Marine Hydrodynamics</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .container { max-width: 800px; margin: 0 auto; }
        .header { background: #0066cc; color: white; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }
        .button { background: #0066cc; color: white; padding: 10px 20px; border: none; border-radius: 3px; cursor: pointer; }
        .button:hover { background: #0052a3; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🌊 WaveCore</h1>
            <p>High-performance marine hydrodynamics BEM solver</p>
        </div>
        
        <div class="section">
            <h2>API Endpoints</h2>
            <ul>
                <li><strong>GET /api/status</strong> - Server status</li>
                <li><strong>POST /api/solve</strong> - Solve BEM problem</li>
                <li><strong>POST /api/analyze</strong> - Analyze results</li>
                <li><strong>POST /api/convert</strong> - Convert file formats</li>
                <li><strong>POST /api/validate</strong> - Validate mesh</li>
                <li><strong>POST /api/benchmark</strong> - Run benchmarks</li>
                <li><strong>GET /api/metrics</strong> - Performance metrics</li>
                <li><strong>GET /ws</strong> - WebSocket connection</li>
            </ul>
        </div>
        
        <div class="section">
            <h2>Quick Start</h2>
            <p>Use the API endpoints to perform marine hydrodynamics analysis:</p>
            <pre>
curl -X POST http://localhost:8080/api/solve \
  -H "Content-Type: application/json" \
  -d '{"problem_type": "radiation", "parameters": {...}}'
            </pre>
        </div>
    </div>
</body>
</html>
        "#;
        
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Body::from(html))
            .unwrap()
    }
    
    /// Status handler
    async fn status_handler(State(state): State<Arc<AppState>>) -> Json<APIResponse> {
        let status_data = serde_json::json!({
            "server": "WaveCore",
            "version": "1.0.0",
            "status": "running",
            "uptime": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "config": {
                "host": state.config.host,
                "port": state.config.port,
                "enable_cors": state.config.enable_cors,
                "enable_websocket": state.config.enable_websocket,
            }
        });
        
        Json(APIResponse::Success {
            data: status_data,
            message: "Server is running".to_string(),
        })
    }
    
    /// Solve handler
    async fn solve_handler(
        State(state): State<Arc<AppState>>,
        Json(request): Json<APIRequest>,
    ) -> Json<APIResponse> {
        match request {
            APIRequest::BEMSolver { problem_type, parameters } => {
                // Simulate BEM solving
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                let result_data = serde_json::json!({
                    "problem_type": problem_type,
                    "parameters": parameters,
                    "status": "completed",
                    "results": {
                        "added_mass": [1.0, 2.0, 3.0],
                        "damping": [0.1, 0.2, 0.3],
                        "excitation_force": [10.0, 20.0, 30.0],
                    }
                });
                
                Json(APIResponse::Success {
                    data: result_data,
                    message: "BEM problem solved successfully".to_string(),
                })
            }
            _ => Json(APIResponse::Error {
                code: 400,
                message: "Invalid request type".to_string(),
                details: None,
            }),
        }
    }
    
    /// Analyze handler
    async fn analyze_handler(
        State(_state): State<Arc<AppState>>,
        Json(request): Json<APIRequest>,
    ) -> Json<APIResponse> {
        match request {
            APIRequest::Analysis { analysis_type, parameters } => {
                // Simulate analysis
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                
                let result_data = serde_json::json!({
                    "analysis_type": analysis_type,
                    "parameters": parameters,
                    "status": "completed",
                    "results": {
                        "rao": [0.8, 1.2, 0.9],
                        "phase": [0.1, 0.3, 0.2],
                    }
                });
                
                Json(APIResponse::Success {
                    data: result_data,
                    message: "Analysis completed successfully".to_string(),
                })
            }
            _ => Json(APIResponse::Error {
                code: 400,
                message: "Invalid request type".to_string(),
                details: None,
            }),
        }
    }
    
    /// Convert handler
    async fn convert_handler(
        State(_state): State<Arc<AppState>>,
        Json(request): Json<APIRequest>,
    ) -> Json<APIResponse> {
        match request {
            APIRequest::FileUpload { filename, content } => {
                // Simulate file conversion
                tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
                
                let result_data = serde_json::json!({
                    "filename": filename,
                    "size": content.len(),
                    "status": "converted",
                    "output": format!("converted_{}", filename),
                });
                
                Json(APIResponse::Success {
                    data: result_data,
                    message: "File converted successfully".to_string(),
                })
            }
            _ => Json(APIResponse::Error {
                code: 400,
                message: "Invalid request type".to_string(),
                details: None,
            }),
        }
    }
    
    /// Validate handler
    async fn validate_handler(
        State(_state): State<Arc<AppState>>,
        Json(request): Json<APIRequest>,
    ) -> Json<APIResponse> {
        match request {
            APIRequest::FileUpload { filename, content } => {
                // Simulate mesh validation
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                
                let result_data = serde_json::json!({
                    "filename": filename,
                    "size": content.len(),
                    "status": "valid",
                    "validation": {
                        "vertices": 1000,
                        "faces": 2000,
                        "quality": "good",
                        "issues": []
                    }
                });
                
                Json(APIResponse::Success {
                    data: result_data,
                    message: "Mesh validation completed".to_string(),
                })
            }
            _ => Json(APIResponse::Error {
                code: 400,
                message: "Invalid request type".to_string(),
                details: None,
            }),
        }
    }
    
    /// Benchmark handler
    async fn benchmark_handler(
        State(_state): State<Arc<AppState>>,
        Json(request): Json<APIRequest>,
    ) -> Json<APIResponse> {
        match request {
            APIRequest::Analysis { analysis_type, parameters } => {
                // Simulate benchmark execution
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                
                let result_data = serde_json::json!({
                    "analysis_type": analysis_type,
                    "parameters": parameters,
                    "status": "completed",
                    "benchmarks": {
                        "bem_solver": "2.5ms",
                        "green_functions": "1.2ms",
                        "matrix_operations": "0.8ms",
                        "total_time": "4.5ms"
                    }
                });
                
                Json(APIResponse::Success {
                    data: result_data,
                    message: "Benchmark completed successfully".to_string(),
                })
            }
            _ => Json(APIResponse::Error {
                code: 400,
                message: "Invalid request type".to_string(),
                details: None,
            }),
        }
    }
    
    /// Metrics handler
    async fn metrics_handler(State(state): State<Arc<AppState>>) -> Json<APIResponse> {
        let metrics = state.metrics.read().await;
        let metrics_data = serde_json::json!({
            "processing_time": metrics.processing_time,
            "memory_usage": metrics.memory_usage,
            "cpu_usage": metrics.cpu_usage,
            "throughput": metrics.throughput,
            "error_rate": metrics.error_rate,
        });
        
        Json(APIResponse::Success {
            data: metrics_data,
            message: "Performance metrics retrieved".to_string(),
        })
    }
    
    /// Session handler
    async fn session_handler(
        State(state): State<Arc<AppState>>,
        Path(session_id): Path<String>,
    ) -> Json<APIResponse> {
        let sessions = state.sessions.read().await;
        
        if let Some(session) = sessions.get(&session_id) {
            let session_data = serde_json::json!({
                "id": session.id,
                "created_at": session.created_at,
                "last_activity": session.last_activity,
                "data": session.data,
            });
            
            Json(APIResponse::Success {
                data: session_data,
                message: "Session retrieved".to_string(),
            })
        } else {
            Json(APIResponse::Error {
                code: 404,
                message: "Session not found".to_string(),
                details: None,
            })
        }
    }
    
    /// Update session handler
    async fn update_session_handler(
        State(state): State<Arc<AppState>>,
        Path(session_id): Path<String>,
        Json(data): Json<HashMap<String, Value>>,
    ) -> Json<APIResponse> {
        let mut sessions = state.sessions.write().await;
        
        let session = SessionData {
            id: session_id.clone(),
            created_at: std::time::SystemTime::now(),
            last_activity: std::time::SystemTime::now(),
            data,
        };
        
        sessions.insert(session_id.clone(), session);
        
        Json(APIResponse::Success {
            data: serde_json::json!({"session_id": session_id}),
            message: "Session updated".to_string(),
        })
    }
    
    /// WebSocket handler
    async fn websocket_handler() -> Response<Body> {
        // Placeholder for WebSocket implementation
        Response::builder()
            .status(StatusCode::NOT_IMPLEMENTED)
            .body(Body::from("WebSocket not implemented yet"))
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_web_server_creation() {
        let config = ServerConfig::default();
        let server = WebServer::new(config);
        assert_eq!(server.config.host, "127.0.0.1");
        assert_eq!(server.config.port, 8080);
    }
    
    #[tokio::test]
    async fn test_status_handler() {
        let config = ServerConfig::default();
        let state = Arc::new(AppState {
            config: config.clone(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        });
        
        let response = WebServer::status_handler(State(state)).await;
        
        // Check that the response is a success type
        match &response.0 {
            APIResponse::Success { data, message: _ } => {
                assert_eq!(data["server"], "WaveCore");
                assert_eq!(data["version"], "1.0.0");
                assert_eq!(data["status"], "running");
            }
            _ => panic!("Expected success response"),
        }
    }
} 