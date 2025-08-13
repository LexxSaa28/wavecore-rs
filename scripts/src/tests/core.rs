use std::time::{Duration, Instant};
use crate::metrics::WaveCoreMetrics;
use crate::statsd_client::{self, StatsDConfig};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Local};
use std::env;
use std::io::{Read, Write};
use lazy_static::lazy_static;
use std::sync::Mutex;

// Global timestamp for the entire test session
lazy_static! {
    static ref GLOBAL_TIMESTAMP: Mutex<Option<String>> = Mutex::new(None);
}

/// Get or create global timestamp for the test session
fn get_global_timestamp() -> String {
    let mut timestamp_guard = GLOBAL_TIMESTAMP.lock().unwrap();
    
    if let Some(ref timestamp) = *timestamp_guard {
        timestamp.clone()
    } else {
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
        *timestamp_guard = Some(timestamp.clone());
        timestamp
    }
}

/// Save test results to shared timestamp folder
fn save_test_results(test_name: &str, metrics: &WaveCoreMetrics, total_duration: Duration, 
                    statsd_duration: Duration, setup_duration: Duration, 
                    execution_duration: Duration, metrics_duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
    
    // Get global timestamp for the test session
    let timestamp = get_global_timestamp();
    
    // Create test_results directory if it doesn't exist
    let test_results_dir = Path::new("test_results");
    if !test_results_dir.exists() {
        fs::create_dir(test_results_dir)?;
    }
    
    // Create timestamp subdirectory
    let timestamp_dir = test_results_dir.join(&timestamp);
    if !timestamp_dir.exists() {
        fs::create_dir(&timestamp_dir)?;
    }
    
    // Calculate performance metrics
    let p50 = metrics.get_p_percentile_latency(50.0);
    let p95 = metrics.get_p_percentile_latency(95.0);
    let p99 = metrics.get_p_percentile_latency(99.0);
    let throughput = metrics.get_throughput_ops_per_sec(execution_duration);
    
    // Create summary markdown file
    let summary_content = format!(
        "# {} Test Results\n\n\
        **Test Date:** {}\n\n\
        **Test Duration:** {:?}\n\n\
        **Total Panels:** {}\n\n\
        **P50 Latency:** {:.2} ms\n\n\
        **P95 Latency:** {:.2} ms\n\n\
        **P99 Latency:** {:.2} ms\n\n\
        **Throughput:** {:.2} ops/sec\n\n\
        **Problem Type:** {}\n\n\
        **Mesh Tier:** {}\n\n\n\
        ## Performance Breakdown\n\n\
        - **StatsD Initialization:** {:?}\n\n\
        - **Test Setup:** {:?}\n\n\
        - **Main Execution:** {:?}\n\n\
        - **Metrics Calculation:** {:?}\n\n",
        test_name.to_uppercase(),
        Local::now().format("%Y-%m-%d %H:%M:%S %Z"),
        total_duration,
        metrics.num_panels,
        p50,
        p95,
        p99,
        throughput,
        metrics.problem_type,
        metrics.mesh_tier,
        statsd_duration,
        setup_duration,
        execution_duration,
        metrics_duration
    );
    
    let summary_path = timestamp_dir.join(format!("{}_summary.md", test_name));
    fs::write(&summary_path, summary_content)?;
    
    // Create detailed JSON file
    let detailed_data = serde_json::json!({
        "test_name": test_name,
        "timestamp": Local::now().to_rfc3339(),
        "duration": {
            "total": total_duration.as_secs_f64(),
            "statsd_init": statsd_duration.as_secs_f64(),
            "setup": setup_duration.as_secs_f64(),
            "execution": execution_duration.as_secs_f64(),
            "metrics_calculation": metrics_duration.as_secs_f64()
        },
        "performance": {
            "p50_latency_ms": p50,
            "p95_latency_ms": p95,
            "p99_latency_ms": p99,
            "throughput_ops_per_sec": throughput,
            "total_panels": metrics.num_panels
        },
        "configuration": {
            "problem_type": metrics.problem_type,
            "mesh_tier": metrics.mesh_tier,
            "has_field": metrics.has_field,
            "num_bodies": metrics.num_bodies,
            "depth_type": metrics.depth_type,
            "if_removal": metrics.if_removal,
            "forward_speed": metrics.forward_speed
        }
    });
    
    let detailed_path = timestamp_dir.join(format!("{}_detailed.json", test_name));
    fs::write(&detailed_path, serde_json::to_string_pretty(&detailed_data)?)?;
    
    // Create or update comprehensive test summary
    create_comprehensive_summary(&timestamp_dir, test_name, &metrics, total_duration, p50, p95, p99, throughput)?;
    
    println!("  💾 [{}] Results saved to: test_results/{}/", test_name.to_uppercase(), timestamp);
    Ok(())
}

