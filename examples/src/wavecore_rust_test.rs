use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Mock BEM solver for testing
struct MockBEMSolver {
    config: BEMConfig,
}

struct BEMConfig {
    num_threads: usize,
    optimization_level: u8,
}

impl MockBEMSolver {
    fn new(config: BEMConfig) -> Self {
        MockBEMSolver { config }
    }
    
    fn solve_radiation(&self, frequency: f64, mode: usize) -> f64 {
        // Simulate BEM solve time based on frequency and mode
        let base_time = 0.001; // 1ms base time (much faster than FFI)
        let frequency_factor = 1.0 + frequency * 0.02;
        let mode_factor = 1.0 + (mode as f64) * 0.01;
        
        // Add some computational work
        let mut result = 0.0;
        for i in 0..1000 {
            result += (frequency * mode_factor * i as f64).sin();
        }
        
        base_time * frequency_factor * mode_factor
    }
    
    fn solve_diffraction(&self, frequency: f64, direction: f64) -> f64 {
        // Simulate diffraction solve
        let base_time = 0.002; // 2ms base time
        let frequency_factor = 1.0 + frequency * 0.03;
        let direction_factor = 1.0 + direction.abs() * 0.01;
        
        // Add computational work
        let mut result = 0.0;
        for i in 0..1500 {
            result += (frequency * direction_factor * i as f64).cos();
        }
        
        base_time * frequency_factor * direction_factor
    }
    
    fn solve_mesh(&self, resolution: usize) -> f64 {
        // Simulate mesh solve time
        let base_time = 0.0005; // 0.5ms base time
        let resolution_factor = 1.0 + (resolution as f64 - 16.0) / 100.0;
        
        // Add computational work
        let mut result = 0.0;
        for i in 0..resolution * 10 {
            result += (i as f64 * resolution_factor).sqrt();
        }
        
        base_time * resolution_factor
    }
}

// Performance measurement utilities
struct PerformanceMetrics {
    latencies: Vec<f64>,
    throughput: f64,
    memory_usage: u64,
}

impl PerformanceMetrics {
    fn new() -> Self {
        PerformanceMetrics {
            latencies: Vec::new(),
            throughput: 0.0,
            memory_usage: 0,
        }
    }
    
    fn add_latency(&mut self, latency_ms: f64) {
        self.latencies.push(latency_ms);
    }
    
    fn calculate_p50(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted[sorted.len() / 2]
    }
    
    fn calculate_p95(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = (sorted.len() as f64 * 0.95) as usize;
        sorted[index.min(sorted.len() - 1)]
    }
    
    fn calculate_p99(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = (sorted.len() as f64 * 0.99) as usize;
        sorted[index.min(sorted.len() - 1)]
    }
    
    fn calculate_throughput(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let total_time: f64 = self.latencies.iter().sum();
        if total_time > 0.0 {
            (self.latencies.len() as f64 * 1000.0) / total_time
        } else {
            0.0
        }
    }
}

// Test functions
fn test_latency_vs_frequency() -> PerformanceMetrics {
    println!("Running latency vs frequency test...");
    
    let config = BEMConfig {
        num_threads: 1,
        optimization_level: 3,
    };
    let solver = MockBEMSolver::new(config);
    let mut metrics = PerformanceMetrics::new();
    
    let frequencies = vec![0.1, 0.2, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0];
    
    for freq in frequencies {
        for _ in 0..10 { // 10 iterations per frequency
            let start = Instant::now();
            let _result = solver.solve_radiation(freq, 0);
            let duration = start.elapsed();
            
            metrics.add_latency(duration.as_secs_f64() * 1000.0);
        }
    }
    
    metrics.throughput = metrics.calculate_throughput();
    metrics
}

fn test_speedup_vs_cores() -> HashMap<usize, PerformanceMetrics> {
    println!("Running speedup vs cores test...");
    
    let mut results = HashMap::new();
    let cores = vec![1, 2, 4, 8, 16];
    
    // Use rayon for better parallel performance
    use rayon::prelude::*;
    
    for &num_cores in &cores {
        let mut metrics = PerformanceMetrics::new();
        
        // Configure rayon thread pool for this test
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cores)
            .build()
            .unwrap();
        
        // Simulate CPU-intensive parallel work
        let work_per_core = 500; // Increased work for better measurement
        let total_work = work_per_core * num_cores;
        
        let start = Instant::now();
        
        if num_cores == 1 {
            // Single-threaded - CPU-intensive work
            for i in 0..total_work {
                // Simulate BEM computation with more realistic workload
                let mut result = 0.0;
                for j in 0..1000 {
                    result += (i as f64 * j as f64).sqrt().sin();
                }
                // Prevent compiler optimization
                if result > 1000000.0 {
                    println!("Impossible result: {}", result);
                }
            }
        } else {
            // Multi-threaded with rayon for better performance
            pool.install(|| {
                (0..total_work).into_par_iter().for_each(|i| {
                    // Simulate BEM computation with more realistic workload
                    let mut result = 0.0;
                    for j in 0..1000 {
                        result += (i as f64 * j as f64).sqrt().sin();
                    }
                    // Prevent compiler optimization
                    if result > 1000000.0 {
                        println!("Impossible result: {}", result);
                    }
                });
            });
        }
        
        let duration = start.elapsed();
        metrics.add_latency(duration.as_secs_f64() * 1000.0);
        metrics.throughput = metrics.calculate_throughput();
        
        results.insert(num_cores, metrics);
    }
    
    results
}

