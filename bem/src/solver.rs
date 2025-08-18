//! BEM solver implementation with matrix assembly

use super::*;
use wavecore_matrices::{Matrix, LinearSolver, LinearSolverTrait, SolverType};
use wavecore_green_functions::{GreenFunction, GreenFunctionParams, Method};
use wavecore_meshes::{Mesh, Panel};
use wavecore_bodies::{FloatingBody};
use nalgebra::Point3;
use rayon::prelude::*;

/// BEM matrix assembly configuration
#[derive(Debug, Clone)]
pub struct AssemblyConfig {
    /// Green function method to use
    pub green_function_method: Method,
    /// Linear solver type
    pub solver_type: SolverType,
    /// Use parallel assembly
    pub parallel: bool,
    /// Integration points per panel
    pub integration_points: usize,
    /// Tolerance for singular integration
    pub singular_tolerance: f64,
}

impl Default for AssemblyConfig {
    fn default() -> Self {
        Self {
            green_function_method: Method::Delhommeau,
            solver_type: SolverType::LU,
            parallel: true,
            integration_points: 4,
            singular_tolerance: 1e-6,
        }
    }
}

/// BEM problem definition
#[derive(Debug, Clone)]
pub struct BEMProblem {
    /// Floating body with mesh
    pub body: FloatingBody,
    /// Problem type (radiation/diffraction)
    pub problem_type: ProblemType,
    /// Assembly configuration
    pub assembly_config: AssemblyConfig,
}

/// BEM result containing solution
#[derive(Debug, Clone)]
pub struct BEMResult {
    /// Solution vector (velocity potentials)
    pub potential: Vec<f64>,
    /// Added mass matrix (for radiation problems)
    pub added_mass: Option<Matrix>,
    /// Damping matrix (for radiation problems)  
    pub damping: Option<Matrix>,
    /// Wave exciting forces (for diffraction problems)
    pub excitation_force: Option<Vec<f64>>,
    /// Computation time in seconds
    pub computation_time: f64,
    /// Number of iterations (for iterative solvers)
    pub iterations: Option<usize>,
}

impl BEMResult {
    /// Get added mass matrix
    pub fn added_mass(&self) -> Option<&Matrix> {
        self.added_mass.as_ref()
    }
    
    /// Get damping matrix
    pub fn damping(&self) -> Option<&Matrix> {
        self.damping.as_ref()
    }
    
    /// Get excitation force vector
    pub fn excitation_force(&self) -> Option<&Vec<f64>> {
        self.excitation_force.as_ref()
    }
    
    /// Get exciting force vector (alias for excitation_force)
    pub fn exciting_force(&self) -> Option<&Vec<f64>> {
        self.excitation_force()
    }
    
    /// Get velocity potential solution
    pub fn potential(&self) -> &Vec<f64> {
        &self.potential
    }
    
    /// Get computation time
    pub fn computation_time(&self) -> f64 {
        self.computation_time
    }
    
    /// Get number of iterations
    pub fn iterations(&self) -> Option<usize> {
        self.iterations
    }
    
    /// Check if result contains added mass data
    pub fn has_added_mass(&self) -> bool {
        self.added_mass.is_some()
    }
    
    /// Check if result contains damping data
    pub fn has_damping(&self) -> bool {
        self.damping.is_some()
    }
    
    /// Check if result contains excitation force data
    pub fn has_excitation_force(&self) -> bool {
        self.excitation_force.is_some()
    }
}

/// BEM solver implementation
pub struct BEMSolverImpl {
    config: BEMConfig,
}

impl BEMSolverImpl {
    /// Create a new BEM solver
    pub fn new(config: BEMConfig) -> Self {
        Self { config }
    }
    
