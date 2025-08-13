use crate::config::{Config, LoadPatternConfig, LoadPhase, SLOConfig, StopConditions};
use crate::types::WaveCoreTestResult;
use crate::metrics::WaveCoreMetrics;
use crate::metrics_collector;
use chrono::{DateTime, Local};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::System;
use tracing::{debug, error, info, warn};

/// Resource monitoring data
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    pub timestamp: DateTime<Local>,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
}

/// Performance metrics during stress testing
#[derive(Debug, Clone)]
pub struct StressTestMetrics {
    pub timestamp: DateTime<Local>,
    pub load_rate: f64,  // Current load rate (multiplier of baseline)
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_ops_per_sec: f64,
    pub error_rate_percent: f64,
    pub resource_metrics: ResourceMetrics,
}

/// Stress test result
#[derive(Debug, Clone)]
pub struct StressTestResult {
    pub pattern_name: String,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub duration_seconds: u64,
    pub max_sustainable_rate: Option<f64>,
    pub slo_violations: Vec<SLOViolation>,
    pub stop_condition_triggered: Option<String>,
    pub metrics: Vec<StressTestMetrics>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SLOViolation {
    pub timestamp: DateTime<Local>,
    pub metric: String,
    pub expected: f64,
    pub actual: f64,
}

/// Simplified test function trait
pub trait TestFunction: Send + Sync {
    fn run(&self, load_multiplier: f64) -> Result<WaveCoreMetrics, Box<dyn std::error::Error>>;
}

/// Linear ramp load pattern executor
pub struct LinearRampExecutor {
    pattern_config: LoadPatternConfig,
}

impl LinearRampExecutor {
    pub fn new(pattern_config: LoadPatternConfig) -> Self {
        Self { pattern_config }
    }
    
    pub fn execute(
        &self,
        config: &Config,
        test_fn: Arc<dyn TestFunction>,
    ) -> Result<StressTestResult, Box<dyn std::error::Error>> {
        let start_time = Local::now();
        let stop_conditions = config.get_stop_conditions();
        let metrics = Arc::new(Mutex::new(Vec::new()));
        
        // Create progress bar
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap()
        );
        
        let mut slo_violations = Vec::new();
        let mut stop_condition_triggered = None;
        let mut success = true;
        
        // Execute each phase
        for phase in &self.pattern_config.phases {
            let phase_success = self.execute_phase(
                phase,
                config,
                &test_fn,
                &progress_bar,
                &metrics,
                &stop_conditions,
            )?;
            
            if !phase_success {
                stop_condition_triggered = Some(format!("Phase: {}", phase.name));
                success = false;
                break;
            }
        }
        
        progress_bar.finish_with_message("Stress test completed");
        
        let end_time = Local::now();
        let duration = end_time.signed_duration_since(start_time).num_seconds() as u64;
        
        // Calculate MSR (Max Sustainable Rate)
        let metrics_clone = metrics.lock().unwrap().clone();
        let max_sustainable_rate = self.calculate_msr(&metrics_clone, config);
        
        Ok(StressTestResult {
            pattern_name: "linear_ramp".to_string(),
            start_time,
            end_time,
            duration_seconds: duration,
            max_sustainable_rate,
            slo_violations,
            stop_condition_triggered,
            metrics: metrics_clone,
            success,
            error_message: None,
        })
    }
    
