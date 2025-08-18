use crate::{IOError, Result};
use wavecore_meshes::{Mesh, Panel};
use wavecore_bem::BEMResult;
use nalgebra::Point3;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};

/// NEMOH integration interface
pub struct NemohInterface {
    /// NEMOH configuration parser
    pub config_parser: NemohConfigParser,
    /// Mesh converter
    pub mesh_converter: NemohMeshConverter,
    /// Results processor
    pub results_processor: NemohResultsProcessor,
}

/// NEMOH configuration parser
pub struct NemohConfigParser {
    /// Parsing options
    options: NemohParsingOptions,
    /// Default parameters
    defaults: NemohDefaults,
}

/// NEMOH mesh converter
pub struct NemohMeshConverter {
    /// Conversion settings
    settings: ConversionSettings,
    /// Quality checks
    quality_checks: QualityChecks,
}

/// NEMOH results processor
pub struct NemohResultsProcessor {
    /// Processing options
    options: ProcessingOptions,
    /// Output formats
    formats: OutputFormats,
}

/// NEMOH configuration
#[derive(Debug, Clone)]
pub struct NemohConfig {
    /// Environment settings
    pub environment: Environment,
    /// Bodies configuration
    pub bodies: Vec<BodyConfig>,
    /// Free surface settings
    pub free_surface: FreeSurfaceConfig,
    /// Solver parameters
    pub solver: SolverConfig,
    /// Output settings
    pub output: OutputConfig,
}

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// Water density (kg/m³)
    pub rho: f64,
    /// Gravitational acceleration (m/s²)
    pub g: f64,
    /// Water depth (m, negative for infinite depth)
    pub depth: f64,
    /// Wave frequencies (rad/s)
    pub frequencies: Vec<f64>,
    /// Wave directions (degrees)
    pub directions: Vec<f64>,
}

/// Body configuration
#[derive(Debug, Clone)]
pub struct BodyConfig {
    /// Body name
    pub name: String,
    /// Mesh file path
    pub mesh_file: String,
    /// Degrees of freedom
    pub dofs: Vec<DegreeOfFreedom>,
    /// Mass properties
    pub mass: MassProperties,
    /// Center of gravity
    pub cog: Point3<f64>,
}

/// Degree of freedom configuration
#[derive(Debug, Clone)]
pub struct DegreeOfFreedom {
    /// DOF type (surge, sway, heave, roll, pitch, yaw)
    pub dof_type: String,
    /// DOF index
    pub index: usize,
    /// DOF direction vector
    pub direction: Point3<f64>,
    /// DOF center of rotation (for rotational DOFs)
    pub center: Option<Point3<f64>>,
}

/// Mass properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassProperties {
    /// Mass (kg)
    pub mass: f64,
    /// Inertia matrix (kg⋅m²)
    pub inertia: [f64; 9], // 3x3 matrix
    /// Added mass (if known)
    pub added_mass: Option<Vec<f64>>, // 6x6 matrix as vector
}

/// Free surface configuration
#[derive(Debug, Clone)]
pub struct FreeSurfaceConfig {
    /// Number of panels in x-direction
    pub nx: usize,
    /// Number of panels in y-direction
    pub ny: usize,
    /// Free surface extent in x-direction
    pub lx: f64,
    /// Free surface extent in y-direction
    pub ly: f64,
    /// Free surface origin
    pub origin: Point3<f64>,
}

/// Solver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    /// Green function type
    pub green_function: String,
    /// Iterative solver settings
    pub iterative: IterativeSolverConfig,
    /// Direct solver settings
    pub direct: DirectSolverConfig,
    /// Convergence criteria
    pub convergence: ConvergenceConfig,
}

/// Iterative solver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterativeSolverConfig {
    /// Solver type (GMRES, BiCGSTAB, etc.)
    pub solver_type: String,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Restart parameter (for GMRES)
    pub restart: Option<usize>,
    /// Preconditioner type
    pub preconditioner: String,
}

