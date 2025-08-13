//! Matrix benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wavecore_matrices::*;

fn matrix_creation_benchmark(c: &mut Criterion) {
    c.bench_function("matrix_creation", |b| {
        b.iter(|| {
            let matrix = Matrix::new(100, 100);
            black_box(matrix);
        });
    });
}

criterion_group!(benches, matrix_creation_benchmark);
criterion_main!(benches); 