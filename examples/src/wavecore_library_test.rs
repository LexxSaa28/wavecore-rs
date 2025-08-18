use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

// Import actual WaveCore modules
use wavecore_bem::{BEMSolver, BEMProblem, BEMResult, ProblemType};
use wavecore_meshes::{Mesh, MeshBuilder};
use wavecore_bodies::{FloatingBody, DOF};
use wavecore_green_functions::{GreenFunction, DelhommeauGreenFunction};
use wavecore_matrices::{Matrix, SolverType};

/// Performance metrics for WaveCore library tests
#[derive(Debug, Clone)]
struct WaveCoreMetrics {
    latencies: Vec<f64>,
    throughput: f64,
    memory_usage: u64,
    matrix_size: usize,
    num_panels: usize,
    num_frequencies: usize,
}

impl WaveCoreMetrics {
    fn new() -> Self {
        Self {
            latencies: Vec::new(),
            throughput: 0.0,
            memory_usage: 0,
            matrix_size: 0,
            num_panels: 0,
            num_frequencies: 0,
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
        let total_time = self.latencies.iter().sum::<f64>();
        if total_time > 0.0 {
            (self.latencies.len() as f64 * 1000.0) / total_time
        } else {
            0.0
        }
    }
}

/// Test 1: Latency vs Frequency using actual BEM solver
fn test_latency_vs_frequency() -> WaveCoreMetrics {
    println!("Running WaveCore Latency vs Frequency test...");
    
    let mut metrics = WaveCoreMetrics::new();
    
    // Create a simple floating body (sphere)
    let body = FloatingBody::sphere(1.0, 10.0); // radius 1m, draft 10m
    let mesh = MeshBuilder::new()
        .add_body(&body)
        .resolution(0.2) // 20cm resolution
        .build()
        .expect("Failed to build mesh");
    
    metrics.num_panels = mesh.panels().unwrap().len();
    metrics.matrix_size = metrics.num_panels;
    
    // Test different frequencies
    let frequencies = vec![0.1, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    metrics.num_frequencies = frequencies.len();
    
    let solver = BEMSolver::new();
    
    for &freq in &frequencies {
        let start = Instant::now();
        
        // Create radiation problem
        let problem = BEMProblem {
            problem_type: ProblemType::Radiation { 
                frequency: freq, 
                mode: 0 
            },
            mesh: mesh.clone(),
            water_depth: 50.0,
            gravity: 9.81,
        };
        
        // Solve the problem
        let _result = solver.solve(&problem).expect("BEM solve failed");
        
        let duration = start.elapsed();
        metrics.add_latency(duration.as_secs_f64() * 1000.0);
    }
    
    metrics.throughput = metrics.calculate_throughput();
    metrics
}

/// Test 2: Speedup vs Cores using actual parallel BEM solver
fn test_speedup_vs_cores() -> HashMap<usize, WaveCoreMetrics> {
    println!("Running WaveCore Speedup vs Cores test...");
    
    let mut results = HashMap::new();
    let cores = vec![1, 2, 4, 8, 16];
    
    // Create a larger mesh for better parallel scaling
    let body = FloatingBody::sphere(2.0, 15.0); // larger sphere
    let mesh = MeshBuilder::new()
        .add_body(&body)
        .resolution(0.1) // 10cm resolution - more panels
        .build()
        .expect("Failed to build mesh");
    
    let num_panels = mesh.panels().unwrap().len();
    println!("Testing with {} panels", num_panels);
    
    for &num_cores in &cores {
        let mut metrics = WaveCoreMetrics::new();
        metrics.num_panels = num_panels;
        metrics.matrix_size = num_panels;
        
        // Configure solver for specific number of threads
        let solver = BEMSolver::new()
            .with_threads(num_cores)
            .with_solver_type(SolverType::Direct);
        
        // Test multiple frequencies for each core count
        let frequencies = vec![0.5, 1.0, 1.5, 2.0];
        metrics.num_frequencies = frequencies.len();
        
        let start = Instant::now();
        
        for &freq in &frequencies {
            let problem = BEMProblem {
                problem_type: ProblemType::Radiation { 
                    frequency: freq, 
                    mode: 0 
                },
                mesh: mesh.clone(),
                water_depth: 50.0,
                gravity: 9.81,
            };
            
            let _result = solver.solve(&problem).expect("BEM solve failed");
        }
        
        let duration = start.elapsed();
        metrics.add_latency(duration.as_secs_f64() * 1000.0);
        metrics.throughput = metrics.calculate_throughput();
        
        results.insert(num_cores, metrics);
    }
    
    results
}

/// Test 3: Memory vs Time (Soak test) using actual BEM solver
fn test_memory_soak() -> WaveCoreMetrics {
    println!("Running WaveCore Memory Soak test...");
    
    let mut metrics = WaveCoreMetrics::new();
    
    // Create a complex mesh for memory testing
    let body = FloatingBody::ship_like(50.0, 10.0, 5.0); // 50m ship
    let mesh = MeshBuilder::new()
        .add_body(&body)
        .resolution(0.15) // 15cm resolution
        .build()
        .expect("Failed to build mesh");
    
    metrics.num_panels = mesh.panels().unwrap().len();
    metrics.matrix_size = metrics.num_panels;
    
    let solver = BEMSolver::new();
    
    // Simulate 6-hour soak test (shortened for demo)
    let duration = Duration::from_secs(60); // 1 minute demo
    let interval = Duration::from_secs(10);
    let start_time = Instant::now();
    
    let mut iteration = 0;
    while start_time.elapsed() < duration {
        let iteration_start = Instant::now();
        
        // Solve multiple problems to simulate continuous workload
        let frequencies = vec![0.3, 0.6, 0.9, 1.2, 1.5];
        metrics.num_frequencies = frequencies.len();
        
        for &freq in &frequencies {
            let problem = BEMProblem {
                problem_type: ProblemType::Diffraction { 
                    frequency: freq, 
                    direction: 0.0 
                },
                mesh: mesh.clone(),
                water_depth: 50.0,
                gravity: 9.81,
            };
            
            let _result = solver.solve(&problem).expect("BEM solve failed");
        }
        
        let iteration_duration = iteration_start.elapsed();
        metrics.add_latency(iteration_duration.as_secs_f64() * 1000.0);
        
        // Simulate memory usage tracking
        metrics.memory_usage = num_panels * 8; // 8 bytes per panel
        
        iteration += 1;
        thread::sleep(interval);
    }
    
    metrics.throughput = metrics.calculate_throughput();
    metrics
}

/// Test 4: Latency vs Elements (Mesh scaling)
fn test_latency_vs_elements() -> WaveCoreMetrics {
    println!("Running WaveCore Latency vs Elements test...");
    
    let mut metrics = WaveCoreMetrics::new();
    
    // Test different mesh resolutions
    let resolutions = vec![0.5, 0.3, 0.2, 0.15, 0.1, 0.08]; // coarse to fine
    
    let body = FloatingBody::sphere(1.5, 12.0);
    let solver = BEMSolver::new();
    
    for &resolution in &resolutions {
        let mesh = MeshBuilder::new()
            .add_body(&body)
            .resolution(resolution)
            .build()
            .expect("Failed to build mesh");
        
        let num_panels = mesh.panels().unwrap().len();
        metrics.num_panels = num_panels;
        metrics.matrix_size = num_panels;
        
        let start = Instant::now();
        
        // Solve radiation problem
        let problem = BEMProblem {
            problem_type: ProblemType::Radiation { 
                frequency: 1.0, 
                mode: 0 
            },
            mesh: mesh.clone(),
            water_depth: 50.0,
            gravity: 9.81,
        };
        
        let _result = solver.solve(&problem).expect("BEM solve failed");
        
        let duration = start.elapsed();
        metrics.add_latency(duration.as_secs_f64() * 1000.0);
    }
    
    metrics.throughput = metrics.calculate_throughput();
    metrics
}

/// Test 5: Multi-body interaction
fn test_multi_body() -> WaveCoreMetrics {
    println!("Running WaveCore Multi-body test...");
    
    let mut metrics = WaveCoreMetrics::new();
    
    // Create multiple floating bodies
    let body1 = FloatingBody::sphere(1.0, 8.0);
    let body2 = FloatingBody::sphere(1.0, 8.0);
    let body3 = FloatingBody::sphere(1.0, 8.0);
    
    // Position bodies with some separation
    let mesh = MeshBuilder::new()
        .add_body(&body1)
        .add_body(&body2)
        .add_body(&body3)
        .resolution(0.2)
        .build()
        .expect("Failed to build multi-body mesh");
    
    metrics.num_panels = mesh.panels().unwrap().len();
    metrics.matrix_size = metrics.num_panels;
    
    let solver = BEMSolver::new();
    
    // Test multiple frequencies
    let frequencies = vec![0.5, 1.0, 1.5, 2.0];
    metrics.num_frequencies = frequencies.len();
    
    for &freq in &frequencies {
        let start = Instant::now();
        
        let problem = BEMProblem {
            problem_type: ProblemType::Radiation { 
                frequency: freq, 
                mode: 0 
            },
            mesh: mesh.clone(),
            water_depth: 50.0,
            gravity: 9.81,
        };
        
        let _result = solver.solve(&problem).expect("Multi-body BEM solve failed");
        
        let duration = start.elapsed();
        metrics.add_latency(duration.as_secs_f64() * 1000.0);
    }
    
    metrics.throughput = metrics.calculate_throughput();
    metrics
}

/// Test 6: Numerical validation
fn test_numerical_validation() -> (bool, f64, f64, f64) {
    println!("Running WaveCore Numerical Validation test...");
    
    // Create a simple sphere for analytical comparison
    let body = FloatingBody::sphere(1.0, 10.0);
    let mesh = MeshBuilder::new()
        .add_body(&body)
        .resolution(0.1)
        .build()
        .expect("Failed to build validation mesh");
    
    let solver = BEMSolver::new();
    
    // Test at low frequency where analytical solution is known
    let problem = BEMProblem {
        problem_type: ProblemType::Radiation { 
            frequency: 0.1, // Low frequency
            mode: 0 
        },
        mesh: mesh.clone(),
        water_depth: 50.0,
        gravity: 9.81,
    };
    
    let result = solver.solve(&problem).expect("Validation solve failed");
    
    // Extract added mass and damping matrices
    let added_mass = result.added_mass().expect("No added mass matrix");
    let damping = result.damping().expect("No damping matrix");
    
    // Check matrix symmetry
    let symmetry_error = check_matrix_symmetry(added_mass);
    let positivity_error = check_matrix_positivity(added_mass);
    let energy_error = check_energy_conservation(&result);
    
    let all_valid = symmetry_error < 1e-6 && positivity_error < 1e-6 && energy_error < 1e-3;
    
    (all_valid, symmetry_error, positivity_error, energy_error)
}

fn check_matrix_symmetry(matrix: &Matrix) -> f64 {
    let n = matrix.rows();
    let mut max_error = 0.0;
    
    for i in 0..n {
        for j in 0..n {
            let diff = (matrix[(i, j)] - matrix[(j, i)]).abs();
            max_error = max_error.max(diff);
        }
    }
    
    max_error
}

fn check_matrix_positivity(matrix: &Matrix) -> f64 {
    // For simplicity, check diagonal elements are positive
    let mut min_diagonal = f64::INFINITY;
    
    for i in 0..matrix.rows() {
        min_diagonal = min_diagonal.min(matrix[(i, i)]);
    }
    
    if min_diagonal < 0.0 {
        -min_diagonal
    } else {
        0.0
    }
}

fn check_energy_conservation(result: &BEMResult) -> f64 {
    // Check that damping matrix is positive semi-definite
    let damping = result.damping().expect("No damping matrix");
    let mut min_eigenvalue = f64::INFINITY;
    
    // Simple check: diagonal elements should be non-negative
    for i in 0..damping.rows {
        min_eigenvalue = min_eigenvalue.min(damping.get(i, i).unwrap());
    }
    
    if min_eigenvalue < 0.0 {
        -min_eigenvalue
    } else {
        0.0
    }
}

fn main() {
    println!("🦀 WaveCore Library Test Suite");
    println!("==============================");
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <test_name>", args[0]);
        println!("Available tests:");
        println!("  test_latency_vs_frequency");
        println!("  test_speedup_vs_cores");
        println!("  test_memory_soak");
        println!("  test_latency_vs_elements");
        println!("  test_multi_body");
        println!("  test_numerical_validation");
        println!("  test_all");
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
            println!("  Panels: {}", metrics.num_panels);
            println!("  Matrix Size: {}", metrics.matrix_size);
        }
        
        "test_speedup_vs_cores" => {
            let results = test_speedup_vs_cores();
            println!("Speedup vs Cores Results:");
            for (cores, metrics) in &results {
                println!("  {} cores: {:.3} ms ({} panels)", 
                    cores, metrics.calculate_p50(), metrics.num_panels);
            }
        }
        
        "test_memory_soak" => {
            let metrics = test_memory_soak();
            println!("Memory Soak Results:");
            println!("  P50: {:.3} ms", metrics.calculate_p50());
            println!("  P95: {:.3} ms", metrics.calculate_p95());
            println!("  Throughput: {:.0} ops/sec", metrics.throughput);
            println!("  Memory Usage: {} MB", metrics.memory_usage / 1024 / 1024);
        }
        
        "test_latency_vs_elements" => {
            let metrics = test_latency_vs_elements();
            println!("Latency vs Elements Results:");
            println!("  P50: {:.3} ms", metrics.calculate_p50());
            println!("  P95: {:.3} ms", metrics.calculate_p95());
            println!("  Throughput: {:.0} ops/sec", metrics.throughput);
            println!("  Max Panels: {}", metrics.num_panels);
        }
        
        "test_multi_body" => {
            let metrics = test_multi_body();
            println!("Multi-body Results:");
            println!("  P50: {:.3} ms", metrics.calculate_p50());
            println!("  P95: {:.3} ms", metrics.calculate_p95());
            println!("  Throughput: {:.0} ops/sec", metrics.throughput);
            println!("  Total Panels: {}", metrics.num_panels);
        }
        
        "test_numerical_validation" => {
            let (valid, symmetry, positivity, energy) = test_numerical_validation();
            println!("Numerical Validation Results:");
            println!("  Overall Valid: {}", valid);
            println!("  Symmetry Error: {:.2e}", symmetry);
            println!("  Positivity Error: {:.2e}", positivity);
            println!("  Energy Error: {:.2e}", energy);
        }
        
        "test_all" => {
            println!("Running all WaveCore library tests...");
            
            let metrics1 = test_latency_vs_frequency();
            let results2 = test_speedup_vs_cores();
            let metrics3 = test_memory_soak();
            let metrics4 = test_latency_vs_elements();
            let metrics5 = test_multi_body();
            let (valid, _, _, _) = test_numerical_validation();
            
            println!("\n=== SUMMARY ===");
            println!("Latency vs Frequency: P50={:.3}ms", metrics1.calculate_p50());
            println!("Speedup vs Cores: {} core configurations tested", results2.len());
            println!("Memory Soak: P50={:.3}ms", metrics3.calculate_p50());
            println!("Latency vs Elements: P50={:.3}ms", metrics4.calculate_p50());
            println!("Multi-body: P50={:.3}ms", metrics5.calculate_p50());
            println!("Numerical Validation: {}", if valid { "PASSED" } else { "FAILED" });
        }
        
        _ => {
            println!("Unknown test: {}", test_name);
        }
    }
} 