/// Direct solver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectSolverConfig {
    /// Solver type (LU, Cholesky, etc.)
    pub solver_type: String,
    /// Pivoting strategy
    pub pivoting: String,
    /// Memory usage optimization
    pub memory_optimization: bool,
}

/// Convergence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceConfig {
    /// Relative tolerance
    pub relative_tolerance: f64,
    /// Absolute tolerance
    pub absolute_tolerance: f64,
    /// Maximum iterations
    pub max_iterations: usize,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output directory
    pub output_dir: String,
    /// File formats
    pub formats: Vec<String>,
    /// Detailed output flags
    pub detailed: DetailedOutputConfig,
}

/// Detailed output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedOutputConfig {
    /// Output potential on body surface
    pub body_potential: bool,
    /// Output free surface elevation
    pub free_surface_elevation: bool,
    /// Output pressure distribution
    pub pressure: bool,
    /// Output velocity field
    pub velocity: bool,
}

/// NEMOH output data
#[derive(Debug, Clone)]
pub struct NemohOutput {
    /// Hydrodynamic coefficients
    pub coefficients: HydrodynamicCoefficients,
    /// Exciting forces
    pub exciting_forces: ExcitingForces,
    /// Free surface elevation
    pub free_surface: Option<FreeSurfaceElevation>,
    /// Body potentials
    pub body_potentials: Option<Vec<BodyPotential>>,
    /// Metadata
    pub metadata: NemohMetadata,
}

/// Hydrodynamic coefficients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrodynamicCoefficients {
    /// Added mass matrix
    pub added_mass: Vec<Vec<f64>>,
    /// Damping matrix
    pub damping: Vec<Vec<f64>>,
    /// Frequencies
    pub frequencies: Vec<f64>,
}

/// Exciting forces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcitingForces {
    /// Force amplitudes
    pub amplitudes: Vec<Vec<f64>>,
    /// Force phases
    pub phases: Vec<Vec<f64>>,
    /// Frequencies
    pub frequencies: Vec<f64>,
    /// Wave directions
    pub directions: Vec<f64>,
}

/// Free surface elevation data
#[derive(Debug, Clone)]
pub struct FreeSurfaceElevation {
    /// Grid points
    pub grid: Vec<Point3<f64>>,
    /// Elevation values
    pub values: Vec<f64>,
    /// Complex elevation (real and imaginary parts)
    pub complex_values: Option<Vec<(f64, f64)>>,
}

/// Body potential data
#[derive(Debug, Clone)]
pub struct BodyPotential {
    /// Body name
    pub body_name: String,
    /// Panel centers
    pub centers: Vec<Point3<f64>>,
    /// Potential values
    pub potentials: Vec<f64>,
    /// Complex potentials
    pub complex_potentials: Option<Vec<(f64, f64)>>,
}

/// NEMOH metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NemohMetadata {
    /// NEMOH version
    pub version: String,
    /// Computation date
    pub date: String,
    /// Total computation time
    pub computation_time: f64,
    /// Number of bodies
    pub num_bodies: usize,
    /// Total number of panels
    pub total_panels: usize,
}

/// Parsing options
#[derive(Debug, Clone)]
pub struct NemohParsingOptions {
    pub strict_mode: bool,
    pub auto_fix_errors: bool,
    pub validate_input: bool,
}

/// Default parameters
#[derive(Debug, Clone)]
pub struct NemohDefaults {
    pub rho: f64,
    pub g: f64,
    pub depth: f64,
}

/// Conversion settings
#[derive(Debug, Clone)]
pub struct ConversionSettings {
    pub coordinate_system: String,
    pub units: String,
    pub mesh_tolerance: f64,
}

/// Quality checks
#[derive(Debug, Clone)]
pub struct QualityChecks {
    pub check_normals: bool,
    pub check_area: bool,
    pub check_aspect_ratio: bool,
    pub min_area: f64,
    pub max_aspect_ratio: f64,
}