/// Create comprehensive test summary with table for all tests
fn create_comprehensive_summary(timestamp_dir: &Path, test_name: &str, metrics: &WaveCoreMetrics, 
                               total_duration: Duration, p50: f64, p95: f64, p99: f64, throughput: f64) 
                               -> Result<(), Box<dyn std::error::Error>> {
    
    let summary_file = timestamp_dir.join("test_summary.md");
    println!("  🔍 [{}] Creating comprehensive summary at: {:?}", test_name.to_uppercase(), summary_file);
    
    // Read existing summary if it exists
    let mut existing_tests = Vec::new();
    if summary_file.exists() {
        println!("  📖 [{}] Reading existing summary file", test_name.to_uppercase());
        if let Ok(content) = fs::read_to_string(&summary_file) {
            // Parse existing tests from table
            for line in content.lines() {
                if line.starts_with("|") && !line.contains("---") && !line.contains("Test Name") {
                    let parts: Vec<&str> = line.split("|").collect();
                    if parts.len() >= 8 {
                        existing_tests.push((
                            parts[1].trim().to_string(),
                            parts[2].trim().to_string(),
                            parts[3].trim().to_string(),
                            parts[4].trim().to_string(),
                            parts[5].trim().to_string(),
                            parts[6].trim().to_string(),
                            parts[7].trim().to_string()
                        ));
                    }
                }
            }
            println!("  📊 [{}] Found {} existing tests in summary", test_name.to_uppercase(), existing_tests.len());
        } else {
            println!("  ⚠️  [{}] Failed to read existing summary file", test_name.to_uppercase());
        }
    } else {
        println!("  📝 [{}] Creating new summary file", test_name.to_uppercase());
    }
    
    // Add current test
    existing_tests.push((
        test_name.to_string(),
        format!("{:.2}s", total_duration.as_secs_f64()),
        format!("{:.2}ms", p50),
        format!("{:.2}ms", p95),
        format!("{:.2}ms", p99),
        format!("{:.2} ops/sec", throughput),
        metrics.num_panels.to_string()
    ));
    
    println!("  ➕ [{}] Added current test to summary", test_name.to_uppercase());
    
    // Create comprehensive summary
    let now: DateTime<Local> = Local::now();
    let global_timestamp = get_global_timestamp();
    let mut summary_content = format!(
        "# WaveCore Test Suite - Comprehensive Summary\n\n\
        **Test Session Date:** {}\n\n\
        **Total Tests:** {}\n\n\
        **Session Duration:** {:.2}s\n\n\
        ## Test Results Overview\n\n\
        | Test Name | Duration | P50 Latency | P95 Latency | P99 Latency | Throughput | Total Panels |\n\
        |-----------|----------|-------------|-------------|-------------|------------|--------------|\n",
        now.format("%Y-%m-%d %H:%M:%S %Z"),
        existing_tests.len(),
        existing_tests.iter().map(|t| t.1.replace("s", "").parse::<f64>().unwrap_or(0.0)).sum::<f64>()
    );
    
    // Add table rows
    for test in &existing_tests {
        summary_content.push_str(&format!("| {} | {} | {} | {} | {} | {} | {} |\n",
            test.0, test.1, test.2, test.3, test.4, test.5, test.6));
    }
    
    // Add summary statistics
    let total_duration: f64 = existing_tests.iter()
        .map(|t| t.1.replace("s", "").parse::<f64>().unwrap_or(0.0))
        .sum();
    
    let avg_p50: f64 = existing_tests.iter()
        .map(|t| t.2.replace("ms", "").parse::<f64>().unwrap_or(0.0))
        .sum::<f64>() / existing_tests.len() as f64;
    
    let avg_throughput: f64 = existing_tests.iter()
        .map(|t| t.5.replace(" ops/sec", "").parse::<f64>().unwrap_or(0.0))
        .sum::<f64>() / existing_tests.len() as f64;
    
    summary_content.push_str(&format!("\n## Summary Statistics\n\n\
        - **Total Session Duration:** {:.2}s\n\
        - **Average P50 Latency:** {:.2}ms\n\
        - **Average Throughput:** {:.2} ops/sec\n\
        - **Total Tests Completed:** {}\n\n",
        total_duration, avg_p50, avg_throughput, existing_tests.len()));
    
    // Add individual test details
    summary_content.push_str("## Individual Test Details\n\n");
    for test in &existing_tests {
        summary_content.push_str(&format!("### {}\n\n\
        - **Duration:** {}\n\
        - **P50 Latency:** {}\n\
        - **P95 Latency:** {}\n\
        - **P99 Latency:** {}\n\
        - **Throughput:** {}\n\
        - **Total Panels:** {}\n\n",
        test.0, test.1, test.2, test.3, test.4, test.5, test.6));
    }
    
    // Add footer
    summary_content.push_str(&format!("---\n\n\
        **Generated:** {}\n\
        **Test Suite Version:** WaveCore v1.0\n\
        **Location:** test_results/{}/\n\n",
        now.format("%Y-%m-%d %H:%M:%S %Z"),
        timestamp_dir.file_name().unwrap().to_str().unwrap()));
    
    println!("  💾 [{}] Writing comprehensive summary with {} tests", test_name.to_uppercase(), existing_tests.len());
    match fs::write(&summary_file, summary_content) {
        Ok(_) => println!("  ✅ [{}] Comprehensive summary written successfully", test_name.to_uppercase()),
        Err(e) => println!("  ❌ [{}] Failed to write comprehensive summary: {}", test_name.to_uppercase(), e),
    }
    
    Ok(())
}

