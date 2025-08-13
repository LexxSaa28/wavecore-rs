# 🌊 WaveCore API Reference

**Complete API documentation for WaveCore - Production-ready marine hydrodynamics solver**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![API](https://img.shields.io/badge/API-complete-success.svg)](WAVECORE_API_REFERENCE.md)

> **Part of the OceanOS Platform** - A comprehensive marine simulation ecosystem for next-generation maritime operations.

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Core Modules](#core-modules)
   - [BEM Module](#bem-module)
   - [Green Functions Module](#green-functions-module)
   - [Meshes Module](#meshes-module)
   - [Matrices Module](#matrices-module)
   - [Bodies Module](#bodies-module)
   - [I/O Module](#io-module)
   - [Validation Module](#validation-module)
3. [Supporting Modules](#supporting-modules)
   - [Resistance Module](#resistance-module)
   - [Post-Processing Module](#post-processing-module)
   - [UI Module](#ui-module)
   - [GPU Module](#gpu-module)
4. [Error Handling](#error-handling)
5. [Common Types](#common-types)
6. [Examples](#examples)
7. [Performance Guidelines](#performance-guidelines)

---

## Overview

WaveCore is a high-performance marine hydrodynamics solver implementing the Boundary Element Method (BEM) with industry-grade performance and GPU acceleration. This API reference provides comprehensive documentation for all public interfaces.

### Key Design Principles

- **Type Safety**: Strong typing with comprehensive error handling
- **Performance**: Optimized for high-performance computing
- **Modularity**: Clean separation of concerns across modules
- **Extensibility**: Trait-based interfaces for easy extension
- **Industry Standards**: Compliance with marine engineering standards

---

## Core Modules

### BEM Module

**Module**: `wavecore_bem`

The Boundary Element Method solver engine for marine hydrodynamics.

#### Main Types

```rust
pub struct BEMSolver {
    config: BEMConfig,
}

pub struct BEMConfig {
    pub engine: SolverEngine,
    pub tolerance: f64,
    pub max_iterations: usize,
    pub parallel: bool,
    pub memory_limit: Option<usize>,
}

pub enum SolverEngine {
    Standard,
    FastMultipole,
    HierarchicalMatrix,
    Adaptive,
}

pub enum ProblemType {
    Radiation {
        frequency: f64,
        mode: usize,
    },
    Diffraction {
        frequency: f64,
        direction: f64,
    },
    Combined {
        frequency: f64,
        direction: f64,
        modes: Vec<usize>,
    },
}
```

#### Main Functions

```rust
impl BEMSolver {
    /// Create a new BEM solver with specified engine
    pub fn new(engine: SolverEngine) -> Self
    
    /// Create solver with custom configuration
    pub fn with_config(config: BEMConfig) -> Self
    
    /// Get current configuration
    pub fn config(&self) -> &BEMConfig
    
    /// Update solver configuration
    pub fn update_config(&mut self, config: BEMConfig)
    
    /// Solve BEM problem
    pub fn solve(&self, problem: &ProblemType, mesh: &Mesh) -> Result<BEMResults>
}
```

#### Usage Example

```rust
use wavecore_bem::{BEMSolver, SolverEngine, ProblemType, BEMConfig};

// Create solver with standard engine
let solver = BEMSolver::new(SolverEngine::Standard);

// Create radiation problem
let problem = ProblemType::Radiation {
    frequency: 1.0,
    mode: 2, // Heave mode
};

// Solve problem
let results = solver.solve(&problem, &mesh)?;
```

### Green Functions Module

**Module**: `wavecore_green_functions`

Green function implementations for marine hydrodynamics.

#### Main Types

```rust
pub struct GreenFunction {
    params: GreenFunctionParams,
    implementation: Box<dyn GreenFunctionTrait>,
}

pub struct GreenFunctionParams {
    pub method: Method,
    pub frequency: f64,
    pub depth: f64,
    pub gravity: f64,
    pub density: f64,
    pub tolerance: f64,
    pub max_points: usize,
}

pub enum Method {
    Delhommeau,
    HAMS,
    LiangWuNoblesse,
    FinGreen3D,
}

pub trait GreenFunctionTrait: Send + Sync {
    fn evaluate(&self, r: f64, z: f64) -> Result<Complex64>;
    fn gradient(&self, r: f64, z: f64) -> Result<(Complex64, Complex64)>;
    fn method(&self) -> Method;
    fn params(&self) -> &GreenFunctionParams;
}
```

#### Main Functions

```rust
impl GreenFunction {
    /// Create new Green function with parameters
    pub fn new(params: GreenFunctionParams) -> Result<Self>
    
    /// Evaluate Green function at point (r, z)
    pub fn evaluate(&self, r: f64, z: f64) -> Result<Complex64>
    
    /// Evaluate Green function gradient
    pub fn gradient(&self, r: f64, z: f64) -> Result<(Complex64, Complex64)>
    
    /// Get method type
    pub fn method(&self) -> Method
    
    /// Get parameters
    pub fn params(&self) -> &GreenFunctionParams
}
```

#### Usage Example

```rust
use wavecore_green_functions::{GreenFunction, GreenFunctionParams, Method};

// Create parameters for Delhommeau method
let params = GreenFunctionParams {
    method: Method::Delhommeau,
    frequency: 1.0,
    depth: f64::INFINITY,
    gravity: 9.81,
    density: 1025.0,
    tolerance: 1e-6,
    max_points: 1000,
};

// Create Green function
let green_fn = GreenFunction::new(params)?;

// Evaluate at point
let value = green_fn.evaluate(1.0, -0.5)?;
println!("Green function value: {:?}", value);
```

### Meshes Module

**Module**: `wavecore_meshes`

Mesh operations for marine hydrodynamics.

#### Main Types

```rust
pub struct Mesh {
    pub vertices: Vec<Point>,
    pub faces: Vec<Face>,
    pub normals: Vec<Vector>,
}

pub struct MeshQuality {
    pub min_quality: f64,
    pub max_quality: f64,
    pub average_quality: f64,
    pub degenerate_elements: usize,
    pub inverted_elements: usize,
    pub total_elements: usize,
}

pub struct MeshStats {
    pub vertices: usize,
    pub faces: usize,
    pub edges: usize,
    pub bounding_box: (Point, Point),
    pub surface_area: f64,
    pub volume: f64,
}

pub enum GeometryType {
    Sphere,
    Cylinder,
    Box,
    ShipHull,
    Custom,
}

pub enum Transformation {
    Translation(Vector),
    Rotation { axis: Vector, angle: f64 },
    Scaling { x: f64, y: f64, z: f64 },
    Combined(Vec<Transformation>),
}
```

#### Main Functions

```rust
impl Mesh {
    /// Create new mesh from vertices and faces
    pub fn new(vertices: Vec<Point>, faces: Vec<Face>) -> Result<Self>
    
    /// Get mesh statistics
    pub fn stats(&self) -> MeshStats
    
    /// Validate mesh quality
    pub fn validate(&self) -> Result<MeshQuality>
    
    /// Apply transformation
    pub fn transform(&mut self, transformation: &Transformation) -> Result<()>
    
    /// Refine mesh
    pub fn refine(&self, criteria: &RefinementCriteria) -> Result<Mesh>
}

impl PredefinedGeometry {
    /// Create sphere mesh
    pub fn sphere(radius: f64, theta_res: usize, phi_res: usize) -> Result<Mesh>
    
    /// Create cylinder mesh
    pub fn cylinder(radius: f64, height: f64, theta_res: usize, z_res: usize) -> Result<Mesh>
    
    /// Create box mesh
    pub fn box_mesh(dimensions: [f64; 3], resolution: [usize; 3]) -> Result<Mesh>
}
```

#### Usage Example

```rust
use wavecore_meshes::{Mesh, PredefinedGeometry, Transformation, Vector};

// Create sphere mesh
let sphere = PredefinedGeometry::sphere(1.0, 32, 16)?;

// Apply transformation
let translation = Transformation::Translation(Vector::new(0.0, 0.0, -1.0));
sphere.transform(&translation)?;

// Validate mesh
let quality = sphere.validate()?;
println!("Mesh quality: {:?}", quality);
```

### Matrices Module

**Module**: `wavecore_matrices`

High-performance linear algebra and matrix operations.

#### Main Types

```rust
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

pub enum SolverType {
    LU,
    Cholesky,
    GMRES,
    ConjugateGradient,
    BiCGSTAB,
}

pub trait LinearSolverTrait {
    fn solve(&self, a: &Matrix, b: &[f64]) -> Result<Vec<f64>>;
    fn solver_type(&self) -> SolverType;
}

pub struct LinearSolver {
    solver_type: SolverType,
}
```

#### Main Functions

```rust
impl Matrix {
    /// Create new matrix
    pub fn new(rows: usize, cols: usize) -> Self
    
    /// Create matrix from vector
    pub fn from_vec(rows: usize, cols: usize, data: Vec<f64>) -> Result<Self>
    
    /// Get element at position
    pub fn get(&self, i: usize, j: usize) -> Result<f64>
    
    /// Set element at position
    pub fn set(&mut self, i: usize, j: usize, value: f64) -> Result<()>
    
    /// Get dimensions
    pub fn dimensions(&self) -> (usize, usize)
    
    /// Check if square matrix
    pub fn is_square(&self) -> bool
    
    /// Check if symmetric
    pub fn is_symmetric(&self) -> bool
}

impl LinearSolver {
    /// Create new linear solver
    pub fn new(solver_type: SolverType) -> Self
    
    /// Get solver type
    pub fn solver_type(&self) -> SolverType
}
```

#### Usage Example

```rust
use wavecore_matrices::{Matrix, LinearSolver, SolverType};

// Create matrix
let matrix = Matrix::from_vec(3, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0])?;

// Create solver
let solver = LinearSolver::new(SolverType::LU);

// Solve system
let b = vec![1.0, 2.0, 3.0];
let x = solver.solve(&matrix, &b)?;
println!("Solution: {:?}", x);
```

### Bodies Module

**Module**: `wavecore_bodies`

Floating body definitions for marine hydrodynamics.

#### Main Types

```rust
pub struct FloatingBody {
    pub name: String,
    pub mass_properties: MassProperties,
    pub hydrostatic_properties: HydrostaticProperties,
    pub pose: BodyPose,
    pub dofs: [bool; 6],
}

pub enum DOF {
    Surge,
    Sway,
    Heave,
    Roll,
    Pitch,
    Yaw,
}

pub struct MassProperties {
    pub mass: f64,
    pub center_of_gravity: [f64; 3],
    pub inertia_matrix: [[f64; 3]; 3],
}

pub struct HydrostaticProperties {
    pub displaced_volume: f64,
    pub center_of_buoyancy: [f64; 3],
    pub waterplane_area: f64,
    pub metacentric_height: f64,
    pub hydrostatic_stiffness: [[f64; 6]; 6],
}

pub struct BodyPose {
    pub position: [f64; 3],
    pub orientation: [f64; 3],
}
```

#### Main Functions

```rust
impl FloatingBody {
    /// Create new floating body
    pub fn new(name: String, mass_properties: MassProperties) -> Result<Self>
    
    /// Set degree of freedom
    pub fn set_dof(&mut self, dof: DOF, enabled: bool) -> Result<()>
    
    /// Get degree of freedom status
    pub fn get_dof(&self, dof: DOF) -> bool
    
    /// Update pose
    pub fn update_pose(&mut self, pose: BodyPose) -> Result<()>
    
    /// Calculate hydrostatic properties
    pub fn calculate_hydrostatics(&mut self, mesh: &Mesh) -> Result<()>
}

impl DOF {
    /// Get all DOFs
    pub fn all() -> Vec<DOF>
    
    /// Get translation DOFs
    pub fn translations() -> Vec<DOF>
    
    /// Get rotation DOFs
    pub fn rotations() -> Vec<DOF>
    
    /// Get DOF index
    pub fn index(&self) -> usize
    
    /// Get DOF name
    pub fn name(&self) -> &'static str
}
```

#### Usage Example

```rust
use wavecore_bodies::{FloatingBody, MassProperties, DOF};

// Create mass properties
let mass_props = MassProperties {
    mass: 1000.0,
    center_of_gravity: [0.0, 0.0, -1.0],
    inertia_matrix: [[1000.0, 0.0, 0.0], [0.0, 1000.0, 0.0], [0.0, 0.0, 1000.0]],
};

// Create floating body
let mut body = FloatingBody::new("ship".to_string(), mass_props)?;

// Enable DOFs
body.set_dof(DOF::Surge, true)?;
body.set_dof(DOF::Sway, true)?;
body.set_dof(DOF::Heave, true)?;

println!("Body name: {}", body.name);
```

### I/O Module

**Module**: `wavecore_io`

Input/Output operations for marine hydrodynamics.

#### Main Types

```rust
pub enum Format {
    STL,
    OBJ,
    NEMOH,
    WAMIT,
    JSON,
    YAML,
    Binary,
    CSV,
    NetCDF,
}

pub enum DataType {
    Float32,
    Float64,
    Int32,
    Int64,
    Complex64,
    Complex128,
}

pub struct FileMetadata {
    pub format: Format,
    pub size: u64,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub attributes: std::collections::HashMap<String, String>,
}

pub struct IOStats {
    pub files_read: usize,
    pub files_written: usize,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub avg_read_time: f64,
    pub avg_write_time: f64,
}
```

#### Main Functions

```rust
impl FileIO {
    /// Load mesh from file
    pub fn load_mesh(path: &str, format: Format) -> Result<Mesh>
    
    /// Save mesh to file
    pub fn save_mesh(mesh: &Mesh, path: &str, format: Format) -> Result<()>
    
    /// Load data array
    pub fn load_data(path: &str, format: Format) -> Result<DataArray>
    
    /// Save data array
    pub fn save_data(data: &DataArray, path: &str, format: Format) -> Result<()>
    
    /// Get file metadata
    pub fn get_metadata(path: &str) -> Result<FileMetadata>
}

impl Format {
    /// Get file extension
    pub fn extension(&self) -> &'static str
    
    /// Get MIME type
    pub fn mime_type(&self) -> &'static str
    
    /// Check if text format
    pub fn is_text(&self) -> bool
    
    /// Check if binary format
    pub fn is_binary(&self) -> bool
}
```

#### Usage Example

```rust
use wavecore_io::{FileIO, Format, DataArray};

// Load mesh from STL file
let mesh = FileIO::load_mesh("hull.stl", Format::STL)?;

// Create data array
let data = DataArray::new(&[100, 50], &vec![1.0; 5000])?;

// Save to JSON
FileIO::save_data(&data, "results.json", Format::JSON)?;
```

### Validation Module

**Module**: `wavecore_validation`

Industry-standard validation benchmarks for marine hydrodynamics.

#### Main Types

```rust
pub struct ValidationFramework {
    pub criteria: ValidationCriteria,
    pub reference_data: ReferenceDatabase,
}

pub struct ValidationReport {
    pub benchmark_name: String,
    pub timestamp: String,
    pub results: SeakeepingResults,
    pub statistics: StatisticalAnalysis,
    pub passed: bool,
}

pub struct ValidationCriteria {
    pub max_relative_error: f64,
    pub max_absolute_error: f64,
    pub min_correlation: f64,
    pub significance_level: f64,
}

pub struct SeakeepingResults {
    pub added_mass: HashMap<String, f64>,
    pub damping: HashMap<String, f64>,
    pub exciting_forces: HashMap<String, f64>,
    pub raos: HashMap<String, f64>,
}

pub trait Benchmark {
    type Config;
    type Results;
    
    fn new(config: Self::Config) -> Self;
    fn run_tests(&self) -> ValidationResult<Self::Results>;
    fn validate(&self, results: &Self::Results) -> ValidationResult<ValidationReport>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}
```

#### Main Functions

```rust
impl ValidationFramework {
    /// Create new validation framework
    pub fn new() -> Self
    
    /// Run all validations
    pub fn run_all_validations(&self) -> ValidationResult<HashMap<String, ValidationReport>>
    
    /// Quick validation check
    pub fn quick_validation(&self) -> ValidationResult<bool>
}

// Global functions
pub fn initialize() -> ValidationResult<ValidationFramework>
pub fn run_all_validations() -> ValidationResult<HashMap<String, ValidationReport>>
pub fn quick_validation() -> ValidationResult<bool>
```

#### Usage Example

```rust
use wavecore_validation::{ValidationFramework, DTMB5415Benchmark, DTMB5415Config};

// Create validation framework
let framework = ValidationFramework::new();

// Run DTMB 5415 benchmark
let benchmark = DTMB5415Benchmark::new(DTMB5415Config::default());
let results = benchmark.run_tests()?;

// Validate results
let report = benchmark.validate(&results)?;
println!("Validation passed: {}", report.passed);
```

---

## Supporting Modules

### Resistance Module

**Module**: `wavecore_resistance`

Ship resistance calculations and analysis.

#### Main Types

```rust
pub struct ResistanceCalculator {
    pub method: ResistanceMethod,
    pub parameters: ResistanceParameters,
}

pub enum ResistanceMethod {
    HoltropMennen,
    AddedResistance,
    Windage,
}

pub struct ResistanceParameters {
    pub ship_length: f64,
    pub beam: f64,
    pub draft: f64,
    pub displacement: f64,
    pub speed: f64,
    pub water_density: f64,
    pub kinematic_viscosity: f64,
}

pub struct ResistanceResults {
    pub frictional_resistance: f64,
    pub residual_resistance: f64,
    pub total_resistance: f64,
    pub effective_power: f64,
    pub resistance_coefficients: HashMap<String, f64>,
}
```

#### Main Functions

```rust
impl ResistanceCalculator {
    /// Create new resistance calculator
    pub fn new(method: ResistanceMethod, parameters: ResistanceParameters) -> Self
    
    /// Calculate resistance
    pub fn calculate(&self) -> Result<ResistanceResults>
    
    /// Calculate added resistance
    pub fn calculate_added_resistance(&self, wave_spectrum: &WaveSpectrum) -> Result<f64>
}
```

### Post-Processing Module

**Module**: `wavecore_post_pro`

Advanced result analysis and visualization.

#### Main Types

```rust
pub struct PostProcessor {
    pub analysis_type: AnalysisType,
    pub parameters: AnalysisParameters,
}

pub enum AnalysisType {
    RAOAnalysis,
    MotionAnalysis,
    ForceAnalysis,
    EnergyAnalysis,
}

pub struct AnalysisParameters {
    pub frequency_range: (f64, f64),
    pub direction_range: (f64, f64),
    pub output_format: OutputFormat,
}

pub struct AnalysisResults {
    pub raos: HashMap<String, Vec<Complex64>>,
    pub motion_statistics: MotionStatistics,
    pub force_statistics: ForceStatistics,
    pub energy_analysis: EnergyAnalysis,
}
```

#### Main Functions

```rust
impl PostProcessor {
    /// Create new post processor
    pub fn new(analysis_type: AnalysisType, parameters: AnalysisParameters) -> Self
    
    /// Process BEM results
    pub fn process(&self, bem_results: &BEMResults) -> Result<AnalysisResults>
    
    /// Generate reports
    pub fn generate_report(&self, results: &AnalysisResults) -> Result<String>
}
```

### UI Module

**Module**: `wavecore_ui`

Command-line and web interfaces.

#### Main Types

```rust
pub struct CLI {
    pub config: CLIConfig,
}

pub struct WebInterface {
    pub config: WebConfig,
}

pub struct CLIConfig {
    pub interactive: bool,
    pub batch_mode: bool,
    pub output_format: OutputFormat,
}

pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub static_files: String,
}
```

#### Main Functions

```rust
impl CLI {
    /// Create new CLI interface
    pub fn new(config: CLIConfig) -> Self
    
    /// Run interactive shell
    pub fn run_interactive(&self) -> Result<()>
    
    /// Process batch commands
    pub fn process_batch(&self, commands: &[String]) -> Result<()>
}

impl WebInterface {
    /// Create new web interface
    pub fn new(config: WebConfig) -> Self
    
    /// Start web server
    pub async fn start(&self) -> Result<()>
    
    /// Handle API requests
    pub async fn handle_request(&self, request: Request) -> Result<Response>
}
```

### GPU Module

**Module**: `wavecore_gpu`

CUDA acceleration and device management.

#### Main Types

```rust
pub struct GPUDevice {
    pub device_id: u32,
    pub memory: u64,
    pub compute_capability: (u32, u32),
}

pub struct GPUSolver {
    pub device: GPUDevice,
    pub config: GPUSolverConfig,
}

pub struct GPUSolverConfig {
    pub block_size: u32,
    pub memory_limit: u64,
    pub use_double_precision: bool,
}
```

#### Main Functions

```rust
impl GPUDevice {
    /// Get available devices
    pub fn list_devices() -> Result<Vec<GPUDevice>>
    
    /// Initialize device
    pub fn new(device_id: u32) -> Result<Self>
    
    /// Get device memory
    pub fn memory(&self) -> u64
}

impl GPUSolver {
    /// Create new GPU solver
    pub fn new(device: GPUDevice, config: GPUSolverConfig) -> Result<Self>
    
    /// Solve BEM problem on GPU
    pub fn solve_bem(&self, problem: &ProblemType, mesh: &Mesh) -> Result<BEMResults>
}
```

---

## Error Handling

All modules use consistent error handling with custom error types:

### Error Types

```rust
// Common error pattern across modules
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Invalid data: {message}")]
    InvalidData { message: String },
    
    #[error("Operation failed: {message}")]
    OperationError { message: String },
    
    #[error("Memory allocation failed")]
    MemoryError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Result Types

```rust
// Standard result type pattern
pub type Result<T> = std::result::Result<T, ModuleError>;
```

### Error Handling Example

```rust
use wavecore_bem::{BEMSolver, SolverEngine, BEMError};

fn solve_problem() -> Result<(), BEMError> {
    let solver = BEMSolver::new(SolverEngine::Standard);
    
    match solver.solve(&problem, &mesh) {
        Ok(results) => {
            println!("Problem solved successfully");
            Ok(())
        }
        Err(BEMError::InvalidProblem { message }) => {
            eprintln!("Invalid problem: {}", message);
            Err(BEMError::InvalidProblem { message })
        }
        Err(e) => {
            eprintln!("Solver error: {}", e);
            Err(e)
        }
    }
}
```

---

## Common Types

### Mathematical Types

```rust
// Re-exported from nalgebra
pub type Point = Point3<f64>;
pub type Vector = Vector3<f64>;
pub type Matrix = Matrix3<f64>;

// Complex numbers
pub type Complex = Complex64;
```

### Configuration Types

```rust
// Common configuration pattern
pub struct Config {
    pub tolerance: f64,
    pub max_iterations: usize,
    pub parallel: bool,
    pub memory_limit: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tolerance: 1e-6,
            max_iterations: 1000,
            parallel: true,
            memory_limit: None,
        }
    }
}
```

### Result Types

```rust
// Common result structures
pub struct AnalysisResults {
    pub success: bool,
    pub data: HashMap<String, f64>,
    pub metadata: HashMap<String, String>,
    pub errors: Vec<String>,
}

pub struct ValidationResults {
    pub passed: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub statistics: StatisticalMetrics,
}
```

---

## Examples

### Complete BEM Analysis

```rust
use wavecore_bem::{BEMSolver, SolverEngine, ProblemType};
use wavecore_meshes::PredefinedGeometry;
use wavecore_green_functions::{GreenFunction, GreenFunctionParams, Method};
use wavecore_bodies::{FloatingBody, MassProperties, DOF};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sphere mesh
    let mesh = PredefinedGeometry::sphere(1.0, 32, 16)?;
    
    // Create floating body
    let mass_props = MassProperties {
        mass: 1000.0,
        center_of_gravity: [0.0, 0.0, -1.0],
        inertia_matrix: [[1000.0, 0.0, 0.0], [0.0, 1000.0, 0.0], [0.0, 0.0, 1000.0]],
    };
    let mut body = FloatingBody::new("sphere".to_string(), mass_props)?;
    body.set_dof(DOF::Heave, true)?;
    
    // Create BEM solver
    let solver = BEMSolver::new(SolverEngine::Standard);
    
    // Create radiation problem
    let problem = ProblemType::Radiation {
        frequency: 1.0,
        mode: 2, // Heave
    };
    
    // Solve problem
    let results = solver.solve(&problem, &mesh)?;
    
    // Extract results
    let added_mass = results.added_mass_matrix();
    let damping = results.damping_matrix();
    
    println!("Added mass (3,3): {:.3}", added_mass[(2, 2)]);
    println!("Damping (3,3): {:.3}", damping[(2, 2)]);
    
    Ok(())
}
```

### Ship Hull Analysis

```rust
use wavecore_io::FileIO;
use wavecore_bem::{BEMSolver, SolverEngine, ProblemType};
use wavecore_validation::DTMB5415Benchmark;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load DTMB 5415 hull
    let mesh = FileIO::load_mesh("dtmb5415.stl", Format::STL)?;
    
    // Create solver
    let solver = BEMSolver::new(SolverEngine::Standard);
    
    // Setup frequency range
    let frequencies = vec![0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6];
    let mut results = Vec::new();
    
    for &freq in &frequencies {
        let problem = ProblemType::Radiation {
            frequency: freq,
            mode: 2, // Heave
        };
        
        let result = solver.solve(&problem, &mesh)?;
        results.push((freq, result));
    }
    
    // Validate against reference data
    let benchmark = DTMB5415Benchmark::new(DTMB5415Config::default());
    let validation = benchmark.validate(&results)?;
    
    println!("Validation passed: {}", validation.passed);
    
    Ok(())
}
```

### GPU-Accelerated Analysis

```rust
use wavecore_gpu::{GPUDevice, GPUSolver, GPUSolverConfig};
use wavecore_bem::ProblemType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get available GPU devices
    let devices = GPUDevice::list_devices()?;
    let device = devices.first().ok_or("No GPU devices available")?;
    
    // Create GPU solver
    let config = GPUSolverConfig {
        block_size: 256,
        memory_limit: 1024 * 1024 * 1024, // 1GB
        use_double_precision: true,
    };
    let gpu_solver = GPUSolver::new(device.clone(), config)?;
    
    // Solve on GPU
    let problem = ProblemType::Radiation {
        frequency: 1.0,
        mode: 2,
    };
    
    let results = gpu_solver.solve_bem(&problem, &mesh)?;
    println!("GPU solution completed");
    
    Ok(())
}
```

---

## Performance Guidelines

### Memory Management

```rust
// Use efficient data structures
let matrix = Matrix::new(rows, cols); // Pre-allocate
let mut results = Vec::with_capacity(expected_size); // Pre-allocate

// Use references to avoid copying
fn process_mesh(mesh: &Mesh) -> Result<ProcessedMesh> {
    // Process without copying
}

// Use parallel processing where possible
use rayon::prelude::*;
let results: Vec<_> = data.par_iter().map(process_item).collect();
```

### GPU Optimization

```rust
// Batch operations for GPU efficiency
let problems: Vec<ProblemType> = create_problem_batch();
let results = gpu_solver.solve_batch(&problems, &mesh)?;

// Use appropriate precision
let config = GPUSolverConfig {
    use_double_precision: false, // Use single precision for speed
    ..Default::default()
};
```

### Algorithm Selection

```rust
// Choose appropriate solver based on problem size
let solver_type = if matrix_size < 1000 {
    SolverType::LU // Direct solver for small problems
} else {
    SolverType::GMRES // Iterative solver for large problems
};

// Choose appropriate Green function method
let method = if depth.is_infinite() {
    Method::Delhommeau // Infinite depth
} else {
    Method::FinGreen3D // Finite depth
};
```

---

## Conclusion

This API reference provides comprehensive documentation for the WaveCore marine hydrodynamics solver. The modular design allows for flexible usage while maintaining high performance and accuracy. For additional examples and advanced usage patterns, refer to the examples directory and user guide.

### Key Features Summary

- **Type Safety**: Strong typing with comprehensive error handling
- **Performance**: Optimized algorithms with GPU acceleration
- **Modularity**: Clean separation of concerns across modules
- **Industry Standards**: Compliance with marine engineering standards
- **Extensibility**: Trait-based interfaces for easy extension

### Getting Help

- **Documentation**: See `WAVECORE_USER_GUIDE.md` for detailed usage
- **Examples**: Check the `examples/` directory for working code
- **Validation**: Use the validation module for benchmarking
- **Issues**: Report problems through the project issue tracker

---

**WaveCore API Reference v0.1.0**  
*Part of the OceanOS Platform - Advancing marine technology through open-source innovation* 