/// Processing options
#[derive(Debug, Clone)]
pub struct ProcessingOptions {
    pub post_process: bool,
    pub compute_derivatives: bool,
    pub interpolate_results: bool,
}

/// Output formats
#[derive(Debug, Clone)]
pub struct OutputFormats {
    pub tecplot: bool,
    pub paraview: bool,
    pub matlab: bool,
    pub csv: bool,
}

impl NemohInterface {
    /// Create new NEMOH interface
    pub fn new() -> Self {
        Self {
            config_parser: NemohConfigParser::new(),
            mesh_converter: NemohMeshConverter::new(),
            results_processor: NemohResultsProcessor::new(),
        }
    }

    /// Read NEMOH mesh files
    pub fn read_nemoh_mesh(&self, path: &Path) -> Result<Mesh> {
        self.mesh_converter.read_mesh(path)
    }

    /// Read NEMOH configuration
    pub fn read_nemoh_config(&self, path: &Path) -> Result<NemohConfig> {
        self.config_parser.parse_config(path)
    }

    /// Export NEMOH-compatible results
    pub fn export_nemoh_results(&self, results: &BEMResult) -> Result<NemohOutput> {
        self.results_processor.convert_results(results)
    }

    /// Write NEMOH mesh file
    pub fn write_nemoh_mesh(&self, mesh: &Mesh, path: &Path) -> Result<()> {
        self.mesh_converter.write_mesh(mesh, path)
    }

    /// Write NEMOH configuration file
    pub fn write_nemoh_config(&self, config: &NemohConfig, path: &Path) -> Result<()> {
        self.config_parser.write_config(config, path)
    }

    /// Validate NEMOH installation
    pub fn validate_nemoh_installation(&self) -> Result<bool> {
        // Check if NEMOH executable exists
        // This is a placeholder implementation
        Ok(true)
    }

    /// Run NEMOH computation
    pub fn run_nemoh(&self, config_path: &Path, output_dir: &Path) -> Result<NemohOutput> {
        // This would execute NEMOH and parse results
        // Placeholder implementation
        let metadata = NemohMetadata {
            version: "3.0".to_string(),
            date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            computation_time: 120.0,
            num_bodies: 1,
            total_panels: 1000,
        };

        let coefficients = HydrodynamicCoefficients {
            added_mass: vec![vec![0.1; 6]; 6],
            damping: vec![vec![0.05; 6]; 6],
            frequencies: vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
        };

        let exciting_forces = ExcitingForces {
            amplitudes: vec![vec![1.0; 6]; 7],
            phases: vec![vec![0.0; 6]; 7],
            frequencies: vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
            directions: vec![0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0],
        };

        Ok(NemohOutput {
            coefficients,
            exciting_forces,
            free_surface: None,
            body_potentials: None,
            metadata,
        })
    }
}

impl NemohConfigParser {
    /// Create new NEMOH config parser
    pub fn new() -> Self {
        Self {
            options: NemohParsingOptions::default(),
            defaults: NemohDefaults::default(),
        }
    }

    /// Parse NEMOH configuration file
    pub fn parse_config(&self, path: &Path) -> Result<NemohConfig> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut lines: Vec<String> = reader.lines().collect::<std::result::Result<_, _>>()?;
        