/// Test 1: Hydrostatics (buoyancy & stability) - WITH OFFICIAL STATSD
pub fn test_hydrostatics() -> Result<WaveCoreMetrics, Box<dyn std::error::Error>> {
    println!("🔄 [HYDROSTATICS] Starting test...");
    let test_start = Instant::now();
    
    // Phase 1: StatsD Initialization
    println!("📡 [HYDROSTATICS] Phase 1: Initializing StatsD client...");
    let statsd_start = Instant::now();
    
    // Initialize StatsD if not already done
    if statsd_client::get_statsd().is_none() || statsd_client::get_statsd().unwrap().lock().unwrap().is_none() {
        let config = StatsDConfig {
            host: "127.0.0.1".to_string(),
            port: 8125,
            prefix: "wavecore".to_string(),
            batch_size: 1,
            flush_interval_ms: 100,
        };
        // println!("  🔧 [HYDROSTATICS] Initializing StatsD client...");
        statsd_client::init_statsd(config)?;
        // println!("  ✅ [HYDROSTATICS] StatsD client initialized successfully");
    } else {
        // println!("  ✅ [HYDROSTATICS] StatsD client already initialized");
    }
    
    let statsd_duration = statsd_start.elapsed();
    // println!("📡 [HYDROSTATICS] StatsD initialization completed in {:?}", statsd_duration);
    
    // Phase 2: Test Setup
    println!("⚙️  [HYDROSTATICS] Phase 2: Setting up test parameters...");
    let setup_start = Instant::now();
    
    let mut metrics = WaveCoreMetrics::new()
        .with_problem_type("hydrostatics")
        .with_mesh_tier("T2");
    
    let num_panels = 40000; // Adjusted for ~1 minute runtime
    metrics.num_panels = num_panels;
    
    println!("  📋 [HYDROSTATICS] Test parameters: {} panels", num_panels);
    let setup_duration = setup_start.elapsed();
    println!("⚙️  [HYDROSTATICS] Test setup completed in {:?}", setup_duration);
    
    // Send test start metric
    let mut tags = HashMap::new();
    tags.insert("test_type".to_string(), "hydrostatics".to_string());
    tags.insert("mesh_tier".to_string(), "T2".to_string());
    
    statsd_client::counter("test.requests", 1.0, Some(tags.clone()));
    println!("  📊 [HYDROSTATICS] Sent test start metric");
    
    // Phase 3: Main Test Execution
    println!("🚀 [HYDROSTATICS] Phase 3: Starting main test execution...");
    let execution_start = Instant::now();
    
    // Simulate hydrostatics computation with realistic timing
    for i in 0..num_panels {
        let panel_start = Instant::now();
        
        // Progress logging every 100 panels
        if i % 100 == 0 {
            println!("  📈 [HYDROSTATICS] Processing panel {}/{} ({}%)", i, num_panels, (i * 100) / num_panels);
        }
        
        // Simulate panel area calculation
        let area = (i as f64 + 1.0) * 0.01; // 0.01 to 10.0 m²
        
        // Simulate center of gravity calculation
        let cog_x = (i as f64 % 100.0) * 0.1;
        let cog_y = ((i / 100) as f64) * 0.1;
        let cog_z = -2.0 + (i as f64 % 50.0) * 0.1;
        
        // Simulate buoyancy force calculation
        let density = 1025.0; // kg/m³
        let gravity = 9.81; // m/s²
        let buoyancy_force = area * density * gravity;
        
        // Simulate moment calculation
        let moment_x = buoyancy_force * cog_y;
        let moment_y = buoyancy_force * cog_x;
        let moment_z = buoyancy_force * cog_z;
        
        // Simulate some computation time
        let sleep_start = Instant::now();
        std::thread::sleep(Duration::from_micros(200 + (i % 200) as u64)); // Increased from 100-200μs to 200-400μs
        let sleep_duration = sleep_start.elapsed();
        
        let panel_duration = panel_start.elapsed();
        metrics.add_latency(panel_duration.as_secs_f64() * 1000.0);
        
        // Send panel metrics every 100 panels
        if i % 100 == 0 {
            println!("  📊 [HYDROSTATICS] Panel {}: processing={:?}, sleep={:?}", 
                     i, panel_duration, sleep_duration);
            
            // Send panel processing metrics
            statsd_client::timer("panel.processing_time", panel_duration.as_millis() as f64, Some(tags.clone()));
            statsd_client::counter("panel.count", 1.0, Some(tags.clone()));
            // println!("  ✅ [HYDROSTATICS] Panel {} processed successfully with StatsD", i);
        }
    }
    
    let execution_duration = execution_start.elapsed();
    println!("🚀 [HYDROSTATICS] Main execution completed in {:?}", execution_duration);
    
    // Phase 4: Final Metrics Calculation
    println!("📊 [HYDROSTATICS] Phase 4: Calculating final metrics...");
    let metrics_start = Instant::now();
    
    // Calculate final metrics
    let p50 = metrics.calculate_p50();
    let p95 = metrics.calculate_p95();
    let p99 = metrics.calculate_p99();
    let throughput = metrics.calculate_throughput();
    
    println!("  📈 [HYDROSTATICS] Calculated metrics: P50={:.2}ms, P95={:.2}ms, P99={:.2}ms, Throughput={:.2} ops/sec", 
             p50, p95, p99, throughput);
    
    // Send final performance metrics
    statsd_client::gauge("test.p50_latency_ms", p50, Some(tags.clone()));
    statsd_client::gauge("test.p95_latency_ms", p95, Some(tags.clone()));
    statsd_client::gauge("test.p99_latency_ms", p99, Some(tags.clone()));
    statsd_client::gauge("test.throughput_ops_per_sec", throughput, Some(tags.clone()));
    statsd_client::gauge("test.total_panels", num_panels as f64, Some(tags));
    
    println!("  📊 [HYDROSTATICS] Sent final performance metrics to StatsD");
    
    let metrics_duration = metrics_start.elapsed();
    println!("📊 [HYDROSTATICS] Final metrics calculation completed in {:?}", metrics_duration);
    
    // Phase 5: Test Completion
    let total_duration = test_start.elapsed();
    println!("✅ [HYDROSTATICS] Test completed successfully!");
    println!("📊 [HYDROSTATICS] Summary:");
    println!("   - Total panels processed: {}", num_panels);
    println!("   - Average latency (P50): {:.2} ms", p50);
    println!("   - Total test duration: {:?}", total_duration);
    println!("   - StatsD init: {:?}", statsd_duration);
    println!("   - Setup: {:?}", setup_duration);
    println!("   - Execution: {:?}", execution_duration);
    println!("   - Final metrics: {:?}", metrics_duration);
    
    // Save test results
    if let Err(e) = save_test_results("hydrostatics", &metrics, total_duration, 
                                    statsd_duration, setup_duration, execution_duration, metrics_duration) {
        eprintln!("⚠️  [HYDROSTATICS] Failed to save test results: {}", e);
    }
    
    Ok(metrics)
}

