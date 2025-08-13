//! Linear solvers benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use wavecore_matrices::*;

fn create_test_matrix(size: usize) -> (Matrix, Vec<f64>) {
    // Create a well-conditioned positive definite matrix for testing
    let mut matrix_data = vec![0.0; size * size];
    let mut rhs = vec![0.0; size];
    
    // Create diagonal dominant matrix for stability
    for i in 0..size {
        for j in 0..size {
            if i == j {
                matrix_data[i * size + j] = 10.0; // Diagonal dominance
            } else {
                matrix_data[i * size + j] = 1.0 / ((i + j + 2) as f64);
            }
        }
        rhs[i] = (i + 1) as f64; // Simple RHS
    }
    
    let matrix = Matrix::from_vec(size, size, matrix_data).unwrap();
    (matrix, rhs)
}

fn create_symmetric_matrix(size: usize) -> (Matrix, Vec<f64>) {
    // Create symmetric positive definite matrix for Cholesky
    let mut matrix_data = vec![0.0; size * size];
    let mut rhs = vec![0.0; size];
    
    for i in 0..size {
        for j in 0..size {
            let val = if i == j {
                10.0 + i as f64 // Diagonal dominance
            } else {
                1.0 / ((i + j + 2) as f64).sqrt()
            };
            matrix_data[i * size + j] = val;
            matrix_data[j * size + i] = val; // Symmetry
        }
        rhs[i] = (i + 1) as f64;
    }
    
    let matrix = Matrix::from_vec(size, size, matrix_data).unwrap();
    (matrix, rhs)
}

fn linear_solvers_benchmark(c: &mut Criterion) {
    let sizes = [10, 50, 100, 200];
    
    // Benchmark LU solver
    let mut group = c.benchmark_group("LU_Solver");
    for size in sizes.iter() {
        let (matrix, rhs) = create_test_matrix(*size);
        group.bench_with_input(BenchmarkId::new("LU", size), size, |b, _| {
            b.iter(|| {
                let result = lu_solve(black_box(&matrix), black_box(&rhs));
                black_box(result)
            });
        });
    }
    group.finish();
    
    // Benchmark Cholesky solver
    let mut group = c.benchmark_group("Cholesky_Solver");
    for size in sizes.iter() {
        let (matrix, rhs) = create_symmetric_matrix(*size);
        group.bench_with_input(BenchmarkId::new("Cholesky", size), size, |b, _| {
            b.iter(|| {
                let result = cholesky_solve(black_box(&matrix), black_box(&rhs));
                black_box(result)
            });
        });
    }
    group.finish();
    
    // Benchmark GMRES solver
    let mut group = c.benchmark_group("GMRES_Solver");
    for size in sizes.iter() {
        let (matrix, rhs) = create_test_matrix(*size);
        group.bench_with_input(BenchmarkId::new("GMRES", size), size, |b, _| {
            b.iter(|| {
                let result = gmres_solve_with_params(
                    black_box(&matrix), 
                    black_box(&rhs), 
                    1e-8, 
                    200, 
                    Some(20)
                );
                black_box(result)
            });
        });
    }
    group.finish();
    
    // Benchmark CG solver
    let mut group = c.benchmark_group("CG_Solver");
    for size in sizes.iter() {
        let (matrix, rhs) = create_symmetric_matrix(*size);
        group.bench_with_input(BenchmarkId::new("CG", size), size, |b, _| {
            b.iter(|| {
                let result = cg_solve_with_params(
                    black_box(&matrix), 
                    black_box(&rhs), 
                    1e-8, 
                    500
                );
                black_box(result)
            });
        });
    }
    group.finish();
    
    // Benchmark BiCGSTAB solver
    let mut group = c.benchmark_group("BiCGSTAB_Solver");
    for size in sizes.iter() {
        let (matrix, rhs) = create_test_matrix(*size);
        group.bench_with_input(BenchmarkId::new("BiCGSTAB", size), size, |b, _| {
            b.iter(|| {
                let result = bicgstab_solve_with_params(
                    black_box(&matrix), 
                    black_box(&rhs), 
                    1e-8, 
                    500
                );
                black_box(result)
            });
        });
    }
    group.finish();
    
    // Benchmark LinearSolver interface
    c.bench_function("linear_solver_interface_lu", |b| {
        let (matrix, rhs) = create_test_matrix(50);
        let solver = LinearSolver::new(SolverType::LU);
        b.iter(|| {
            let result = solver.solve(black_box(&matrix), black_box(&rhs));
            black_box(result)
        });
    });
    
    c.bench_function("linear_solver_interface_gmres", |b| {
        let (matrix, rhs) = create_test_matrix(50);
        let solver = LinearSolver::new(SolverType::GMRES);
        b.iter(|| {
            let result = solver.solve(black_box(&matrix), black_box(&rhs));
            black_box(result)
        });
    });
}

criterion_group!(benches, linear_solvers_benchmark);
criterion_main!(benches); 