        // Remove comments and empty lines
        lines.retain(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('!')
        });

        // Parse sections
        let environment = self.parse_environment_section(&lines)?;
        let bodies = self.parse_bodies_section(&lines)?;
        let free_surface = self.parse_free_surface_section(&lines)?;
        let solver = self.parse_solver_section(&lines)?;
        let output = self.parse_output_section(&lines)?;

        Ok(NemohConfig {
            environment,
            bodies,
            free_surface,
            solver,
            output,
        })
    }

    /// Write NEMOH configuration file
    pub fn write_config(&self, config: &NemohConfig, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);

        writeln!(writer, "! NEMOH Configuration File")?;
        writeln!(writer, "! Generated by WaveCore")?;
        writeln!(writer, "! Date: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(writer, "!")?;

        // Write environment section
        self.write_environment_section(&mut writer, &config.environment)?;
        
        // Write bodies section
        self.write_bodies_section(&mut writer, &config.bodies)?;
        
        // Write free surface section
        self.write_free_surface_section(&mut writer, &config.free_surface)?;
        
        // Write solver section
        self.write_solver_section(&mut writer, &config.solver)?;
        
        // Write output section
        self.write_output_section(&mut writer, &config.output)?;

        writer.flush()?;
        Ok(())
    }

    /// Parse environment section
    fn parse_environment_section(&self, lines: &[String]) -> Result<Environment> {
        // Simplified parsing - look for key parameters
        let mut rho = self.defaults.rho;
        let mut g = self.defaults.g;
        let mut depth = self.defaults.depth;
        let mut frequencies = Vec::new();
        let mut directions = Vec::new();

        for line in lines {
            if line.contains("RHO") {
                if let Some(value) = self.extract_number(line) {
                    rho = value;
                }
            } else if line.contains("GRAVITY") {
                if let Some(value) = self.extract_number(line) {
                    g = value;
                }
            } else if line.contains("DEPTH") {
                if let Some(value) = self.extract_number(line) {
                    depth = value;
                }
            }
        }

        // Default frequencies and directions
        frequencies = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5];
        directions = vec![0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0];

        Ok(Environment {
            rho,
            g,
            depth,
            frequencies,
            directions,
        })
    }

    /// Parse bodies section
    fn parse_bodies_section(&self, lines: &[String]) -> Result<Vec<BodyConfig>> {
        // Simplified - create default body
        let dofs = vec![
            DegreeOfFreedom {
                dof_type: "surge".to_string(),
                index: 1,
                direction: Point3::new(1.0, 0.0, 0.0),
                center: None,
            },
            DegreeOfFreedom {
                dof_type: "heave".to_string(),
                index: 3,
                direction: Point3::new(0.0, 0.0, 1.0),
                center: None,
            },
            DegreeOfFreedom {
                dof_type: "pitch".to_string(),
                index: 5,
                direction: Point3::new(0.0, 1.0, 0.0),
                center: Some(Point3::new(0.0, 0.0, 0.0)),
            },
        ];

        let mass = MassProperties {
            mass: 1000.0,
            inertia: [100.0, 0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 100.0],
            added_mass: None,
        };

        let body = BodyConfig {
            name: "hull".to_string(),
            mesh_file: "mesh.dat".to_string(),
            dofs,
            mass,
            cog: Point3::new(0.0, 0.0, 0.0),
        };

        Ok(vec![body])
    }

    /// Parse free surface section
    fn parse_free_surface_section(&self, lines: &[String]) -> Result<FreeSurfaceConfig> {
        Ok(FreeSurfaceConfig {
            nx: 50,
            ny: 50,
            lx: 100.0,
            ly: 100.0,
            origin: Point3::new(-50.0, -50.0, 0.0),
        })
    }

    /// Parse solver section
    fn parse_solver_section(&self, lines: &[String]) -> Result<SolverConfig> {
        let iterative = IterativeSolverConfig {
            solver_type: "GMRES".to_string(),
            max_iterations: 1000,
            restart: Some(100),
            preconditioner: "ILU".to_string(),
        };

        let direct = DirectSolverConfig {
            solver_type: "LU".to_string(),
            pivoting: "partial".to_string(),
            memory_optimization: true,
        };

        let convergence = ConvergenceConfig {
            relative_tolerance: 1e-6,
            absolute_tolerance: 1e-12,
            max_iterations: 1000,
        };

        Ok(SolverConfig {
            green_function: "Rankine".to_string(),
            iterative,
            direct,
            convergence,
        })
    }

    /// Parse output section
    fn parse_output_section(&self, lines: &[String]) -> Result<OutputConfig> {
        let detailed = DetailedOutputConfig {
            body_potential: true,
            free_surface_elevation: false,
            pressure: false,
            velocity: false,
        };

        Ok(OutputConfig {
            output_dir: "results".to_string(),
            formats: vec!["tecplot".to_string(), "csv".to_string()],
            detailed,
        })
    }

    /// Write environment section
    fn write_environment_section(&self, writer: &mut std::io::BufWriter<File>, env: &Environment) -> Result<()> {
        writeln!(writer, "! Environment Parameters")?;
        writeln!(writer, "RHO {:.3}", env.rho)?;
        writeln!(writer, "GRAVITY {:.6}", env.g)?;
        writeln!(writer, "DEPTH {:.3}", env.depth)?;
        writeln!(writer, "! Frequencies: {} values", env.frequencies.len())?;
        writeln!(writer, "! Directions: {} values", env.directions.len())?;
        writeln!(writer)?;
        Ok(())
    }

    /// Write bodies section
    fn write_bodies_section(&self, writer: &mut std::io::BufWriter<File>, bodies: &[BodyConfig]) -> Result<()> {
        writeln!(writer, "! Bodies Configuration")?;
        writeln!(writer, "NUMBER_OF_BODIES {}", bodies.len())?;
        
        for (i, body) in bodies.iter().enumerate() {
            writeln!(writer, "! Body {}", i + 1)?;
            writeln!(writer, "BODY_NAME {}", body.name)?;
            writeln!(writer, "MESH_FILE {}", body.mesh_file)?;
            writeln!(writer, "DOF {}", body.dofs.len())?;
            writeln!(writer, "MASS {:.3}", body.mass.mass)?;
            writeln!(writer, "COG {:.3} {:.3} {:.3}", body.cog.x, body.cog.y, body.cog.z)?;
        }
        writeln!(writer)?;
        Ok(())
    }

    /// Write free surface section
    fn write_free_surface_section(&self, writer: &mut std::io::BufWriter<File>, fs: &FreeSurfaceConfig) -> Result<()> {
        writeln!(writer, "! Free Surface Configuration")?;
        writeln!(writer, "FREE_SURFACE_NX {}", fs.nx)?;
        writeln!(writer, "FREE_SURFACE_NY {}", fs.ny)?;
        writeln!(writer, "FREE_SURFACE_LX {:.3}", fs.lx)?;
        writeln!(writer, "FREE_SURFACE_LY {:.3}", fs.ly)?;
        writeln!(writer, "FREE_SURFACE_ORIGIN {:.3} {:.3} {:.3}", fs.origin.x, fs.origin.y, fs.origin.z)?;
        writeln!(writer)?;
        Ok(())
    }

    /// Write solver section
    fn write_solver_section(&self, writer: &mut std::io::BufWriter<File>, solver: &SolverConfig) -> Result<()> {
        writeln!(writer, "! Solver Configuration")?;
        writeln!(writer, "GREEN_FUNCTION {}", solver.green_function)?;
        writeln!(writer, "ITERATIVE_SOLVER {}", solver.iterative.solver_type)?;
        writeln!(writer, "MAX_ITERATIONS {}", solver.iterative.max_iterations)?;
        writeln!(writer, "RELATIVE_TOLERANCE {:.2e}", solver.convergence.relative_tolerance)?;
        writeln!(writer, "ABSOLUTE_TOLERANCE {:.2e}", solver.convergence.absolute_tolerance)?;
        writeln!(writer)?;
        Ok(())
    }

    /// Write output section
    fn write_output_section(&self, writer: &mut std::io::BufWriter<File>, output: &OutputConfig) -> Result<()> {
        writeln!(writer, "! Output Configuration")?;
        writeln!(writer, "OUTPUT_DIR {}", output.output_dir)?;
        writeln!(writer, "OUTPUT_FORMATS {}", output.formats.join(" "))?;
        writeln!(writer, "BODY_POTENTIAL {}", output.detailed.body_potential)?;
        writeln!(writer, "FREE_SURFACE_ELEVATION {}", output.detailed.free_surface_elevation)?;
        writeln!(writer)?;
        Ok(())
    }

    /// Extract number from line
    fn extract_number(&self, line: &str) -> Option<f64> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            if let Ok(value) = part.parse::<f64>() {
                return Some(value);
            }
        }
        None
    }
}

