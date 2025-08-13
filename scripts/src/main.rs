mod types;
mod metrics;
mod tests;
mod reporting;
mod config;
mod stress_testing;
mod metrics_collector;
mod metrics_server;
mod statsd_client;

use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use tracing::{info, warn, error};
use tracing_subscriber;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;

use types::WaveCoreTestResult;
use tests::*;
use reporting::generate_report;
use config::{Config, ConfigManager};
use stress_testing::{StressTestManager, TestFunction};
use metrics::WaveCoreMetrics;
use statsd_client::StatsDConfig;

#[derive(Parser)]
#[command(name = "wavecore-test-suite")]
#[command(about = "Comprehensive test suite for WaveCore library")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Configuration file path
    #[arg(short, long, default_value = "config.yml")]
    config: String,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run individual test
    Test {
        /// Test name to run
        test_name: String,
    },
    
    /// Run all tests
    All,
    
    /// Run core functional tests
    Core,
    
    /// Run performance tests
    Performance,
    
    /// Run extension tests
    Extensions,
    
    /// Run validation tests
    Validation,
    
    /// Run stress tests
    Stress,
    
    /// Run MSR (Max Sustainable Rate) search
    Msr,
    
    /// Show configuration
    Config,
    
    /// Validate configuration
    Validate,
    
    /// Start metrics server for Prometheus
    MetricsServer {
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        
        /// Port to bind to
        #[arg(short, long, default_value = "4040")]
        port: u16,
    },
    
    /// Debug StatsD connection
    Debug,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global metrics collector
    metrics_collector::init_global_collector();
    
    // Initialize global StatsD client
    let statsd_config = statsd_client::StatsDConfig {
        host: "127.0.0.1".to_string(),
        port: 8125,
        prefix: "wavecore".to_string(),
        batch_size: 1, // Flush immediately
        flush_interval_ms: 100, // Flush every 100ms
    };
    statsd_client::init_statsd(statsd_config)?;
    
    let cli = Cli::parse();
    
    // Load configuration
    let config_manager = ConfigManager::new("config.yml".to_string())?;
    let config = config_manager.get_config();
    
    // Validate configuration
    if let Err(errors) = config.validate() {
        println!("❌ Configuration validation failed:");
        for error in errors {
            println!("   - {}", error);
        }
        return Ok(());
    }
    
    match &cli.command {
        Commands::Test { test_name } => {
            match test_name.as_str() {
                "hydrostatics" => {
                    println!("🧪 Running hydrostatics test...");
                    let result = test_hydrostatics()?;
                    println!("✅ Hydrostatics test completed successfully");
                    println!("   - Problem Type: {}", result.problem_type);
                    println!("   - Mesh Tier: {}", result.mesh_tier);
                    println!("   - Has Field: {}", result.has_field);
                    println!("   - Num Bodies: {}", result.num_bodies);
                    println!("   - Depth Type: {}", result.depth_type);
                    println!("   - IF Removal: {}", result.if_removal);
                    println!("   - Forward Speed: {}", result.forward_speed);
                    println!("   - P50 Latency: {:.2} ms", result.calculate_p50());
                    println!("   - P95 Latency: {:.2} ms", result.calculate_p95());
                    println!("   - P99 Latency: {:.2} ms", result.calculate_p99());
                    println!("   - Throughput: {:.2} ops/sec", result.throughput);
                }
                "radiation" => {
                    println!("🧪 Running radiation test...");
                    let result = test_radiation()?;
                    println!("✅ Radiation test completed successfully");
                    println!("   - P50 Latency: {:.2} ms", result.calculate_p50());
                    println!("   - P95 Latency: {:.2} ms", result.calculate_p95());
                    println!("   - P99 Latency: {:.2} ms", result.calculate_p99());
                    println!("   - Throughput: {:.2} ops/sec", result.throughput);
                }
                "diffraction" => {
                    println!("🧪 Running diffraction test...");
                    let result = test_diffraction()?;
                    println!("✅ Diffraction test completed successfully");
                    println!("   - P50 Latency: {:.2} ms", result.calculate_p50());
                    println!("   - P95 Latency: {:.2} ms", result.calculate_p95());
                    println!("   - P99 Latency: {:.2} ms", result.calculate_p99());
                    println!("   - Throughput: {:.2} ops/sec", result.throughput);
                }
                "rao" => {
                    println!("🧪 Running RAO test...");
                    let result = test_rao()?;
                    println!("✅ RAO test completed successfully");
                    println!("   - P50 Latency: {:.2} ms", result.calculate_p50());
                    println!("   - P95 Latency: {:.2} ms", result.calculate_p95());
                    println!("   - P99 Latency: {:.2} ms", result.calculate_p99());
                    println!("   - Throughput: {:.2} ops/sec", result.throughput);
                }
                _ => {
                    println!("❌ Unknown test: {}", test_name);
                    print_usage();
                    return Ok(());
                }
            }
        }
        Commands::All => {
            println!("🧪 Running all tests...");
            let tests = vec!["hydrostatics", "radiation", "diffraction", "rao"];
            let mut results = Vec::new();
            
            for test in &tests {
                println!("  Running {} test...", test);
                match *test {
                    "hydrostatics" => results.push(test_hydrostatics()?),
                    "radiation" => results.push(test_radiation()?),
                    "diffraction" => results.push(test_diffraction()?),
                    "rao" => results.push(test_rao()?),
                    _ => {}
                }
            }
            
            println!("✅ All tests completed successfully!");
            println!("📊 Summary:");
            for (i, result) in results.iter().enumerate() {
                println!("   {}. {}: P50={:.2}ms, P95={:.2}ms, P99={:.2}ms, TPS={:.2}",
                    i + 1, tests[i], result.calculate_p50(), result.calculate_p95(), 
                    result.calculate_p99(), result.throughput);
            }
        }
        Commands::Core => {
            println!("🧪 Running core functional tests...");
            
            // Initialize global metrics collector for core tests
            metrics_collector::init_global_collector();
            
            let tests = vec!["hydrostatics", "radiation", "diffraction", "rao"];
            let mut results = Vec::new();
            
            for test in &tests {
                println!("  Running {} test...", test);
                match *test {
                    "hydrostatics" => results.push(test_hydrostatics()?),
                    "radiation" => results.push(test_radiation()?),
                    "diffraction" => results.push(test_diffraction()?),
                    "rao" => results.push(test_rao()?),
                    _ => {}
                }
            }
            
            println!("✅ Core tests completed successfully!");
        }
        Commands::Performance => {
            println!("🧪 Running performance tests...");
            
            // Initialize global metrics collector for performance tests
            metrics_collector::init_global_collector();
            
            let tests = vec!["green_functions", "multi_body", "field_computations"];
            
            for test in tests {
                println!("  Running {} test...", test);
                match test {
                    "green_functions" => {
                        let result = test_green_functions()?;
                        println!("    ✅ Green functions: P50={:.2}ms, TPS={:.2}", 
                            result.calculate_p50(), result.throughput);
                    }
                    "multi_body" => {
                        let result = test_multi_body()?;
                        println!("    ✅ Multi-body: P50={:.2}ms, TPS={:.2}", 
                            result.calculate_p50(), result.throughput);
                    }
                    "field_computations" => {
                        let result = test_field_computations()?;
                        println!("    ✅ Field computations: P50={:.2}ms, TPS={:.2}", 
                            result.calculate_p50(), result.throughput);
                    }
                    _ => {}
                }
            }
            
            println!("✅ Performance tests completed successfully!");
        }
        Commands::Extensions => {
            println!("🧪 Running extension tests...");
            
            // Initialize global metrics collector for extension tests
            metrics_collector::init_global_collector();
            
            let tests = vec!["finite_depth", "irregular_frequency", "forward_speed"];
            
            for test in tests {
                println!("  Running {} test...", test);
                match test {
                    "finite_depth" => {
                        let result = test_finite_depth()?;
                        println!("    ✅ Finite depth: P50={:.2}ms, TPS={:.2}", 
                            result.calculate_p50(), result.throughput);
                    }
                    "irregular_frequency" => {
                        let result = test_irregular_frequency()?;
                        println!("    ✅ Irregular frequency: P50={:.2}ms, TPS={:.2}", 
                            result.calculate_p50(), result.throughput);
                    }
                    "forward_speed" => {
                        let result = test_forward_speed()?;
                        println!("    ✅ Forward speed: P50={:.2}ms, TPS={:.2}", 
                            result.calculate_p50(), result.throughput);
                    }
                    _ => {}
                }
            }
            
            println!("✅ Extension tests completed successfully!");
        }
        Commands::Validation => {
            println!("🧪 Running validation tests...");
            
            // Initialize global metrics collector for validation tests
            metrics_collector::init_global_collector();
            
            let result = test_numerical_validation()?;
            println!("✅ Validation test completed successfully");
            println!("   - P50 Latency: {:.2} ms", result.calculate_p50());
            println!("   - P95 Latency: {:.2} ms", result.calculate_p95());
            println!("   - P99 Latency: {:.2} ms", result.calculate_p99());
            println!("   - Throughput: {:.2} ops/sec", result.throughput);
        }
        Commands::Stress => {
            println!("🧪 Running stress tests...");
            
            // Initialize global metrics collector for stress tests
            metrics_collector::init_global_collector();
            
            // Create test function for stress testing
            struct StressTestFunction;
            impl TestFunction for StressTestFunction {
                fn run(&self, load_multiplier: f64) -> Result<WaveCoreMetrics, Box<dyn std::error::Error>> {
                    // Run hydrostatics test as baseline
                    let base_result = test_hydrostatics()?;
                    
                    // Apply load multiplier by running multiple iterations
                    let iterations = (load_multiplier * 10.0) as usize;
                    let mut total_duration = 0.0;
                    
                    for _ in 0..iterations {
                        let start = std::time::Instant::now();
                        test_hydrostatics()?;
                        total_duration += start.elapsed().as_secs_f64();
                    }
                    
                    // Calculate adjusted metrics
                    let avg_duration = total_duration / iterations as f64;
                    let throughput = iterations as f64 / total_duration;
                    
                    let mut metrics = WaveCoreMetrics::new();
                    metrics.latencies = vec![avg_duration * 1000.0]; // Convert to ms
                    metrics.throughput = throughput;
                    Ok(metrics)
                }
            }
            
            let test_fn = Arc::new(StressTestFunction);
            let stress_manager = StressTestManager::new(config.clone());
            
            match stress_manager.run_stress_tests(test_fn) {
                Ok(results) => {
                    println!("✅ Stress tests completed successfully!");
                    for result in results {
                        println!("   - Pattern: {}", result.pattern_name);
                        println!("   - Duration: {} seconds", result.duration_seconds);
                        println!("   - Success: {}", if result.success { "✅" } else { "❌" });
                        if let Some(msr) = result.max_sustainable_rate {
                            println!("   - MSR: {:.2}x", msr);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Stress tests failed: {}", e);
                }
            }
        }
        Commands::Msr => {
            println!("🎯 Running MSR (Max Sustainable Rate) search...");
            
            // Initialize global metrics collector for MSR search
            metrics_collector::init_global_collector();
            
            // Create test function for MSR search
            struct MSRTestFunction;
            impl TestFunction for MSRTestFunction {
                fn run(&self, load_multiplier: f64) -> Result<WaveCoreMetrics, Box<dyn std::error::Error>> {
                    // Run hydrostatics test as baseline
                    let base_result = test_hydrostatics()?;
                    
                    // Apply load multiplier by running multiple iterations
                    let iterations = (load_multiplier * 10.0) as usize;
                    let mut total_duration = 0.0;
                    
                    for _ in 0..iterations {
                        let start = std::time::Instant::now();
                        test_hydrostatics()?;
                        total_duration += start.elapsed().as_secs_f64();
                    }
                    
                    // Calculate adjusted metrics
                    let avg_duration = total_duration / iterations as f64;
                    let throughput = iterations as f64 / total_duration;
                    
                    let mut metrics = WaveCoreMetrics::new();
                    metrics.latencies = vec![avg_duration * 1000.0]; // Convert to ms
                    metrics.throughput = throughput;
                    Ok(metrics)
                }
            }
            
            let test_fn = Arc::new(MSRTestFunction);
            let stress_manager = StressTestManager::new(config.clone());
            
            match stress_manager.run_msr_search(test_fn) {
                Ok(msr) => {
                    println!("✅ MSR search completed successfully!");
                    println!("   - Max Sustainable Rate: {:.2}x", msr);
                }
                Err(e) => {
                    println!("❌ MSR search failed: {}", e);
                }
            }
        }
        Commands::Config => {
            println!("📋 Configuration Summary:");
            println!("   Environment: {}", config.global.environment);
            println!("   Log Level: {}", config.global.log_level);
            println!("   Output Directory: {}", config.global.output_dir);
            println!("   Hardware Baseline: {} ({} cores, {} GB RAM)", 
                config.hardware.baseline.cpu, 
                config.hardware.baseline.cores, 
                config.hardware.baseline.ram_gb);
            
            println!("\n📊 Test Categories:");
            println!("   Functional: {}", if config.test_categories.functional.enabled { "✅ Enabled" } else { "❌ Disabled" });
            println!("   Performance: {}", if config.test_categories.performance.enabled { "✅ Enabled" } else { "❌ Disabled" });
            println!("   Extensions: {}", if config.test_categories.extensions.enabled { "✅ Enabled" } else { "❌ Disabled" });
            println!("   Validation: {}", if config.test_categories.validation.enabled { "✅ Enabled" } else { "❌ Disabled" });
            
            println!("\n⚡ Stress Testing:");
            println!("   Enabled: {}", if config.stress_testing.enabled { "✅ Yes" } else { "❌ No" });
            println!("   SLO P50 Latency: {} ms", config.stress_testing.slo.p50_latency_ms);
            println!("   SLO P95 Latency: {} ms", config.stress_testing.slo.p95_latency_ms);
            println!("   SLO P99 Latency: {} ms", config.stress_testing.slo.p99_latency_ms);
            println!("   SLO Error Rate: {}%", config.stress_testing.slo.error_rate_percent);
            println!("   SLO Throughput: {} ops/sec", config.stress_testing.slo.throughput_sps);
            
            println!("\n🔍 Load Patterns:");
            println!("   Linear Ramp: {}", if config.stress_testing.load_patterns.linear_ramp.enabled { "✅ Enabled" } else { "❌ Disabled" });
            println!("   Step Load: {}", if config.stress_testing.load_patterns.step_load.enabled { "✅ Enabled" } else { "❌ Disabled" });
            println!("   Wave Load: {}", if config.stress_testing.load_patterns.wave_load.enabled { "✅ Enabled" } else { "❌ Disabled" });
            println!("   Stress-to-Failure: {}", if config.stress_testing.load_patterns.stress_to_failure.enabled { "✅ Enabled" } else { "❌ Disabled" });
            
            println!("\n🎯 MSR Search:");
            println!("   Enabled: {}", if config.msr_search.enabled { "✅ Yes" } else { "❌ No" });
            println!("   Algorithm: {}", config.msr_search.algorithm);
            
            println!("\n📈 Monitoring:");
            println!("   Enabled: {}", if config.monitoring.enabled { "✅ Yes" } else { "❌ No" });
            println!("   Latency Collection: {}", if config.monitoring.metrics.latency.enabled { "✅ Enabled" } else { "❌ Disabled" });
            println!("   Throughput Collection: {}", if config.monitoring.metrics.throughput.enabled { "✅ Enabled" } else { "❌ Disabled" });
            println!("   Resource Monitoring: {}", if config.monitoring.metrics.resource_usage.enabled { "✅ Enabled" } else { "❌ Disabled" });
            
            println!("\n📊 Test Matrix:");
            println!("   Mesh Sizes: {} tiers", config.test_matrix.mesh_sizes.len());
            println!("   Frequencies: {} ranges", config.test_matrix.frequencies.len());
            println!("   Problem Types: {} types", config.test_matrix.problem_types.len());
            
            println!("\n📄 Reporting:");
            println!("   Enabled: {}", if config.reporting.enabled { "✅ Yes" } else { "❌ No" });
            println!("   Formats: {}", config.reporting.formats.join(", "));
            println!("   Sections: {} sections", config.reporting.sections.len());
            println!("   Visualizations: {} charts", config.reporting.visualizations.len());
            
            println!("\n🔧 Advanced Settings:");
            println!("   Note: Advanced settings are configured in config.yml");
            println!("   - Parallel execution, caching, and debug mode can be configured");
            println!("   - Custom parameters for green functions, solver type, and precision available");
        }
        Commands::Validate => {
            println!("🔍 Validating configuration...");
            match config.validate() {
                Ok(_) => println!("✅ Configuration is valid"),
                Err(errors) => {
                    println!("❌ Configuration validation failed:");
                    for error in errors {
                        println!("   - {}", error);
                    }
                }
            }
        }
        Commands::MetricsServer { host, port } => {
            println!("🚀 Starting WaveCore Test Suite Metrics Server...");
            println!("📊 Prometheus metrics will be available at: http://{}:{}/metrics", host, port);
            println!("🏥 Health check at: http://{}:{}/health", host, port);
            println!("🌐 Root endpoint at: http://{}:{}", host, port);
            println!();
            println!("💡 To view metrics in Grafana:");
            println!("   1. Open Grafana at http://localhost:3000 (admin/wavecore123)");
            println!("   2. Add Prometheus data source: http://prometheus:9090");
            println!("   3. Import dashboard from monitoring/grafana/dashboards/");
            println!();
            println!("🔄 Starting background tasks...");
            
            // Start background tasks
            // Background tasks are now handled within the metrics server
            
            // Start metrics server
            metrics_server::start_metrics_server(host, *port).await?;
        }
        Commands::Debug => {
            println!("🔍 Debug Test - Checking StatsD Connection");
            
            // Send test metrics
            let mut tags = HashMap::new();
            tags.insert("test_type".to_string(), "debug".to_string());
            
            println!("📤 Sending test metrics...");
            
            // Send counter
            statsd_client::counter("test.requests", 1.0, Some(tags.clone()));
            println!("  ✅ Sent counter: test.requests");
            
            // Send gauge
            statsd_client::gauge("test.p50_latency_ms", 100.0, Some(tags.clone()));
            println!("  ✅ Sent gauge: test.p50_latency_ms");
            
            // Send timer
            statsd_client::timer("test.duration", 500.0, Some(tags.clone()));
            println!("  ✅ Sent timer: test.duration");
            
            // Wait for metrics to be sent
            std::thread::sleep(Duration::from_millis(200));
            
            println!("✅ Debug test completed");
            println!("📊 Check Grafana: http://localhost:3000");
            println!("📊 Check StatsD: http://localhost:8080");
        }
    }
    
    Ok(())
}

fn print_usage() {
    println!("Usage: wavecore_test_suite <command> [options]");
    println!();
    println!("Available commands:");
    println!("  test <test_name>     Run individual test");
    println!("  all                  Run all tests");
    println!("  core                 Run core functional tests");
    println!("  performance          Run performance tests");
    println!("  extensions           Run extension tests");
    println!("  validation           Run validation tests");
    println!("  stress               Run stress tests");
    println!("  msr                  Run MSR search");
    println!("  config               Show configuration");
    println!("  validate             Validate configuration");
    println!("  metrics-server       Start metrics server for Prometheus");
    println!();
    println!("Available individual tests:");
    println!("  hydrostatics");
    println!("  radiation");
    println!("  diffraction");
    println!("  rao");
    println!("  green_functions");
    println!("  multi_body");
    println!("  finite_depth");
    println!("  irregular_frequency");
    println!("  forward_speed");
    println!("  field_computations");
    println!("  numerical_validation");
    println!();
    println!("Options:");
    println!("  -c, --config <file>  Configuration file (default: config.yml)");
    println!("  -d, --debug          Enable debug logging");
    println!("  -h, --help           Show this help message");
}

fn run_single_test<F>(test_name: &str, test_fn: F, results: &mut Vec<WaveCoreTestResult>) 
where 
    F: FnOnce() -> Result<crate::metrics::WaveCoreMetrics, Box<dyn std::error::Error>>
{
    match test_fn() {
        Ok(metrics) => {
            let result = WaveCoreTestResult {
                test_name: test_name.to_string(),
                timestamp: Local::now(),
                duration_ms: 0.0,
                p50_latency_ms: metrics.calculate_p50(),
                p95_latency_ms: metrics.calculate_p95(),
                p99_latency_ms: metrics.calculate_p99(),
                throughput_ops_per_sec: metrics.throughput,
                memory_usage_mb: metrics.memory_usage / 1024 / 1024,
                num_panels: metrics.num_panels,
                matrix_size: metrics.matrix_size,
                num_frequencies: metrics.num_frequencies,
                success: true,
                error_message: None,
            };
            results.push(result);
        }
        Err(e) => {
            results.push(WaveCoreTestResult {
                test_name: test_name.to_string(),
                timestamp: Local::now(),
                duration_ms: 0.0,
                p50_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                throughput_ops_per_sec: 0.0,
                memory_usage_mb: 0,
                num_panels: 0,
                matrix_size: 0,
                num_frequencies: 0,
                success: false,
                error_message: Some(e.to_string()),
            });
        }
    }
}

fn run_all_tests(results: &mut Vec<WaveCoreTestResult>) {
    println!("Running: Hydrostatics");
    run_single_test("Hydrostatics", test_hydrostatics, results);
    println!("Running: Radiation");
    run_single_test("Radiation", test_radiation, results);
    println!("Running: Diffraction");
    run_single_test("Diffraction", test_diffraction, results);
    println!("Running: RAO");
    run_single_test("RAO", test_rao, results);
    println!("Running: Green Functions");
    run_single_test("Green Functions", test_green_functions, results);
    println!("Running: Multi-body");
    run_single_test("Multi-body", test_multi_body, results);
    println!("Running: Finite Depth");
    run_single_test("Finite Depth", test_finite_depth, results);
    println!("Running: Irregular Frequency");
    run_single_test("Irregular Frequency", test_irregular_frequency, results);
    println!("Running: Forward Speed");
    run_single_test("Forward Speed", test_forward_speed, results);
    println!("Running: Field Computations");
    run_single_test("Field Computations", test_field_computations, results);
    println!("Running: Numerical Validation");
    run_single_test("Numerical Validation", test_numerical_validation, results);
}

fn run_core_tests(results: &mut Vec<WaveCoreTestResult>) {
    println!("Running: Hydrostatics");
    run_single_test("Hydrostatics", test_hydrostatics, results);
    println!("Running: Radiation");
    run_single_test("Radiation", test_radiation, results);
    println!("Running: Diffraction");
    run_single_test("Diffraction", test_diffraction, results);
    println!("Running: RAO");
    run_single_test("RAO", test_rao, results);
}

fn run_performance_tests(results: &mut Vec<WaveCoreTestResult>) {
    println!("Running: Green Functions");
    run_single_test("Green Functions", test_green_functions, results);
    println!("Running: Multi-body");
    run_single_test("Multi-body", test_multi_body, results);
    println!("Running: Field Computations");
    run_single_test("Field Computations", test_field_computations, results);
}

fn run_extension_tests(results: &mut Vec<WaveCoreTestResult>) {
    println!("Running: Finite Depth");
    run_single_test("Finite Depth", test_finite_depth, results);
    println!("Running: Irregular Frequency");
    run_single_test("Irregular Frequency", test_irregular_frequency, results);
    println!("Running: Forward Speed");
    run_single_test("Forward Speed", test_forward_speed, results);
}

// Stress testing functions temporarily disabled due to async compatibility issues 