/// Test 2: Radiation problems (added mass & damping) - WITH STATSD
pub fn test_radiation() -> Result<WaveCoreMetrics, Box<dyn std::error::Error>> {
    println!("🔄 [RADIATION] Starting test...");
    let test_start = Instant::now();
    
    // Phase 1: StatsD Initialization
    println!("📡 [RADIATION] Phase 1: Initializing StatsD client...");
    let statsd_start = Instant::now();
    
    // Initialize StatsD if not already done
    if statsd_client::get_statsd().is_none() || statsd_client::get_statsd().unwrap().lock().unwrap().is_none() {
        let config = StatsDConfig {
            host: "127.0.0.1".to_string(),
            port: 8125,
            prefix: "wavecore".to_string(),
            batch_size: 1,
            flush_interval_ms: 100,
        };
        // println!("  🔧 [RADIATION] Initializing StatsD client...");
        statsd_client::init_statsd(config)?;
        // println!("  ✅ [RADIATION] StatsD client initialized successfully");
    } else {
        // println!("  ✅ [RADIATION] StatsD client already initialized");
    }
    
    let statsd_duration = statsd_start.elapsed();
    // println!("📡 [RADIATION] StatsD initialization completed in {:?}", statsd_duration);
    
    // Phase 2: Test Setup
    println!("⚙️  [RADIATION] Phase 2: Setting up test parameters...");
    let setup_start = Instant::now();
    
    let mut metrics = WaveCoreMetrics::new()
        .with_problem_type("radiation")
        .with_mesh_tier("T2");
    
    let frequencies = 1200; // Adjusted for ~1 minute runtime
    let dofs = 6;
    let matrix_size = 50;
    metrics.num_panels = frequencies * dofs;
    
    println!("  📋 [RADIATION] Test parameters: {} frequencies, {} DOFs", frequencies, dofs);
    let setup_duration = setup_start.elapsed();
    println!("⚙️  [RADIATION] Test setup completed in {:?}", setup_duration);
    
    // Send test start metric
    let mut tags = HashMap::new();
    tags.insert("test_type".to_string(), "radiation".to_string());
    tags.insert("mesh_tier".to_string(), "T2".to_string());
    
    statsd_client::counter("test.requests", 1.0, Some(tags.clone()));
    println!("  📊 [RADIATION] Sent test start metric");
    
    // Phase 3: Main Test Execution
    println!("🚀 [RADIATION] Phase 3: Starting main test execution...");
    let execution_start = Instant::now();
    
    for freq_idx in 0..frequencies {
        let frequency = 0.2 + (freq_idx as f64 * 2.3) / frequencies as f64;
        
        // Progress logging every 20 frequencies
        if freq_idx % 20 == 0 {
            println!("  📈 [RADIATION] Processing frequency {}/{} ({}%)", freq_idx, frequencies, (freq_idx * 100) / frequencies);
        }
        
        for dof_idx in 0..dofs {
            let start = Instant::now();
        
            // Simulate BEM matrix assembly for radiation
            let matrix_size = 1000; // T2 mesh size
            let mut bem_matrix = vec![vec![0.0; matrix_size]; matrix_size];
            
            // Assembly BEM matrix (simplified)
            for i in 0..matrix_size {
                for j in 0..matrix_size {
                    let distance = ((i as f64 - j as f64).abs() as f64).sqrt();
                    bem_matrix[i][j] = (frequency * distance).exp() / (distance + 1.0);
                }
            }
            
            // Simulate matrix solve (simplified)
            let mut added_mass = 0.0;
            let mut damping = 0.0;
            
            for i in 0..matrix_size {
                for j in 0..matrix_size {
                    added_mass += bem_matrix[i][j] * (frequency * (i + j) as f64).cos();
                    damping += bem_matrix[i][j] * (frequency * (i + j) as f64).sin();
                }
            }
            
            // Prevent compiler optimization (silent check)
            let _large_coeffs = added_mass > 1000000.0 || damping > 1000000.0;
        
            let duration = start.elapsed();
            metrics.add_latency(duration.as_secs_f64() * 1000.0);
            
            // Send frequency/DOF metrics every 50 iterations
            if (freq_idx * dofs + dof_idx) % 50 == 0 {
                println!("  📊 [RADIATION] Freq {} DOF {}: processing={:?}", freq_idx, dof_idx, duration);
                
                // Send processing metrics
                statsd_client::timer("frequency.processing_time", duration.as_millis() as f64, Some(tags.clone()));
                statsd_client::counter("frequency.count", 1.0, Some(tags.clone()));
                // println!("  ✅ [RADIATION] Freq {} DOF {} processed successfully with StatsD", freq_idx, dof_idx);
            }
        }
    }
    
    let execution_duration = execution_start.elapsed();
    println!("🚀 [RADIATION] Main execution completed in {:?}", execution_duration);
    
    // Phase 4: Final Metrics Calculation
    println!("📊 [RADIATION] Phase 4: Calculating final metrics...");
    let metrics_start = Instant::now();
    
    // Calculate final metrics
    let p50 = metrics.calculate_p50();
    let p95 = metrics.calculate_p95();
    let p99 = metrics.calculate_p99();
    let throughput = metrics.calculate_throughput();
    
    println!("  📈 [RADIATION] Calculated metrics: P50={:.2}ms, P95={:.2}ms, P99={:.2}ms, Throughput={:.2} ops/sec", 
             p50, p95, p99, throughput);
    
    // Send final performance metrics
    statsd_client::gauge("test.p50_latency_ms", p50, Some(tags.clone()));
    statsd_client::gauge("test.p95_latency_ms", p95, Some(tags.clone()));
    statsd_client::gauge("test.p99_latency_ms", p99, Some(tags.clone()));
    statsd_client::gauge("test.throughput_ops_per_sec", throughput, Some(tags.clone()));
    statsd_client::gauge("test.total_frequencies", frequencies as f64, Some(tags));
    
    println!("  📊 [RADIATION] Sent final performance metrics to StatsD");
    
    let metrics_duration = metrics_start.elapsed();
    println!("📊 [RADIATION] Final metrics calculation completed in {:?}", metrics_duration);
    
    // Phase 5: Test Completion
    let total_duration = test_start.elapsed();
    println!("✅ [RADIATION] Test completed successfully!");
    println!("📊 [RADIATION] Summary:");
    println!("   - Total frequencies: {}", frequencies);
    println!("   - Total DOFs: {}", dofs);
    println!("   - Average latency (P50): {:.2} ms", p50);
    println!("   - Total test duration: {:?}", total_duration);
    println!("   - StatsD init: {:?}", statsd_duration);
    println!("   - Setup: {:?}", setup_duration);
    println!("   - Execution: {:?}", execution_duration);
    println!("   - Final metrics: {:?}", metrics_duration);
    
    // Save test results
    if let Err(e) = save_test_results("radiation", &metrics, total_duration, 
                                    statsd_duration, setup_duration, execution_duration, metrics_duration) {
        eprintln!("⚠️  [RADIATION] Failed to save test results: {}", e);
    }
    
    Ok(metrics)
}