    fn execute_phase(
        &self,
        phase: &LoadPhase,
        config: &Config,
        test_fn: &Arc<dyn TestFunction>,
        progress_bar: &ProgressBar,
        metrics: &Arc<Mutex<Vec<StressTestMetrics>>>,
        stop_conditions: &StopConditions,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let phase_name = &phase.name;
        info!("Starting phase: {}", phase_name);
        
        let duration = self.get_phase_duration(phase);
        let start_time = Instant::now();
        
        // Calculate load rates for this phase
        let load_rates = self.calculate_load_rates(phase, config);
        
        for (i, load_rate) in load_rates.iter().enumerate() {
            if start_time.elapsed() >= duration {
                break;
            }
            
            // Check stop conditions
            if self.check_stop_conditions(metrics, stop_conditions) {
                warn!("Stop condition triggered during phase {}", phase_name);
                return Ok(false);
            }
            
            // Execute test with current load rate
            let test_result = test_fn.run(*load_rate);
            
            // Record metrics
            if let Ok(test_metrics) = test_result {
                let resource_metrics = self.get_resource_metrics();
                let stress_metrics = StressTestMetrics {
                    timestamp: Local::now(),
                    load_rate: *load_rate,
                    p50_latency_ms: test_metrics.calculate_p50(),
                    p95_latency_ms: test_metrics.calculate_p95(),
                    p99_latency_ms: test_metrics.calculate_p99(),
                    throughput_ops_per_sec: test_metrics.throughput,
                    error_rate_percent: 0.0, // TODO: Calculate from test results
                    resource_metrics,
                };
                
                // Update Prometheus metrics
                metrics_collector::update_stress_metrics_sync(*load_rate, 1, None);
                metrics_collector::update_performance_metrics_sync(
                    test_metrics.calculate_p50(),
                    test_metrics.calculate_p95(),
                    test_metrics.calculate_p99(),
                    test_metrics.throughput
                );
                
                metrics.lock().unwrap().push(stress_metrics);
            } else {
                // Record error
                metrics_collector::record_test_metrics_sync("stress_test", Duration::from_secs(0), false);
            }
            
            // Update progress
            progress_bar.set_message(format!("Phase: {} | Load: {:.2}x | Step: {}/{}", 
                phase_name, load_rate, i + 1, load_rates.len()));
            
            // Wait for step duration
            if let Some(step_duration) = phase.step_duration_seconds {
                std::thread::sleep(Duration::from_secs(step_duration as u64));
            }
        }
        
        Ok(true)
    }
    
    fn get_phase_duration(&self, phase: &LoadPhase) -> Duration {
        if let Some(minutes) = phase.duration_minutes {
            Duration::from_secs(minutes as u64 * 60)
        } else if let Some(seconds) = phase.duration_seconds {
            Duration::from_secs(seconds as u64)
        } else if let Some(hours) = phase.duration_hours {
            Duration::from_secs(hours as u64 * 3600)
        } else {
            Duration::from_secs(60) // Default 1 minute
        }
    }
    
    fn calculate_load_rates(&self, phase: &LoadPhase, config: &Config) -> Vec<f64> {
        let slo = config.get_slo();
        let baseline_rate = slo.throughput_sps;
        
        if let (Some(start_rate), Some(end_rate), Some(step_increase)) = 
            (phase.start_rate, phase.end_rate, phase.step_increase) {
            // Linear ramp with steps
            let mut rates = Vec::new();
            let mut current_rate = start_rate;
            
            while current_rate <= end_rate {
                rates.push(current_rate);
                current_rate += step_increase;
            }
            
            rates
        } else if let Some(rate) = phase.rate {
            // Constant rate
            vec![rate]
        } else {
            // Default to baseline rate
            vec![1.0]
        }
    }
    
    fn check_stop_conditions(
        &self,
        metrics: &Arc<Mutex<Vec<StressTestMetrics>>>,
        stop_conditions: &StopConditions,
    ) -> bool {
        let metrics_guard = metrics.lock().unwrap();
        
        if metrics_guard.is_empty() {
            return false;
        }
        
        // Get latest metrics
        let latest = &metrics_guard[metrics_guard.len() - 1];
        
        // Check latency stop condition
        if latest.p99_latency_ms > stop_conditions.p99_latency_ms {
            return true;
        }
        
        // Check error rate stop condition
        if latest.error_rate_percent > stop_conditions.error_rate_percent {
            return true;
        }
        
        // Check resource usage stop conditions
        if latest.resource_metrics.cpu_usage_percent > stop_conditions.cpu_usage_percent {
            return true;
        }
        
        if latest.resource_metrics.memory_usage_percent > stop_conditions.memory_usage_percent {
            return true;
        }
        
        false
    }
    
    fn get_resource_metrics(&self) -> ResourceMetrics {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_used = sys.used_memory();
        let memory_total = sys.total_memory();
        let memory_usage_percent = (memory_used as f64 / memory_total as f64) * 100.0;
        
        ResourceMetrics {
            timestamp: Local::now(),
            cpu_usage_percent: cpu_usage as f64,
            memory_usage_percent,
            memory_used_mb: memory_used / 1024 / 1024,
            memory_total_mb: memory_total / 1024 / 1024,
        }
    }
    
    fn calculate_msr(&self, metrics: &[StressTestMetrics], config: &Config) -> Option<f64> {
        if metrics.is_empty() {
            return None;
        }
        
        let slo = config.get_slo();
        let mut max_sustainable_rate = 0.0;
        
        // Find the highest load rate where SLOs are met
        for metric in metrics {
            let slo_violated = 
                metric.p99_latency_ms > slo.p99_latency_ms ||
                metric.error_rate_percent > slo.error_rate_percent;
            
            if !slo_violated && metric.load_rate > max_sustainable_rate {
                max_sustainable_rate = metric.load_rate;
            }
        }
        
        if max_sustainable_rate > 0.0 {
            Some(max_sustainable_rate)
        } else {
            None
        }
    }
}