    /// Solve a BEM problem
    pub fn solve(&self, problem: &BEMProblem) -> Result<BEMResult> {
        let start_time = std::time::Instant::now();
        
        // Extract mesh from body
        let mut mesh = problem.body.mesh()?.clone();
        
        // Validate mesh
        if mesh.panels()?.is_empty() {
            return Err(BEMError::InvalidProblem {
                message: "Mesh has no panels".to_string(),
            });
        }
        
        // Set up Green function
        let green_function = self.setup_green_function(problem)?;
        
        // Assemble BEM matrix
        let bem_matrix = self.assemble_bem_matrix(&mut mesh, &green_function, &problem.assembly_config)?;
        
        // Set up right-hand side based on problem type
        let rhs = self.setup_right_hand_side(problem, &mut mesh)?;
        
        // Solve linear system
        let solver = LinearSolver::new(problem.assembly_config.solver_type);
        let potential = solver.solve(&bem_matrix, &rhs)?;
        
        // Post-process results
        let result = self.post_process_results(problem, potential, start_time.elapsed())?;
        
        Ok(result)
    }
    
    /// Set up Green function for the problem
    fn setup_green_function(&self, problem: &BEMProblem) -> Result<GreenFunction> {
        let frequency = match &problem.problem_type {
            ProblemType::Radiation { frequency, .. } => *frequency,
            ProblemType::Diffraction { frequency, .. } => *frequency,
            ProblemType::Combined { frequency, .. } => *frequency,
        };
        
        let params = GreenFunctionParams {
            method: problem.assembly_config.green_function_method,
            frequency,
            depth: f64::INFINITY, // TODO: Add depth support
            gravity: 9.81,
            ..Default::default()
        };
        
        let green_function = GreenFunction::new(params)?;
        Ok(green_function)
    }
    
    /// Assemble BEM influence matrix
    fn assemble_bem_matrix(
        &self, 
        mesh: &mut Mesh, 
        green_function: &GreenFunction,
        config: &AssemblyConfig
    ) -> Result<Matrix> {
        let panels = mesh.panels()?;
        let n_panels = panels.len();
        
        // Initialize matrix
        let mut matrix_data = vec![0.0; n_panels * n_panels];
        
        if config.parallel {
            // Parallel assembly using rayon
            let matrix_rows: Vec<Vec<f64>> = (0..n_panels)
                .into_par_iter()
                .map(|i| {
                    let mut row = vec![0.0; n_panels];
                    for j in 0..n_panels {
                        row[j] = self.compute_influence_coefficient(
                            i, j, &panels, green_function, config
                        ).unwrap_or(0.0);
                    }
                    row
                })
                .collect();
            
            // Copy results to matrix_data
            for (i, row) in matrix_rows.iter().enumerate() {
                for (j, value) in row.iter().enumerate() {
                    matrix_data[i * n_panels + j] = *value;
                }
            }
        } else {
            // Sequential assembly
            for i in 0..n_panels {
                for j in 0..n_panels {
                    matrix_data[i * n_panels + j] = self.compute_influence_coefficient(
                        i, j, &panels, green_function, config
                    )?;
                }
            }
        }
        
        Ok(Matrix::from_vec(n_panels, n_panels, matrix_data)?)
    }
    
    /// Compute influence coefficient between two panels
    fn compute_influence_coefficient(
        &self,
        source_panel: usize,
        field_panel: usize,
        panels: &[Panel],
        green_function: &GreenFunction,
        config: &AssemblyConfig,
    ) -> Result<f64> {
        let source = &panels[source_panel];
        let field = &panels[field_panel];
        
        // Get panel centroids
        let source_center = source.centroid();
        let field_center = field.centroid();
        
        if source_panel == field_panel {
            // Singular case - use specialized integration
            self.compute_singular_influence(source, green_function, config)
        } else {
            // Regular case - direct Green function evaluation
            let r1 = Point3::new(field_center.x, field_center.y, field_center.z);
            let r2 = Point3::new(source_center.x, source_center.y, source_center.z);
            
            // Convert Point3 to r,z coordinates for Green function
            let r = ((r2.x - r1.x).powi(2) + (r2.y - r1.y).powi(2)).sqrt();
            let z = r2.z - r1.z;
            
            match green_function.evaluate(r, z) {
                Ok(g_value) => {
                    // Apply panel area weighting
                    let area = source.area();
                    Ok(g_value.re * area) // Take real part for BEM matrix
                }
                Err(_) => Ok(0.0), // Handle errors gracefully
            }
        }
    }
    
