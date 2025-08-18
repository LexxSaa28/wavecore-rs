//! Green functions benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wavecore_green_functions::*;

fn green_function_benchmark(c: &mut Criterion) {
    c.bench_function("green_function_creation", |b| {
        b.iter(|| {
            let params = GreenFunctionParams::default();
            let green_fn = GreenFunction::new(params).unwrap();
            black_box(green_fn);
        });
    });
    
    // Benchmark actual Green function evaluation
    c.bench_function("delhommeau_evaluation", |b| {
        let params = GreenFunctionParams {
            method: Method::Delhommeau,
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        let green_fn = GreenFunction::new(params).unwrap();
        
        b.iter(|| {
            let value = green_fn.evaluate(black_box(1.0), black_box(-0.5)).unwrap();
            black_box(value);
        });
    });
    
    // Benchmark gradient evaluation
    c.bench_function("delhommeau_gradient", |b| {
        let params = GreenFunctionParams {
            method: Method::Delhommeau,
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        let green_fn = GreenFunction::new(params).unwrap();
        
        b.iter(|| {
            let gradient = green_fn.gradient(black_box(1.0), black_box(-0.5)).unwrap();
            black_box(gradient);
        });
    });
    
    // Benchmark finite depth evaluation
    c.bench_function("delhommeau_finite_depth", |b| {
        let params = GreenFunctionParams {
            method: Method::Delhommeau,
            frequency: 1.0,
            depth: 10.0,  // finite depth
            ..Default::default()
        };
        let green_fn = GreenFunction::new(params).unwrap();
        
        b.iter(|| {
            let value = green_fn.evaluate(black_box(1.0), black_box(-0.5)).unwrap();
            black_box(value);
        });
    });
    
    // Benchmark HAMS method evaluation
    c.bench_function("hams_evaluation", |b| {
        let params = GreenFunctionParams {
            method: Method::HAMS,
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        let green_fn = GreenFunction::new(params).unwrap();
        
        b.iter(|| {
            let value = green_fn.evaluate(black_box(1.0), black_box(-0.5)).unwrap();
            black_box(value);
        });
    });
    
    // Benchmark HAMS finite depth (series expansion)
    c.bench_function("hams_finite_depth", |b| {
        let params = GreenFunctionParams {
            method: Method::HAMS,
            frequency: 1.0,
            depth: 10.0,  // finite depth with series expansion
            tolerance: 1e-6,
            max_points: 100,
            ..Default::default()
        };
        let green_fn = GreenFunction::new(params).unwrap();
        
        b.iter(|| {
            let value = green_fn.evaluate(black_box(1.0), black_box(-0.5)).unwrap();
            black_box(value);
        });
    });
    
    // Benchmark HAMS gradient calculation
    c.bench_function("hams_gradient", |b| {
        let params = GreenFunctionParams {
            method: Method::HAMS,
            frequency: 1.0,
            depth: 10.0,
            ..Default::default()
        };
        let green_fn = GreenFunction::new(params).unwrap();
        
        b.iter(|| {
            let gradient = green_fn.gradient(black_box(1.0), black_box(-0.5)).unwrap();
            black_box(gradient);
        });
    });
    
    // Benchmark LiangWuNoblesse method evaluation
    c.bench_function("liangwunoblesse_evaluation", |b| {
        let params = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        let green_fn = GreenFunction::new(params).unwrap();
        
        b.iter(|| {
            let value = green_fn.evaluate(black_box(1.0), black_box(-0.5)).unwrap();
            black_box(value);
        });
    });
    
    // Benchmark LiangWuNoblesse finite depth (complex geometry)
    c.bench_function("liangwunoblesse_finite_depth", |b| {
        let params = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            frequency: 1.0,
            depth: 8.0,
            ..Default::default()
        };
        let green_fn = GreenFunction::new(params).unwrap();
        
        b.iter(|| {
            let value = green_fn.evaluate(black_box(1.5), black_box(-0.8)).unwrap();
            black_box(value);
        });
    });
    
    // Benchmark LiangWuNoblesse gradient (enhanced)
    c.bench_function("liangwunoblesse_gradient", |b| {
        let params = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            frequency: 1.0,
            depth: 12.0,
            ..Default::default()
        };
        let green_fn = GreenFunction::new(params).unwrap();
        
        b.iter(|| {
            let gradient = green_fn.gradient(black_box(1.2), black_box(-0.6)).unwrap();
            black_box(gradient);
        });
    });
}

criterion_group!(benches, green_function_benchmark);
criterion_main!(benches); 