/// Step load pattern executor
pub struct StepLoadExecutor {
    pattern_config: LoadPatternConfig,
}

impl StepLoadExecutor {
    pub fn new(pattern_config: LoadPatternConfig) -> Self {
        Self { pattern_config }
    }
    
    pub fn execute(
        &self,
        config: &Config,
        test_fn: Arc<dyn TestFunction>,
    ) -> Result<StressTestResult, Box<dyn std::error::Error>> {
        let start_time = Local::now();
        let metrics = Arc::new(Mutex::new(Vec::new()));
        
        // Execute each step
        for phase in &self.pattern_config.phases {
            if let Some(rate) = phase.rate {
                let test_result = test_fn.run(rate);
                
                if let Ok(test_metrics) = test_result {
                    let resource_metrics = self.get_resource_metrics();
                    let stress_metrics = StressTestMetrics {
                        timestamp: Local::now(),
                        load_rate: rate,
                        p50_latency_ms: test_metrics.calculate_p50(),
                        p95_latency_ms: test_metrics.calculate_p95(),
                        p99_latency_ms: test_metrics.calculate_p99(),
                        throughput_ops_per_sec: test_metrics.throughput,
                        error_rate_percent: 0.0,
                        resource_metrics,
                    };
                    
                    metrics.lock().unwrap().push(stress_metrics);
                }
                
                // Wait for step duration
                if let Some(duration) = phase.duration_seconds {
                    std::thread::sleep(Duration::from_secs(duration as u64));
                }
            }
        }
        
        let end_time = Local::now();
        let duration = end_time.signed_duration_since(start_time).num_seconds() as u64;
        let metrics_clone = metrics.lock().unwrap().clone();
        
        Ok(StressTestResult {
            pattern_name: "step_load".to_string(),
            start_time,
            end_time,
            duration_seconds: duration,
            max_sustainable_rate: None, // TODO: Calculate MSR
            slo_violations: Vec::new(),
            stop_condition_triggered: None,
            metrics: metrics_clone,
            success: true,
            error_message: None,
        })
    }
    
    fn get_resource_metrics(&self) -> ResourceMetrics {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_used = sys.used_memory();
        let memory_total = sys.total_memory();
        let memory_usage_percent = (memory_used as f64 / memory_total as f64) * 100.0;
        
        ResourceMetrics {
            timestamp: Local::now(),
            cpu_usage_percent: cpu_usage as f64,
            memory_usage_percent,
            memory_used_mb: memory_used / 1024 / 1024,
            memory_total_mb: memory_total / 1024 / 1024,
        }
    }
}

/// MSR Search using AIMD (Additive Increase, Multiplicative Decrease)
pub struct MSRSearch {
    config: Config,
}

impl MSRSearch {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub fn find_msr(&self, test_fn: Arc<dyn TestFunction>) -> Result<f64, Box<dyn std::error::Error>> {
        let aimd_config = &self.config.msr_search.aimd;
        let criteria = &self.config.msr_search.criteria;
        let slo = self.config.get_slo();
        
        let mut current_rate = 0.5; // Start at 50% of baseline
        let mut max_sustainable_rate = 0.0;
        let mut consecutive_violations = 0;
        let mut consecutive_successes = 0;
        
        info!("Starting MSR search with AIMD algorithm");
        
