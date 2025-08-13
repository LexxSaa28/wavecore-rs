//! # WaveCore Examples
//! 
//! Example applications for marine hydrodynamics analysis.
//! 
//! This module provides comprehensive examples demonstrating the use of WaveCore
//! for various marine hydrodynamics applications.
//! 
//! ## Examples
//! 
//! - **Sphere Example**: Basic sphere BEM analysis
//! - **Cylinder Example**: Cylinder wave diffraction
//! - **Ship Example**: Ship hull analysis
//! - **Benchmark Example**: Performance benchmarking
//! 
//! ## Usage
//! 
//! ```bash
//! # Run sphere example
//! cargo run --example sphere
//! 
//! # Run cylinder example
//! cargo run --example cylinder
//! 
//! # Run ship example
//! cargo run --example ship
//! 
//! # Run benchmark example
//! cargo run --example benchmark
//! ```

use anyhow::Result;
use log::{info, error};

mod sphere_example;
mod cylinder_example;
mod ship_example;
mod benchmark_example;

use sphere_example::SphereExample;
use cylinder_example::CylinderExample;
use ship_example::ShipExample;
use benchmark_example::BenchmarkExample;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    info!("🌊 WaveCore Examples - Marine Hydrodynamics Analysis");
    info!("==================================================");
    
    let args: Vec<String> = std::env::args().collect();
    let example = args.get(1).map(|s| s.as_str()).unwrap_or("sphere");
    
    match example {
        "sphere" => {
            info!("Running Sphere Example...");
            let sphere_example = SphereExample::new();
            sphere_example.run().await?;
        }
        "cylinder" => {
            info!("Running Cylinder Example...");
            let cylinder_example = CylinderExample::new();
            cylinder_example.run().await?;
        }
        "ship" => {
            info!("Running Ship Example...");
            let ship_example = ShipExample::new();
            ship_example.run().await?;
        }
        "benchmark" => {
            info!("Running Benchmark Example...");
            let benchmark_example = BenchmarkExample::new();
            benchmark_example.run().await?;
        }
        _ => {
            error!("Unknown example: {}", example);
            println!("Available examples:");
            println!("  sphere     - Basic sphere BEM analysis");
            println!("  cylinder   - Cylinder wave diffraction");
            println!("  ship       - Ship hull analysis");
            println!("  benchmark  - Performance benchmarking");
            std::process::exit(1);
        }
    }
    
    info!("Example completed successfully!");
    Ok(())
} 