/// Test 3: Diffraction problems (wave exciting forces) - WITH STATSD
pub fn test_diffraction() -> Result<WaveCoreMetrics, Box<dyn std::error::Error>> {
    println!("🔄 [DIFFRACTION] Starting test...");
    let test_start = Instant::now();
    
    // Phase 1: StatsD Initialization
    println!("📡 [DIFFRACTION] Phase 1: Initializing StatsD client...");
    let statsd_start = Instant::now();
    
    // Initialize StatsD if not already done
    if statsd_client::get_statsd().is_none() || statsd_client::get_statsd().unwrap().lock().unwrap().is_none() {
        let config = StatsDConfig {
            host: "127.0.0.1".to_string(),
            port: 8125,
            prefix: "wavecore".to_string(),
            batch_size: 1,
            flush_interval_ms: 100,
        };
        // println!("  🔧 [DIFFRACTION] Initializing StatsD client...");
        statsd_client::init_statsd(config)?;
        // println!("  ✅ [DIFFRACTION] StatsD client initialized successfully");
    } else {
        // println!("  ✅ [DIFFRACTION] StatsD client already initialized");
    }
    
    let statsd_duration = statsd_start.elapsed();
    // println!("📡 [DIFFRACTION] StatsD initialization completed in {:?}", statsd_duration);
    
    // Phase 2: Test Setup
    println!("⚙️  [DIFFRACTION] Phase 2: Setting up test parameters...");
    let setup_start = Instant::now();
    
    let mut metrics = WaveCoreMetrics::new()
        .with_problem_type("diffraction")
        .with_mesh_tier("T2");
    
    let frequencies = 1600; // Adjusted for ~1 minute runtime
    let directions = 20; // Adjusted for ~1 minute runtime
    metrics.num_panels = frequencies * directions;
    
    println!("  📋 [DIFFRACTION] Test parameters: {} frequencies, {} directions", frequencies, directions);
    let setup_duration = setup_start.elapsed();
    println!("⚙️  [DIFFRACTION] Test setup completed in {:?}", setup_duration);
    
    // Send test start metric
    let mut tags = HashMap::new();
    tags.insert("test_type".to_string(), "diffraction".to_string());
    tags.insert("mesh_tier".to_string(), "T2".to_string());
    
    statsd_client::counter("test.requests", 1.0, Some(tags.clone()));
    println!("  📊 [DIFFRACTION] Sent test start metric");
    
    // Phase 3: Main Test Execution
    println!("🚀 [DIFFRACTION] Phase 3: Starting main test execution...");
    let execution_start = Instant::now();
    
    for freq_idx in 0..frequencies {
        let frequency = 0.2 + (freq_idx as f64 * 2.3) / frequencies as f64;
        
        // Progress logging every 40 frequencies
        if freq_idx % 40 == 0 {
            println!("  📈 [DIFFRACTION] Processing frequency {}/{} ({}%)", freq_idx, frequencies, (freq_idx * 100) / frequencies);
        }
        
        for dir_idx in 0..directions {
            let direction = (dir_idx as f64 * 30.0) * std::f64::consts::PI / 180.0;
            let start = Instant::now();
        
            // Simulate diffraction problem
            let matrix_size = 1000;
            let mut excitation_forces = vec![0.0; 6];
            
            // Calculate wave exciting forces
            for dof in 0..6 {
                let mut force = 0.0;
                for i in 0..matrix_size {
                    let phase = frequency * (i as f64 * 0.1) * direction.cos();
                    force += (phase).sin() * (dof + 1) as f64;
                }
                excitation_forces[dof] = force;
            }
            
            // Calculate Froude-Krylov forces
            let mut fk_forces = vec![0.0; 6];
            for dof in 0..6 {
                fk_forces[dof] = excitation_forces[dof] * 0.7; // Simplified
            }
            
            // Prevent compiler optimization
            let total_force: f64 = excitation_forces.iter().sum();
            if total_force > 1000000.0 {
                println!("  ⚠️  [DIFFRACTION] Large excitation force: {:.3}", total_force);
            }
            
            let duration = start.elapsed();
            metrics.add_latency(duration.as_secs_f64() * 1000.0);
            
            // Send direction metrics every 50 iterations
            if (freq_idx * directions + dir_idx) % 50 == 0 {
                println!("  📊 [DIFFRACTION] Freq {} Dir {}: processing={:?}", freq_idx, dir_idx, duration);
                
                // Send processing metrics
                statsd_client::timer("direction.processing_time", duration.as_millis() as f64, Some(tags.clone()));
                statsd_client::counter("direction.count", 1.0, Some(tags.clone()));
                // println!("  ✅ [DIFFRACTION] Freq {} Dir {} processed successfully with StatsD", freq_idx, dir_idx);
            }
        }
    }
    
    let execution_duration = execution_start.elapsed();
    println!("🚀 [DIFFRACTION] Main execution completed in {:?}", execution_duration);
    
    // Phase 4: Final Metrics Calculation
    println!("📊 [DIFFRACTION] Phase 4: Calculating final metrics...");
    let metrics_start = Instant::now();
    
    // Calculate final metrics
    let p50 = metrics.calculate_p50();
    let p95 = metrics.calculate_p95();
    let p99 = metrics.calculate_p99();
    let throughput = metrics.calculate_throughput();
    
    println!("  📈 [DIFFRACTION] Calculated metrics: P50={:.2}ms, P95={:.2}ms, P99={:.2}ms, Throughput={:.2} ops/sec", 
             p50, p95, p99, throughput);
    
    // Send final performance metrics
    statsd_client::gauge("test.p50_latency_ms", p50, Some(tags.clone()));
    statsd_client::gauge("test.p95_latency_ms", p95, Some(tags.clone()));
    statsd_client::gauge("test.p99_latency_ms", p99, Some(tags.clone()));
    statsd_client::gauge("test.throughput_ops_per_sec", throughput, Some(tags.clone()));
    statsd_client::gauge("test.total_frequencies", frequencies as f64, Some(tags));
    
    println!("  📊 [DIFFRACTION] Sent final performance metrics to StatsD");
    
    let metrics_duration = metrics_start.elapsed();
    println!("📊 [DIFFRACTION] Final metrics calculation completed in {:?}", metrics_duration);
    
    // Phase 5: Test Completion
    let total_duration = test_start.elapsed();
    println!("✅ [DIFFRACTION] Test completed successfully!");
    println!("📊 [DIFFRACTION] Summary:");
    println!("   - Total frequencies: {}", frequencies);
    println!("   - Total directions: {}", directions);
    println!("   - Average latency (P50): {:.2} ms", p50);
    println!("   - Total test duration: {:?}", total_duration);
    println!("   - StatsD init: {:?}", statsd_duration);
    println!("   - Setup: {:?}", setup_duration);
    println!("   - Execution: {:?}", execution_duration);
    println!("   - Final metrics: {:?}", metrics_duration);
    
    // Save test results
    if let Err(e) = save_test_results("diffraction", &metrics, total_duration, 
                                    statsd_duration, setup_duration, execution_duration, metrics_duration) {
        eprintln!("⚠️  [DIFFRACTION] Failed to save test results: {}", e);
    }
    
    Ok(metrics)
}