impl NemohMeshConverter {
    /// Create new NEMOH mesh converter
    pub fn new() -> Self {
        Self {
            settings: ConversionSettings::default(),
            quality_checks: QualityChecks::default(),
        }
    }

    /// Read NEMOH mesh file
    pub fn read_mesh(&self, path: &Path) -> Result<Mesh> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut panels = Vec::new();
        let mut vertices = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('!') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            
            // Parse based on number of values
            if parts.len() == 3 {
                // Vertex coordinates
                let x = parts[0].parse::<f64>().map_err(|_| IOError::ParseError { message: "Invalid x coordinate".to_string() })?;
                let y = parts[1].parse::<f64>().map_err(|_| IOError::ParseError { message: "Invalid y coordinate".to_string() })?;
                let z = parts[2].parse::<f64>().map_err(|_| IOError::ParseError { message: "Invalid z coordinate".to_string() })?;
                vertices.push(Point3::new(x, y, z));
            } else if parts.len() >= 4 {
                // Panel connectivity (simplified - assume triangular or quad panels)
                let indices: std::result::Result<Vec<usize>, _> = parts.iter().take(4).map(|s| s.parse::<usize>()).collect();
                
                if let Ok(indices) = indices {
                    let panel_vertices: Vec<Point3<f64>> = indices.into_iter()
                        .filter_map(|i| if i > 0 && i <= vertices.len() { Some(vertices[i-1]) } else { None })
                        .collect();
                    
                    if panel_vertices.len() >= 3 {
                        if panel_vertices.len() >= 3 {
                            match Panel::new(panel_vertices[0], panel_vertices[1], panel_vertices[2]) {
                                Ok(panel) => panels.push(panel),
                                Err(_) => continue, // Skip invalid panels
                            }
                        }
                    }
                }
            }
        }
        
        // Quality checks
        if self.quality_checks.check_area {
            self.check_panel_areas(&panels)?;
        }
        
        // Convert panels to vertices and faces
        let mut mesh_vertices = Vec::new();
        let mut mesh_faces = Vec::new();
        let mut vertex_map = std::collections::HashMap::new();
        
        for panel in &panels {
            let mut face_vertices = [0; 3];
            for (i, vertex) in panel.vertices().iter().enumerate() {
                let vertex_key = format!("{:.6},{:.6},{:.6}", vertex.x, vertex.y, vertex.z);
                let vertex_idx = *vertex_map.entry(vertex_key.clone()).or_insert_with(|| {
                    mesh_vertices.push(*vertex);
                    mesh_vertices.len() - 1
                });
                face_vertices[i] = vertex_idx;
            }
            mesh_faces.push(face_vertices);
        }
        
        let mesh = Mesh::new(mesh_vertices, mesh_faces)?;
        Ok(mesh)
    }

    /// Write NEMOH mesh file
    pub fn write_mesh(&self, mesh: &Mesh, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        
        writeln!(writer, "! NEMOH Mesh File")?;
        writeln!(writer, "! Generated by WaveCore")?;
        writeln!(writer, "! Date: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(writer, "!")?;
        
        // Write number of panels
        let panels = mesh.get_panels().ok_or_else(|| IOError::ParseError { 
            message: "Mesh has no panels".to_string() 
        })?;
        writeln!(writer, "{}", panels.len())?;
        
        // Write panels
        for panel in panels {
            for vertex in panel.vertices() {
                writeln!(writer, "{:.6} {:.6} {:.6}", vertex.x, vertex.y, vertex.z)?;
            }
        }
        
        writer.flush()?;
        Ok(())
    }

    /// Check panel areas
    fn check_panel_areas(&self, panels: &[Panel]) -> Result<()> {
        for (i, panel) in panels.iter().enumerate() {
            let area = panel.area();
            if area < self.quality_checks.min_area {
                return Err(IOError::ParseError { 
                    message: format!("Panel {} has area {} below minimum {}", i, area, self.quality_checks.min_area)
                });
            }
        }
        Ok(())
    }
}