    /// Compute singular influence coefficient (self-influence)
    fn compute_singular_influence(
        &self,
        panel: &Panel,
        green_function: &GreenFunction,
        config: &AssemblyConfig,
    ) -> Result<f64> {
        // For singular panels, use analytical or numerical integration
        // This is a simplified implementation - real BEM would use more sophisticated methods
        
        let area = panel.area();
        
        // Use a small offset to avoid singularity
        let offset = config.singular_tolerance;
        let normal = panel.normal();
        let center = panel.centroid();
        
        // Evaluate Green function at offset point
        let offset_point = Point3::new(
            center.x + offset * normal.x,
            center.y + offset * normal.y,
            center.z + offset * normal.z,
        );
        let center_point = Point3::new(center.x, center.y, center.z);
        
        // Convert to r,z coordinates
        let r = ((offset_point.x - center_point.x).powi(2) + (offset_point.y - center_point.y).powi(2)).sqrt();
        let z = offset_point.z - center_point.z;
        
        match green_function.evaluate(r, z) {
            Ok(g_value) => Ok(g_value.re * area),
            Err(_) => {
                // Fallback to analytical estimate for flat panels
                Ok(-area / (4.0 * std::f64::consts::PI))
            }
        }
    }
    
    /// Set up right-hand side vector based on problem type
    fn setup_right_hand_side(&self, problem: &BEMProblem, mesh: &mut Mesh) -> Result<Vec<f64>> {
        let panels = mesh.panels()?;
        let n_panels = panels.len();
        
        match &problem.problem_type {
            ProblemType::Radiation { frequency, mode } => {
                // For radiation problems, RHS depends on body motion
                self.setup_radiation_rhs(*frequency, *mode, &panels)
            }
            ProblemType::Diffraction { frequency, direction } => {
                // For diffraction problems, RHS is incident wave potential
                self.setup_diffraction_rhs(*frequency, *direction, &panels)
            }
            ProblemType::Combined { frequency, direction, modes } => {
                // For combined problems, solve for first mode (simplification)
                if let Some(&first_mode) = modes.first() {
                    self.setup_radiation_rhs(*frequency, first_mode, &panels)
                } else {
                    self.setup_diffraction_rhs(*frequency, *direction, &panels)
                }
            }
        }
    }
    
    /// Set up radiation problem right-hand side
    fn setup_radiation_rhs(&self, frequency: f64, mode: usize, panels: &[Panel]) -> Result<Vec<f64>> {
        let n_panels = panels.len();
        let mut rhs = vec![0.0; n_panels];
        
        // For radiation problems, RHS = -n · (iω ξ)
        // where n is normal vector, ω is frequency, ξ is motion amplitude
        let omega = frequency;
        
        for (i, panel) in panels.iter().enumerate() {
            let normal = panel.normal();
            
            // Unit motion in specified mode
            let motion_velocity = match mode {
                0 => nalgebra::Vector3::new(1.0, 0.0, 0.0), // Surge
                1 => nalgebra::Vector3::new(0.0, 1.0, 0.0), // Sway
                2 => nalgebra::Vector3::new(0.0, 0.0, 1.0), // Heave
                3 => {
                    // Roll - velocity depends on position
                    let center = panel.centroid();
                    nalgebra::Vector3::new(0.0, -center.z, center.y)
                }
                4 => {
                    // Pitch - velocity depends on position
                    let center = panel.centroid();
                    nalgebra::Vector3::new(center.z, 0.0, -center.x)
                }
                5 => {
                    // Yaw - velocity depends on position
                    let center = panel.centroid();
                    nalgebra::Vector3::new(-center.y, center.x, 0.0)
                }
                _ => nalgebra::Vector3::new(0.0, 0.0, 0.0), // Invalid mode
            };
            
            // RHS = -n · (iω ξ) = -ω * (n · ξ) for the imaginary part
            let normal_vec = nalgebra::Vector3::new(normal.x, normal.y, normal.z);
            rhs[i] = -omega * normal_vec.dot(&motion_velocity);
        }
        
        Ok(rhs)
    }
    