fn test_memory_soak() -> PerformanceMetrics {
    println!("Running memory soak test...");
    
    let config = BEMConfig {
        num_threads: 1,
        optimization_level: 3,
    };
    let solver = MockBEMSolver::new(config);
    let mut metrics = PerformanceMetrics::new();
    
    // Simulate 6-hour soak test (shortened for demo)
    let duration = Duration::from_secs(60); // 1 minute demo
    let interval = Duration::from_secs(10);
    let start_time = Instant::now();
    
    while start_time.elapsed() < duration {
        let iteration_start = Instant::now();
        
        // Simulate memory-intensive work
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push(i as f64);
        }
        
        // Solve multiple problems
        for _ in 0..10 {
            let _result = solver.solve_diffraction(1.0, 0.0);
        }
        
        let iteration_duration = iteration_start.elapsed();
        metrics.add_latency(iteration_duration.as_secs_f64() * 1000.0);
        
        // Simulate memory usage tracking
        metrics.memory_usage = data.len() as u64;
        
        thread::sleep(interval);
    }
    
    metrics.throughput = metrics.calculate_throughput();
    metrics
}

fn test_latency_vs_elements() -> PerformanceMetrics {
    println!("Running latency vs elements test...");
    
    let config = BEMConfig {
        num_threads: 1,
        optimization_level: 3,
    };
    let solver = MockBEMSolver::new(config);
    let mut metrics = PerformanceMetrics::new();
    
    let resolutions = vec![16, 32, 64, 128];
    
    for res in resolutions {
        for _ in 0..20 { // 20 iterations per resolution
            let start = Instant::now();
            let _result = solver.solve_mesh(res);
            let duration = start.elapsed();
            
            metrics.add_latency(duration.as_secs_f64() * 1000.0);
        }
    }
    
    metrics.throughput = metrics.calculate_throughput();
    metrics
}

// Main test runner
fn main() {
    println!("🦀 WaveCore Rust Native Test Suite");
    println!("===================================");
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <test_name>", args[0]);
        println!("Available tests:");
        println!("  test_latency_vs_frequency");
        println!("  test_speedup_vs_cores");
        println!("  test_memory_soak");
        println!("  test_latency_vs_elements");
        return;
    }
    
    let test_name = &args[1];
    
    match test_name.as_str() {
        "test_latency_vs_frequency" => {
            let metrics = test_latency_vs_frequency();
            println!("Latency vs Frequency Results:");
            println!("  P50: {:.3} ms", metrics.calculate_p50());
            println!("  P95: {:.3} ms", metrics.calculate_p95());
            println!("  P99: {:.3} ms", metrics.calculate_p99());
            println!("  Throughput: {:.0} ops/sec", metrics.throughput);
        }
        
        "test_speedup_vs_cores" => {
            let results = test_speedup_vs_cores();
            println!("Speedup vs Cores Results:");
            for (cores, metrics) in results {
                println!("  {} cores: {:.3} ms, {:.0} ops/sec", 
                    cores, metrics.calculate_p50(), metrics.throughput);
            }
        }
        
        "test_memory_soak" => {
            let metrics = test_memory_soak();
            println!("Memory Soak Results:");
            println!("  P50: {:.3} ms", metrics.calculate_p50());
            println!("  P95: {:.3} ms", metrics.calculate_p95());
            println!("  Throughput: {:.0} ops/sec", metrics.throughput);
            println!("  Memory Usage: {} KB", metrics.memory_usage);
        }
        
        "test_latency_vs_elements" => {
            let metrics = test_latency_vs_elements();
            println!("Latency vs Elements Results:");
            println!("  P50: {:.3} ms", metrics.calculate_p50());
            println!("  P95: {:.3} ms", metrics.calculate_p95());
            println!("  P99: {:.3} ms", metrics.calculate_p99());
            println!("  Throughput: {:.0} ops/sec", metrics.throughput);
        }
        
        _ => {
            println!("Unknown test: {}", test_name);
        }
    }
} 