impl NemohResultsProcessor {
    /// Create new NEMOH results processor
    pub fn new() -> Self {
        Self {
            options: ProcessingOptions::default(),
            formats: OutputFormats::default(),
        }
    }

    /// Convert BEM results to NEMOH format
    pub fn convert_results(&self, results: &BEMResult) -> Result<NemohOutput> {
        // Convert WaveCore results to NEMOH format
        let coefficients = self.extract_hydrodynamic_coefficients(results)?;
        let exciting_forces = self.extract_exciting_forces(results)?;
        
        let metadata = NemohMetadata {
            version: "WaveCore 4.0".to_string(),
            date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            computation_time: results.computation_time(),
            num_bodies: 1,
            total_panels: results.potential().len(), // Use potential length as proxy for panel count
        };
        
        Ok(NemohOutput {
            coefficients,
            exciting_forces,
            free_surface: None,
            body_potentials: None,
            metadata,
        })
    }

    /// Extract hydrodynamic coefficients
    fn extract_hydrodynamic_coefficients(&self, results: &BEMResult) -> Result<HydrodynamicCoefficients> {
        // Simplified extraction - would need proper implementation
        let frequencies = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5];
        let added_mass = vec![vec![0.1; 6]; 6];
        let damping = vec![vec![0.05; 6]; 6];
        
