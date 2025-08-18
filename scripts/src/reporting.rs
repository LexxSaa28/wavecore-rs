use std::fs;
use std::path::Path;
use chrono::Local;
use crate::types::WaveCoreTestResult;

/// Generate comprehensive test report
pub fn generate_report(results: &[WaveCoreTestResult]) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    
    // Create results directory
    let results_dir = Path::new("test_results");
    fs::create_dir_all(results_dir)?;
    
    // Save JSON results
    let json_path = results_dir.join(format!("wavecore_test_results_{}.json", timestamp));
    let json_content = serde_json::to_string_pretty(&results)?;
    fs::write(&json_path, json_content)?;
    
    // Generate Markdown report
    let md_path = results_dir.join(format!("WAVECORE_TEST_REPORT_{}.md", timestamp));
    let md_content = generate_markdown_report(results, &timestamp);
    fs::write(&md_path, md_content)?;
    
    println!("📊 Test report saved:");
    println!("  JSON: {}", json_path.display());
    println!("  Markdown: {}", md_path.display());
    
    Ok(())
}

fn generate_markdown_report(results: &[WaveCoreTestResult], timestamp: &str) -> String {
    let mut report = String::new();
    
    // Calculate summary statistics
    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.success).count();
    let failed_tests = total_tests - passed_tests;
    
    let avg_p50 = results.iter().map(|r| r.p50_latency_ms).sum::<f64>() / total_tests as f64;
    let avg_p95 = results.iter().map(|r| r.p95_latency_ms).sum::<f64>() / total_tests as f64;
    let avg_throughput = results.iter().map(|r| r.throughput_ops_per_sec).sum::<f64>() / total_tests as f64;
    
    report.push_str(&format!("# 🦀 WaveCore Library Test Report\n\n"));
    report.push_str(&format!("**Test Date:** {}\n", timestamp));
    report.push_str(&format!("**Total Tests:** {}\n", total_tests));
    report.push_str(&format!("**Passed:** {}\n", passed_tests));
    report.push_str(&format!("**Failed:** {}\n", failed_tests));
    report.push_str(&format!("**Success Rate:** {:.1}%\n\n", (passed_tests as f64 / total_tests as f64) * 100.0));
    
    report.push_str("## 📊 Performance Summary\n\n");
    report.push_str(&format!("- **Average P50 Latency:** {:.3} ms\n", avg_p50));
    report.push_str(&format!("- **Average P95 Latency:** {:.3} ms\n", avg_p95));
    report.push_str(&format!("- **Average Throughput:** {:.0} ops/sec\n\n", avg_throughput));
    
    report.push_str("## 🔍 Detailed Results\n\n");
    report.push_str("| Test | Status | P50 (ms) | P95 (ms) | Throughput | Memory | Matrix |\n");
    report.push_str("|------|--------|----------|----------|------------|--------|--------|\n");
    
    for result in results {
        let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
        report.push_str(&format!("| {} | {} | {:.3} | {:.3} | {:.0} | {} MB | {} |\n",
            result.test_name, status, result.p50_latency_ms, result.p95_latency_ms,
            result.throughput_ops_per_sec, result.memory_usage_mb, result.matrix_size));
    }
    
    report.push_str("\n## 🚨 Issues\n\n");
    
    let failed_results: Vec<_> = results.iter().filter(|r| !r.success).collect();
    if failed_results.is_empty() {
        report.push_str("✅ No issues found - all tests passed!\n");
    } else {
        for result in &failed_results {
            report.push_str(&format!("- **{}**: {}\n", 
                result.test_name, 
                result.error_message.as_ref().unwrap_or(&"Unknown error".to_string())));
        }
    }
    
    report.push_str("\n## 🎯 Production Readiness\n\n");
    
    let production_ready = failed_results.is_empty() && 
                          avg_p50 < 1000.0 && // P50 < 1 second
                          avg_p95 < 2000.0;   // P95 < 2 seconds
    
    if production_ready {
        report.push_str("✅ **PRODUCTION READY** - All criteria met!\n");
    } else {
        report.push_str("❌ **NOT PRODUCTION READY** - Issues found\n");
    }
    
    report
} 