    /// Set up diffraction problem right-hand side
    fn setup_diffraction_rhs(&self, frequency: f64, direction: f64, panels: &[Panel]) -> Result<Vec<f64>> {
        let n_panels = panels.len();
        let mut rhs = vec![0.0; n_panels];
        
        // For diffraction problems, RHS = -∂φ_I/∂n
        // where φ_I is incident wave potential
        let wave_number = frequency * frequency / 9.81; // k = ω²/g
        
        for (i, panel) in panels.iter().enumerate() {
            let center = panel.centroid();
            let normal = panel.normal();
            
            // Incident wave: φ_I = (g/iω) * A * exp(ikx cos β + iky sin β + kz)
            // ∂φ_I/∂n = (g/iω) * A * ik * (n · k_vec) * exp(...)
            
            let k_x = wave_number * direction.cos();
            let k_y = wave_number * direction.sin();
            let k_z = wave_number;
            
            let wave_vector = nalgebra::Vector3::new(k_x, k_y, 0.0); // Horizontal propagation
            let normal_vec = nalgebra::Vector3::new(normal.x, normal.y, normal.z);
            
            // Simplified: assume unit amplitude and take real part
            let phase = k_x * center.x + k_y * center.y + k_z * center.z;
            let amplitude = 1.0; // Unit wave amplitude
            
            rhs[i] = -amplitude * wave_number * normal_vec.dot(&wave_vector) * phase.cos();
        }
        
        Ok(rhs)
    }
    
    /// Post-process results to extract hydrodynamic coefficients
    fn post_process_results(
        &self,
        problem: &BEMProblem,
        potential: Vec<f64>,
        computation_time: std::time::Duration,
    ) -> Result<BEMResult> {
        let mut result = BEMResult {
            potential,
            added_mass: None,
            damping: None,
            excitation_force: None,
            computation_time: computation_time.as_secs_f64(),
            iterations: None,
        };
        
        // For radiation problems, compute added mass and damping
        if let ProblemType::Radiation { frequency, mode } = &problem.problem_type {
            // This is where we would integrate pressure over body surface
            // to get added mass and damping coefficients
            // Simplified implementation:
            
            let n_dof = 6; // 6 degrees of freedom
            let mut added_mass_data = vec![0.0; n_dof * n_dof];
            let mut damping_data = vec![0.0; n_dof * n_dof];
            
            // Placeholder computation - real implementation would integrate
            // pressure = iωρφ over body surface
            for i in 0..n_dof {
                added_mass_data[i * n_dof + i] = 1000.0; // Diagonal terms
                damping_data[i * n_dof + i] = 100.0 * frequency; // Frequency dependent
            }
            
            result.added_mass = Some(Matrix::from_vec(n_dof, n_dof, added_mass_data)?);
            result.damping = Some(Matrix::from_vec(n_dof, n_dof, damping_data)?);
        }
        
        // For diffraction problems, compute wave exciting forces
        if let ProblemType::Diffraction { frequency, direction } = &problem.problem_type {
            // Compute exciting forces from pressure integration
            let mut forces = vec![0.0; 6]; // 6 DOF forces/moments
            
            // Simplified computation
            for i in 0..6 {
                forces[i] = 1000.0 * frequency.sin() * direction.cos(); // Placeholder
            }
            
            result.excitation_force = Some(forces);
        }
        
        Ok(result)
    }
} 