/// Test 4: RAO calculations - WITH STATSD
pub fn test_rao() -> Result<WaveCoreMetrics, Box<dyn std::error::Error>> {
    println!("🔄 [RAO] Starting test...");
    let test_start = Instant::now();
    
    // Phase 1: StatsD Initialization
    println!("📡 [RAO] Phase 1: Initializing StatsD client...");
    let statsd_start = Instant::now();
    
    // Initialize StatsD if not already done
    if statsd_client::get_statsd().is_none() || statsd_client::get_statsd().unwrap().lock().unwrap().is_none() {
        let config = StatsDConfig {
            host: "127.0.0.1".to_string(),
            port: 8125,
            prefix: "wavecore".to_string(),
            batch_size: 1,
            flush_interval_ms: 100,
        };
        // println!("  🔧 [RAO] Initializing StatsD client...");
        statsd_client::init_statsd(config)?;
        // println!("  ✅ [RAO] StatsD client initialized successfully");
    } else {
        // println!("  ✅ [RAO] StatsD client already initialized");
    }
    
    let statsd_duration = statsd_start.elapsed();
    // println!("📡 [RAO] StatsD initialization completed in {:?}", statsd_duration);
    
    // Phase 2: Test Setup
    println!("⚙️  [RAO] Phase 2: Setting up test parameters...");
    let setup_start = Instant::now();
    
    let mut metrics = WaveCoreMetrics::new()
        .with_problem_type("rao")
        .with_mesh_tier("T2");
    
    let frequencies = 1200; // Adjusted for ~1 minute runtime
    let directions = 16; // Adjusted for ~1 minute runtime
    metrics.num_panels = frequencies * directions;
    
    println!("  📋 [RAO] Test parameters: {} frequencies, {} directions", frequencies, directions);
    let setup_duration = setup_start.elapsed();
    println!("⚙️  [RAO] Test setup completed in {:?}", setup_duration);
    
    // Send test start metric
    let mut tags = HashMap::new();
    tags.insert("test_type".to_string(), "rao".to_string());
    tags.insert("mesh_tier".to_string(), "T2".to_string());
    
    statsd_client::counter("test.requests", 1.0, Some(tags.clone()));
    println!("  📊 [RAO] Sent test start metric");
    
    // Phase 3: Main Test Execution
    println!("🚀 [RAO] Phase 3: Starting main test execution...");
    let execution_start = Instant::now();
    
    for freq_idx in 0..frequencies {
        let frequency = 0.2 + (freq_idx as f64 * 2.3) / frequencies as f64;
        
        // Progress logging every 30 frequencies
        if freq_idx % 30 == 0 {
            println!("  📈 [RAO] Processing frequency {}/{} ({}%)", freq_idx, frequencies, (freq_idx * 100) / frequencies);
        }
        
        for dir_idx in 0..directions {
            let direction = (dir_idx as f64 * 45.0) * std::f64::consts::PI / 180.0;
            let start = Instant::now();
        
            // Simulate RAO calculation
            let matrix_size = 1000;
            let mut rao_values = vec![0.0; 6];
            
            // Calculate RAO for each DOF
            for dof in 0..6 {
                let mut rao = 0.0;
                for i in 0..matrix_size {
                    let phase = frequency * (i as f64 * 0.1) * direction.cos();
                    rao += (phase).cos() * (dof + 1) as f64 * frequency;
                }
                rao_values[dof] = rao;
            }
            
            // Calculate response spectrum
            let mut spectrum = 0.0;
            for dof in 0..6 {
                spectrum += rao_values[dof] * rao_values[dof];
            }
            spectrum = spectrum.sqrt();
            
            // Prevent compiler optimization
            if spectrum > 1000000.0 {
                println!("  ⚠️  [RAO] Large RAO spectrum: {:.3}", spectrum);
            }
            
            let duration = start.elapsed();
            metrics.add_latency(duration.as_secs_f64() * 1000.0);
            
            // Send RAO metrics every 50 iterations
            if (freq_idx * directions + dir_idx) % 50 == 0 {
                println!("  📊 [RAO] Freq {} Dir {}: processing={:?}", freq_idx, dir_idx, duration);
                
                // Send processing metrics
                statsd_client::timer("rao.processing_time", duration.as_millis() as f64, Some(tags.clone()));
                statsd_client::counter("rao.count", 1.0, Some(tags.clone()));
                // println!("  ✅ [RAO] Freq {} Dir {} processed successfully with StatsD", freq_idx, dir_idx);
            }
        }
    }
    
    let execution_duration = execution_start.elapsed();
    println!("🚀 [RAO] Main execution completed in {:?}", execution_duration);
    
    // Phase 4: Final Metrics Calculation
    println!("📊 [RAO] Phase 4: Calculating final metrics...");
    let metrics_start = Instant::now();
    
    // Calculate final metrics
    let p50 = metrics.calculate_p50();
    let p95 = metrics.calculate_p95();
    let p99 = metrics.calculate_p99();
    let throughput = metrics.calculate_throughput();
    
    println!("  📈 [RAO] Calculated metrics: P50={:.2}ms, P95={:.2}ms, P99={:.2}ms, Throughput={:.2} ops/sec", 
             p50, p95, p99, throughput);
    
    // Send final performance metrics
    statsd_client::gauge("test.p50_latency_ms", p50, Some(tags.clone()));
    statsd_client::gauge("test.p95_latency_ms", p95, Some(tags.clone()));
    statsd_client::gauge("test.p99_latency_ms", p99, Some(tags.clone()));
    statsd_client::gauge("test.throughput_ops_per_sec", throughput, Some(tags.clone()));
    statsd_client::gauge("test.total_frequencies", frequencies as f64, Some(tags));
    
    println!("  📊 [RAO] Sent final performance metrics to StatsD");
    
    let metrics_duration = metrics_start.elapsed();
    println!("📊 [RAO] Final metrics calculation completed in {:?}", metrics_duration);
    
    // Phase 5: Test Completion
    let total_duration = test_start.elapsed();
    println!("✅ [RAO] Test completed successfully!");
    println!("📊 [RAO] Summary:");
    println!("   - Total frequencies: {}", frequencies);
    println!("   - Total directions: {}", directions);
    println!("   - Average latency (P50): {:.2} ms", p50);
    println!("   - Total test duration: {:?}", total_duration);
    println!("   - StatsD init: {:?}", statsd_duration);
    println!("   - Setup: {:?}", setup_duration);
    println!("   - Execution: {:?}", execution_duration);
    println!("   - Final metrics: {:?}", metrics_duration);
    
    // Save test results
    if let Err(e) = save_test_results("rao", &metrics, total_duration, 
                                    statsd_duration, setup_duration, execution_duration, metrics_duration) {
        eprintln!("⚠️  [RAO] Failed to save test results: {}", e);
    }
    
    Ok(metrics)
} 