        for iteration in 0..20 { // Limit iterations
            // Execute test with current rate
            let test_result = test_fn.run(current_rate);
            
            if let Ok(metrics) = test_result {
                let p99_latency = metrics.calculate_p99();
                let slo_violated = p99_latency > criteria.p99_latency_ms;
                
                // Update Prometheus metrics
                metrics_collector::update_stress_metrics_sync(current_rate, 1, None);
                metrics_collector::update_performance_metrics_sync(
                    metrics.calculate_p50(),
                    metrics.calculate_p95(),
                    metrics.calculate_p99(),
                    metrics.throughput
                );
                
                if slo_violated {
                    // Multiplicative decrease
                    current_rate *= aimd_config.multiplicative_decrease;
                    consecutive_violations += 1;
                    consecutive_successes = 0;
                    
                    info!("SLO violated at rate {:.2}x, decreasing to {:.2}x", 
                        current_rate / aimd_config.multiplicative_decrease, current_rate);
                } else {
                    // Additive increase
                    current_rate += aimd_config.additive_increase;
                    consecutive_successes += 1;
                    consecutive_violations = 0;
                    
                    if current_rate > max_sustainable_rate {
                        max_sustainable_rate = current_rate;
                        // Update MSR metric
                        metrics_collector::update_stress_metrics_sync(current_rate, 1, Some(max_sustainable_rate));
                    }
                    
                    info!("SLO met at rate {:.2}x, increasing to {:.2}x", 
                        current_rate - aimd_config.additive_increase, current_rate);
                }
                
                // Check convergence
                if consecutive_successes >= 3 && consecutive_violations == 0 {
                    if (current_rate - max_sustainable_rate).abs() < aimd_config.convergence_threshold {
                        info!("MSR search converged at {:.2}x", max_sustainable_rate);
                        return Ok(max_sustainable_rate);
                    }
                }
                
                // Check if we've gone too low
                if current_rate < 0.1 {
                    info!("Rate too low, stopping MSR search");
                    return Ok(max_sustainable_rate);
                }
            } else {
                // Test failed, decrease rate
                current_rate *= aimd_config.multiplicative_decrease;
                consecutive_violations += 1;
                consecutive_successes = 0;
            }
            
            // Wait for interval
            std::thread::sleep(Duration::from_secs(aimd_config.interval_seconds as u64));
        }
        
        info!("MSR search completed after 20 iterations, best rate: {:.2}x", max_sustainable_rate);
        Ok(max_sustainable_rate)
    }
}

/// Stress test manager for orchestrating all stress testing
pub struct StressTestManager {
    config: Config,
}

impl StressTestManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub fn run_stress_tests(&self, test_fn: Arc<dyn TestFunction>) -> Result<Vec<StressTestResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        if !self.config.is_stress_testing_enabled() {
            info!("Stress testing is disabled in configuration");
            return Ok(results);
        }
        
        let enabled_patterns = self.config.get_enabled_load_patterns();
        info!("Running stress tests with patterns: {:?}", enabled_patterns);
        
        for pattern_name in enabled_patterns {
            if let Some(pattern_config) = self.config.get_load_pattern(pattern_name) {
                if pattern_config.enabled {
                    info!("Running stress test pattern: {}", pattern_name);
                    
                    let executor = self.create_executor(pattern_name, pattern_config.clone());
                    let result = executor.execute(&self.config, test_fn.clone())?;
                    
                    results.push(result);
                    
                    // Check if we should stop due to previous failure
                    if let Some(last_result) = results.last() {
                        if !last_result.success {
                            warn!("Stress test pattern {} failed, stopping", pattern_name);
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    fn create_executor(&self, pattern_name: &str, pattern_config: LoadPatternConfig) -> Box<dyn StressTestExecutor> {
        match pattern_name {
            "linear_ramp" => Box::new(LinearRampExecutor::new(pattern_config)),
            "step_load" => Box::new(StepLoadExecutor::new(pattern_config)),
            _ => {
                // Default to linear ramp
                warn!("Unknown pattern {}, using linear_ramp", pattern_name);
                Box::new(LinearRampExecutor::new(pattern_config))
            }
        }
    }
    
    pub fn run_msr_search(&self, test_fn: Arc<dyn TestFunction>) -> Result<f64, Box<dyn std::error::Error>> {
        if !self.config.is_msr_search_enabled() {
            return Err("MSR search is disabled in configuration".into());
        }
        
        let msr_search = MSRSearch::new(self.config.clone());
        msr_search.find_msr(test_fn)
    }
}

/// Trait for stress test executors
pub trait StressTestExecutor: Send + Sync {
    fn execute(&self, config: &Config, test_fn: Arc<dyn TestFunction>) -> Result<StressTestResult, Box<dyn std::error::Error>>;
}

impl StressTestExecutor for LinearRampExecutor {
    fn execute(&self, config: &Config, test_fn: Arc<dyn TestFunction>) -> Result<StressTestResult, Box<dyn std::error::Error>> {
        self.execute(config, test_fn)
    }
}

impl StressTestExecutor for StepLoadExecutor {
    fn execute(&self, config: &Config, test_fn: Arc<dyn TestFunction>) -> Result<StressTestResult, Box<dyn std::error::Error>> {
        self.execute(config, test_fn)
    }
} 