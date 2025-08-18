use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/// Metrics collector for WaveCore test suite
pub struct MetricsCollector {
    registry: Registry,
    
    // Test metrics
    test_requests_total: IntCounter,
    test_errors_total: IntCounter,
    test_duration_seconds: Histogram,
    test_success_rate: Gauge,
    
    // Stress test metrics
    stress_test_load_multiplier: Gauge,
    stress_test_current_phase: Gauge,
    stress_test_msr: Gauge,
    
    // System metrics
    system_cpu_usage_percent: Gauge,
    system_memory_usage_percent: Gauge,
    system_memory_used_mb: Gauge,
    system_memory_total_mb: Gauge,
    
    // Performance metrics
    p50_latency_ms: Gauge,
    p95_latency_ms: Gauge,
    p99_latency_ms: Gauge,
    throughput_ops_per_sec: Gauge,
}

impl MetricsCollector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        
        // Test metrics
        let test_requests_total = IntCounter::new(
            "wavecore_test_requests_total",
            "Total number of test requests"
        )?;
        
        let test_errors_total = IntCounter::new(
            "wavecore_test_errors_total",
            "Total number of test errors"
        )?;
        
        let test_duration_seconds = Histogram::with_opts(
            HistogramOpts::new(
                "wavecore_test_duration_seconds",
                "Test duration in seconds"
            )
            .buckets(vec![0.001, 0.01, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0])
        )?;
        
        let test_success_rate = Gauge::new(
            "wavecore_test_success_rate",
            "Test success rate percentage"
        )?;
        
        // Stress test metrics
        let stress_test_load_multiplier = Gauge::new(
            "wavecore_stress_test_load_multiplier",
            "Current load multiplier in stress test"
        )?;
        
        let stress_test_current_phase = Gauge::new(
            "wavecore_stress_test_current_phase",
            "Current phase in stress test (0=ramp_up, 1=hold, 2=spike, 3=recovery)"
        )?;
        
        let stress_test_msr = Gauge::new(
            "wavecore_stress_test_msr",
            "Maximum Sustainable Rate discovered"
        )?;
        
        // System metrics
        let system_cpu_usage_percent = Gauge::new(
            "wavecore_system_cpu_usage_percent",
            "System CPU usage percentage"
        )?;
        
        let system_memory_usage_percent = Gauge::new(
            "wavecore_system_memory_usage_percent",
            "System memory usage percentage"
        )?;
        
        let system_memory_used_mb = Gauge::new(
            "wavecore_system_memory_used_mb",
            "System memory used in MB"
        )?;
        
        let system_memory_total_mb = Gauge::new(
            "wavecore_system_memory_total_mb",
            "Total system memory in MB"
        )?;
        
        // Performance metrics
        let p50_latency_ms = Gauge::new(
            "wavecore_p50_latency_ms",
            "P50 latency in milliseconds"
        )?;
        
        let p95_latency_ms = Gauge::new(
            "wavecore_p95_latency_ms",
            "P95 latency in milliseconds"
        )?;
        
        let p99_latency_ms = Gauge::new(
            "wavecore_p99_latency_ms",
            "P99 latency in milliseconds"
        )?;
        
        let throughput_ops_per_sec = Gauge::new(
            "wavecore_throughput_ops_per_sec",
            "Throughput in operations per second"
        )?;
        
        // Register metrics
        registry.register(Box::new(test_requests_total.clone()))?;
        registry.register(Box::new(test_errors_total.clone()))?;
        registry.register(Box::new(test_duration_seconds.clone()))?;
        registry.register(Box::new(test_success_rate.clone()))?;
        registry.register(Box::new(stress_test_load_multiplier.clone()))?;
        registry.register(Box::new(stress_test_current_phase.clone()))?;
        registry.register(Box::new(stress_test_msr.clone()))?;
        registry.register(Box::new(system_cpu_usage_percent.clone()))?;
        registry.register(Box::new(system_memory_usage_percent.clone()))?;
        registry.register(Box::new(system_memory_used_mb.clone()))?;
        registry.register(Box::new(system_memory_total_mb.clone()))?;
        registry.register(Box::new(p50_latency_ms.clone()))?;
        registry.register(Box::new(p95_latency_ms.clone()))?;
        registry.register(Box::new(p99_latency_ms.clone()))?;
        registry.register(Box::new(throughput_ops_per_sec.clone()))?;
        
        Ok(Self {
            registry,
            test_requests_total,
            test_errors_total,
            test_duration_seconds,
            test_success_rate,
            stress_test_load_multiplier,
            stress_test_current_phase,
            stress_test_msr,
            system_cpu_usage_percent,
            system_memory_usage_percent,
            system_memory_used_mb,
            system_memory_total_mb,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            throughput_ops_per_sec,
        })
    }
    
    /// Record a test request
    pub fn record_test_request(&self, test_name: &str) {
        self.test_requests_total.inc();
    }
    
    /// Record a test error
    pub fn record_test_error(&self, test_name: &str) {
        self.test_errors_total.inc();
    }
    
    /// Record test duration
    pub fn record_test_duration(&self, duration: Duration) {
        self.test_duration_seconds.observe(duration.as_secs_f64());
    }
    
    /// Update test success rate
    pub fn update_success_rate(&self, success_rate: f64) {
        self.test_success_rate.set(success_rate);
    }
    
    /// Update stress test load multiplier
    pub fn update_stress_load_multiplier(&self, load_multiplier: f64) {
        self.stress_test_load_multiplier.set(load_multiplier);
    }
    
    /// Update stress test phase
    pub fn update_stress_phase(&self, phase: u32) {
        self.stress_test_current_phase.set(phase as f64);
    }
    
    /// Update MSR
    pub fn update_msr(&self, msr: f64) {
        self.stress_test_msr.set(msr);
    }
    
    /// Update system metrics
    pub fn update_system_metrics(&self, cpu_usage: f64, memory_usage: f64, memory_used: f64, memory_total: f64) {
        self.system_cpu_usage_percent.set(cpu_usage);
        self.system_memory_usage_percent.set(memory_usage);
        self.system_memory_used_mb.set(memory_used);
        self.system_memory_total_mb.set(memory_total);
    }
    
    /// Update performance metrics
    pub fn update_performance_metrics(&self, p50: f64, p95: f64, p99: f64, throughput: f64) {
        self.p50_latency_ms.set(p50);
        self.p95_latency_ms.set(p95);
        self.p99_latency_ms.set(p99);
        self.throughput_ops_per_sec.set(throughput);
    }
    
    /// Get metrics as Prometheus format
    pub fn get_metrics(&self) -> Result<String, Box<dyn std::error::Error>> {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&self.registry.gather(), &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
    
    /// Get registry reference
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

/// Global metrics collector instance
lazy_static::lazy_static! {
    static ref METRICS_COLLECTOR: Arc<RwLock<MetricsCollector>> = {
        Arc::new(RwLock::new(MetricsCollector::new().expect("Failed to create metrics collector")))
    };
}

/// Get global metrics collector
pub async fn get_metrics_collector() -> Arc<RwLock<MetricsCollector>> {
    METRICS_COLLECTOR.clone()
}

/// Synchronous access to metrics collector
pub fn get_metrics_collector_sync() -> Arc<RwLock<MetricsCollector>> {
    METRICS_COLLECTOR.clone()
}

/// Record test metrics
pub async fn record_test_metrics(test_name: &str, duration: Duration, success: bool) {
    let collector = get_metrics_collector().await;
    let collector = collector.read().await;
    
    collector.record_test_request(test_name);
    collector.record_test_duration(duration);
    
    if !success {
        collector.record_test_error(test_name);
    }
}

/// Update stress test metrics
pub async fn update_stress_metrics(load_multiplier: f64, phase: u32, msr: Option<f64>) {
    let collector = get_metrics_collector().await;
    let collector = collector.read().await;
    
    collector.update_stress_load_multiplier(load_multiplier);
    collector.update_stress_phase(phase);
    
    if let Some(msr_value) = msr {
        collector.update_msr(msr_value);
    }
}

/// Update system metrics
pub async fn update_system_metrics(cpu_usage: f64, memory_usage: f64, memory_used: f64, memory_total: f64) {
    let collector = get_metrics_collector().await;
    let collector = collector.read().await;
    
    collector.update_system_metrics(cpu_usage, memory_usage, memory_used, memory_total);
}

/// Update performance metrics
pub async fn update_performance_metrics(p50: f64, p95: f64, p99: f64, throughput: f64) {
    let collector = get_metrics_collector().await;
    let collector = collector.read().await;
    
    collector.update_performance_metrics(p50, p95, p99, throughput);
}

/// Get metrics as Prometheus format
pub async fn get_metrics_prometheus() -> Result<String, Box<dyn std::error::Error>> {
    let collector = get_metrics_collector().await;
    let collector = collector.read().await;
    collector.get_metrics()
}

/// Global static metrics collector for synchronous access
pub static mut GLOBAL_COLLECTOR: Option<MetricsCollector> = None;

/// Initialize global collector
pub fn init_global_collector() {
    unsafe {
        GLOBAL_COLLECTOR = Some(MetricsCollector::new().expect("Failed to create metrics collector"));
    }
}

/// Synchronous wrapper functions for metrics collection
pub fn record_test_metrics_sync(test_name: &str, duration: Duration, success: bool) {
    unsafe {
        if let Some(collector) = &mut GLOBAL_COLLECTOR {
            collector.record_test_request(test_name);
            collector.record_test_duration(duration);
            if !success {
                collector.record_test_error(test_name);
            }
        }
    }
}

pub fn update_performance_metrics_sync(p50: f64, p95: f64, p99: f64, throughput: f64) {
    unsafe {
        if let Some(collector) = &mut GLOBAL_COLLECTOR {
            collector.update_performance_metrics(p50, p95, p99, throughput);
        }
    }
}

pub fn update_stress_metrics_sync(load_multiplier: f64, phase: u32, msr: Option<f64>) {
    unsafe {
        if let Some(collector) = &mut GLOBAL_COLLECTOR {
            collector.update_stress_load_multiplier(load_multiplier);
            collector.update_stress_phase(phase);
            if let Some(msr_value) = msr {
                collector.update_msr(msr_value);
            }
        }
    }
} 