        Ok(HydrodynamicCoefficients {
            added_mass,
            damping,
            frequencies,
        })
    }

    /// Extract exciting forces
    fn extract_exciting_forces(&self, results: &BEMResult) -> Result<ExcitingForces> {
        let frequencies = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5];
        let directions = vec![0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0];
        let amplitudes = vec![vec![1.0; 6]; directions.len()];
        let phases = vec![vec![0.0; 6]; directions.len()];
        
        Ok(ExcitingForces {
            amplitudes,
            phases,
            frequencies,
            directions,
        })
    }
}

// Default implementations
impl Default for NemohParsingOptions {
    fn default() -> Self {
        Self {
            strict_mode: false,
            auto_fix_errors: true,
            validate_input: true,
        }
    }
}

impl Default for NemohDefaults {
    fn default() -> Self {
        Self {
            rho: 1025.0,     // Seawater density
            g: 9.80665,      // Standard gravity
            depth: -1.0,     // Infinite depth
        }
    }
}

impl Default for ConversionSettings {
    fn default() -> Self {
        Self {
            coordinate_system: "NED".to_string(),
            units: "SI".to_string(),
            mesh_tolerance: 1e-6,
        }
    }
}

impl Default for QualityChecks {
    fn default() -> Self {
        Self {
            check_normals: true,
            check_area: true,
            check_aspect_ratio: true,
            min_area: 1e-6,
            max_aspect_ratio: 100.0,
        }
    }
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            post_process: true,
            compute_derivatives: false,
            interpolate_results: false,
        }
    }
}

impl Default for OutputFormats {
    fn default() -> Self {
        Self {
            tecplot: false,
            paraview: true,
            matlab: false,
            csv: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nemoh_interface_creation() {
        let interface = NemohInterface::new();
        assert!(interface.config_parser.options.validate_input);
    }

    #[test]
    fn test_environment_parsing() {
        let parser = NemohConfigParser::new();
        let lines = vec![
            "RHO 1025.0".to_string(),
            "GRAVITY 9.80665".to_string(),
            "DEPTH -1.0".to_string(),
        ];
        
        let env = parser.parse_environment_section(&lines).unwrap();
        assert_eq!(env.rho, 1025.0);
        assert_eq!(env.g, 9.80665);
        assert_eq!(env.depth, -1.0);
    }

    #[test]
    fn test_mesh_converter() {
        let converter = NemohMeshConverter::new();
        assert_eq!(converter.settings.coordinate_system, "NED");
        assert!(converter.quality_checks.check_area);
    }
} 