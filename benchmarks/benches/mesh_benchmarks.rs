//! Mesh benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wavecore_meshes::*;

fn mesh_creation_benchmark(c: &mut Criterion) {
    c.bench_function("sphere_mesh_creation", |b| {
        b.iter(|| {
            let sphere = PredefinedGeometry::sphere(1.0, 32, 16).unwrap();
            black_box(sphere);
        });
    });
}

criterion_group!(benches, mesh_creation_benchmark);
criterion_main!(benches); 