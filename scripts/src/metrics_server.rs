use actix_web::{web, App, HttpServer, HttpResponse, Result};
use crate::metrics_collector;
use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::System;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct UpdateMetricsRequest {
    test_requests: Option<u64>,
    test_errors: Option<u64>,
    p50_latency: Option<f64>,
    p95_latency: Option<f64>,
    p99_latency: Option<f64>,
    throughput: Option<f64>,
    stress_load_multiplier: Option<f64>,
    stress_phase: Option<u32>,
    msr: Option<f64>,
}

/// HTTP handler for metrics endpoint
async fn metrics_handler() -> Result<HttpResponse> {
    // Use global collector for synchronous access
    unsafe {
        if let Some(collector) = &crate::metrics_collector::GLOBAL_COLLECTOR {
            match collector.get_metrics() {
                Ok(metrics) => Ok(HttpResponse::Ok()
                    .content_type("text/plain; version=0.0.4; charset=utf-8")
                    .body(metrics)),
                Err(e) => Ok(HttpResponse::InternalServerError()
                    .body(format!("Error getting metrics: {}", e)))
            }
        } else {
            Ok(HttpResponse::InternalServerError()
                .body("Global collector not initialized"))
        }
    }
}

/// HTTP handler for health check endpoint
async fn health_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "wavecore-metrics-server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// HTTP handler for updating metrics manually
async fn update_metrics_handler(req: web::Json<UpdateMetricsRequest>) -> Result<HttpResponse> {
    unsafe {
        if let Some(collector) = &mut crate::metrics_collector::GLOBAL_COLLECTOR {
            // Update metrics based on request
            if let Some(requests) = req.test_requests {
                for _ in 0..requests {
                    collector.record_test_request("manual_update");
                }
            }
            
            if let Some(errors) = req.test_errors {
                for _ in 0..errors {
                    collector.record_test_error("manual_update");
                }
            }
            
            if let Some(p50) = req.p50_latency {
                if let Some(p95) = req.p95_latency {
                    if let Some(p99) = req.p99_latency {
                        if let Some(throughput) = req.throughput {
                            collector.update_performance_metrics(p50, p95, p99, throughput);
                        }
                    }
                }
            }
            
            if let Some(load_multiplier) = req.stress_load_multiplier {
                collector.update_stress_load_multiplier(load_multiplier);
            }
            
            if let Some(phase) = req.stress_phase {
                collector.update_stress_phase(phase);
            }
            
            if let Some(msr) = req.msr {
                collector.update_msr(msr);
            }
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Metrics updated successfully"
            })))
        } else {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Global collector not initialized"
            })))
        }
    }
}

/// Background task for collecting system metrics
async fn collect_system_metrics() {
    let mut sys = System::new_all();
    
    loop {
        sys.refresh_all();
        
        unsafe {
            if let Some(collector) = &mut crate::metrics_collector::GLOBAL_COLLECTOR {
                let cpu_usage = sys.global_cpu_info().cpu_usage();
                let memory_usage = sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0;
                
                collector.update_system_metrics(cpu_usage as f64, memory_usage, sys.used_memory() as f64, sys.total_memory() as f64);
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

/// Start the metrics HTTP server
pub async fn start_metrics_server(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global collector for synchronous access
    metrics_collector::init_global_collector();
    
    println!("🚀 Starting metrics server on {}:{}", host, port);
    println!("📊 Metrics available at: http://{}:{}/metrics", host, port);
    println!("🏥 Health check at: http://{}:{}/health", host, port);
    println!("🔄 Update metrics at: http://{}:{}/update", host, port);
    println!("💡 To view metrics in Grafana:");
    println!("   1. Open Grafana at http://localhost:3000 (admin/wavecore123)");
    println!("   2. Add Prometheus data source: http://prometheus:9090");
    println!("   3. Import dashboard from monitoring/grafana/dashboards/");
    
    // Start background system metrics collection
    tokio::spawn(collect_system_metrics());
    
    HttpServer::new(|| {
        App::new()
            .route("/metrics", web::get().to(metrics_handler))
            .route("/health", web::get().to(health_handler))
            .route("/update", web::post().to(update_metrics_handler))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;
    
    Ok(())
} 