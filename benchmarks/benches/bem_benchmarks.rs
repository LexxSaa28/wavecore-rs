//! BEM benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wavecore_bem::*;

fn bem_solver_benchmark(c: &mut Criterion) {
    c.bench_function("bem_solver_creation", |b| {
        b.iter(|| {
            let solver = BEMSolver::new(SolverEngine::Standard);
            black_box(solver);
        });
    });
}

criterion_group!(benches, bem_solver_benchmark);
criterion_main!(benches); 