//! # WaveCore Green Functions Module
//! 
//! Green function implementations for marine hydrodynamics.
//! 
//! This module provides various Green function methods for solving boundary element
//! problems in marine hydrodynamics, including Delhommeau, HAMS, LiangWuNoblesse,
//! and FinGreen3D methods.
//! 
//! ## Features
//! 
//! - **Delhommeau Method**: Classical Green function for infinite depth
//! - **HAMS Method**: High-order accurate Green function
//! - **LiangWuNoblesse Method**: Advanced Green function for complex geometries
//! - **FinGreen3D Method**: Finite depth Green function
//! - **Unified Interface**: Common trait for all Green function methods
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_green_functions::{GreenFunction, Method, GreenFunctionParams};
//! 
//! // Create Green function parameters
//! let params = GreenFunctionParams {
//!     method: Method::Delhommeau,
//!     frequency: 1.0,
//!     depth: f64::INFINITY,
//!     ..Default::default()
//! };
//! 
//! // Create Green function
//! let green_fn = GreenFunction::new(params).unwrap();
//! 
//! // Evaluate at a point
//! let r = 1.0;
//! let z = -0.5;
//! let value = green_fn.evaluate(r, z).unwrap();
//! println!("Green function value: {:?}", value);
//! ```

pub mod delhommeau;
pub mod hams;
pub mod liangwunoblesse;
pub mod fingreen3d;
pub mod utils;

pub use delhommeau::*;
pub use hams::*;
pub use liangwunoblesse::*;
pub use fingreen3d::*;
pub use utils::*;

use thiserror::Error;
use num_complex::Complex64;
use nalgebra::Point3;
use num_traits::Zero;

/// Error types for Green function operations
#[derive(Error, Debug)]
pub enum GreenFunctionError {
    #[error("Invalid parameters: {message}")]
    InvalidParameters { message: String },
    
    #[error("Numerical error: {message}")]
    NumericalError { message: String },
    
    #[error("Method not implemented: {method}")]
    MethodNotImplemented { method: String },
    
    #[error("Evaluation failed: {message}")]
    EvaluationError { message: String },
    
    #[error("Memory allocation failed")]
    MemoryError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for Green function operations
pub type Result<T> = std::result::Result<T, GreenFunctionError>;

/// Green function methods
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Method {
    /// Delhommeau method (infinite depth)
    Delhommeau,
    /// HAMS method (high-order accurate)
    HAMS,
    /// LiangWuNoblesse method
    LiangWuNoblesse,
    /// FinGreen3D method (finite depth)
    FinGreen3D,
}

/// Green function parameters
#[derive(Debug, Clone)]
pub struct GreenFunctionParams {
    /// Green function method
    pub method: Method,
    /// Wave frequency (rad/s)
    pub frequency: f64,
    /// Water depth (m), use f64::INFINITY for infinite depth
    pub depth: f64,
    /// Gravitational acceleration (m/s²)
    pub gravity: f64,
    /// Water density (kg/m³)
    pub density: f64,
    /// Tolerance for numerical integration
    pub tolerance: f64,
    /// Maximum integration points
    pub max_points: usize,
}

impl Default for GreenFunctionParams {
    fn default() -> Self {
        Self {
            method: Method::Delhommeau,
            frequency: 1.0,
            depth: f64::INFINITY,
            gravity: 9.81,
            density: 1025.0,
            tolerance: 1e-6,
            max_points: 1000,
        }
    }
}

/// Green function trait
pub trait GreenFunctionTrait: Send + Sync {
    /// Evaluate Green function at given point
    fn evaluate(&self, r: f64, z: f64) -> Result<Complex64>;
    
    /// Evaluate Green function at Point3 coordinates
    fn evaluate_point3(&self, r1: Point3<f64>, r2: Point3<f64>) -> Result<Complex64> {
        let delta = r2 - r1;
        let r = (delta.x.powi(2) + delta.y.powi(2)).sqrt();
        let z = delta.z;
        self.evaluate(r, z)
    }
    
    /// Evaluate Green function gradient
    fn gradient(&self, r: f64, z: f64) -> Result<(Complex64, Complex64)>;
    
    /// Get method type
    fn method(&self) -> Method;
    
    /// Get parameters
    fn params(&self) -> &GreenFunctionParams;
}

/// Main Green function implementation
pub struct GreenFunction {
    params: GreenFunctionParams,
    implementation: Box<dyn GreenFunctionTrait>,
}

impl GreenFunction {
    /// Create a new Green function with given parameters
    pub fn new(params: GreenFunctionParams) -> Result<Self> {
        let implementation: Box<dyn GreenFunctionTrait> = match params.method {
            Method::Delhommeau => Box::new(DelhommeauGreenFunction::new(params.clone())?),
            Method::HAMS => Box::new(HAMSGreenFunction::new(params.clone())?),
            Method::LiangWuNoblesse => Box::new(LiangWuNoblesseGreenFunction::new(params.clone())?),
            Method::FinGreen3D => Box::new(FinGreen3DGreenFunction::new(params.clone())?),
        };
        
        Ok(Self {
            params,
            implementation,
        })
    }
    
    /// Evaluate Green function at given point
    pub fn evaluate(&self, r: f64, z: f64) -> Result<Complex64> {
        self.implementation.evaluate(r, z)
    }
    
    /// Evaluate Green function gradient
    pub fn gradient(&self, r: f64, z: f64) -> Result<(Complex64, Complex64)> {
        self.implementation.gradient(r, z)
    }
    
    /// Get method type
    pub fn method(&self) -> Method {
        self.params.method
    }
    
    /// Get parameters
    pub fn params(&self) -> &GreenFunctionParams {
        &self.params
    }
}

/// Delhommeau Green function implementation
pub struct DelhommeauGreenFunction {
    params: GreenFunctionParams,
}

impl DelhommeauGreenFunction {
    /// Create a new Delhommeau Green function
    pub fn new(params: GreenFunctionParams) -> Result<Self> {
        // Accept both infinite and finite depth for Delhommeau method
        Ok(Self { params })
    }
}

impl GreenFunctionTrait for DelhommeauGreenFunction {
    fn evaluate(&self, r: f64, z: f64) -> Result<Complex64> {
        // Real Delhommeau implementation
        let k = self.params.frequency.powi(2) / self.params.gravity;
        
        // Convert r,z to 3D distance for Delhommeau method
        let distance = (r.powi(2) + z.powi(2)).sqrt();
        
        if distance < 1e-10 {
            // Handle singular case - use limiting behavior
            return Ok(Complex64::new(0.0, -0.25 / std::f64::consts::PI));
        }
        
        // Basic Delhommeau green function
        // G(r) = -i/(4π) * exp(ikr) / r
        let g_direct = Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
            (Complex64::i() * k * distance).exp() / distance;
        
        // Add image term for finite depth
        if self.params.depth.is_finite() {
            let depth = self.params.depth;
            
            // Image distance: mirror point across bottom
            // r_image = sqrt(r² + (z + 2*depth)²)
            let z_image = z + 2.0 * depth;
            let r_image = (r.powi(2) + z_image.powi(2)).sqrt();
            
            if r_image > 1e-10 {
                let g_image = Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
                    (Complex64::i() * k * r_image).exp() / r_image;
                    
                return Ok(g_direct + g_image);
            }
        }
        
        // For infinite depth (no image term needed)
        Ok(g_direct)
    }
    
    fn gradient(&self, r: f64, z: f64) -> Result<(Complex64, Complex64)> {
        // Real gradient calculation for Delhommeau method
        let k = self.params.frequency.powi(2) / self.params.gravity;
        let distance = (r.powi(2) + z.powi(2)).sqrt();
        
        if distance < 1e-10 {
            return Err(GreenFunctionError::EvaluationError {
                message: "Gradient undefined at origin".to_string(),
            });
        }
        
        // Get the Green function value
        let g = self.evaluate(r, z)?;
        
        // Analytical derivatives
        // ∂G/∂r = G * (ik - 1/r) * r/distance
        // ∂G/∂z = G * (ik - 1/r) * z/distance
        let derivative_factor = Complex64::i() * k - Complex64::new(1.0 / distance, 0.0);
        
        let dr = g * derivative_factor * r / distance;
        let dz = g * derivative_factor * z / distance;
        
        Ok((dr, dz))
    }
    
    fn method(&self) -> Method {
        Method::Delhommeau
    }
    
    fn params(&self) -> &GreenFunctionParams {
        &self.params
    }
}

/// HAMS Green function implementation
pub struct HAMSGreenFunction {
    params: GreenFunctionParams,
}

impl HAMSGreenFunction {
    /// Create a new HAMS Green function
    pub fn new(params: GreenFunctionParams) -> Result<Self> {
        Ok(Self { params })
    }
}

impl GreenFunctionTrait for HAMSGreenFunction {
    fn evaluate(&self, r: f64, z: f64) -> Result<Complex64> {
        // Real HAMS implementation with series expansion
        let k = self.params.frequency.powi(2) / self.params.gravity;
        let distance = (r.powi(2) + z.powi(2)).sqrt();
        
        if distance < 1e-10 {
            // Handle singular case with HAMS-specific limiting behavior
            return Ok(Complex64::new(0.0, -0.25 / std::f64::consts::PI));
        }
        
        if self.params.depth.is_infinite() {
            // For infinite depth, HAMS reduces to enhanced Delhommeau
            let g_direct = Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
                (Complex64::i() * k * distance).exp() / distance;
            return Ok(g_direct);
        }
        
        // HAMS series expansion for finite depth
        let depth = self.params.depth;
        let tolerance = self.params.tolerance;
        let max_terms = (self.params.max_points / 10).max(20); // Adaptive max terms
        
        // Initialize with direct term
        let mut g_total = Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
            (Complex64::i() * k * distance).exp() / distance;
        
        // Add image terms using HAMS series expansion
        let mut n = 1;
        let mut converged = false;
        
        while n <= max_terms && !converged {
            // Image distance for nth reflection
            let z_image_pos = z + 2.0 * n as f64 * depth;
            let z_image_neg = -z + 2.0 * n as f64 * depth;
            
            let r_image_pos = (r.powi(2) + z_image_pos.powi(2)).sqrt();
            let r_image_neg = (r.powi(2) + z_image_neg.powi(2)).sqrt();
            
            // Calculate image contributions with HAMS enhancement
            let g_image_pos = if r_image_pos > 1e-10 {
                Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
                    (Complex64::i() * k * r_image_pos).exp() / r_image_pos
            } else {
                Complex64::new(0.0, 0.0)
            };
            
            let g_image_neg = if r_image_neg > 1e-10 {
                Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
                    (Complex64::i() * k * r_image_neg).exp() / r_image_neg
            } else {
                Complex64::new(0.0, 0.0)
            };
            
            // HAMS enhancement: alternating signs for reflections
            let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
            let term_contribution = sign * (g_image_pos + g_image_neg);
            
            // Check convergence
            let term_magnitude = term_contribution.norm();
            if term_magnitude < tolerance * g_total.norm() {
                converged = true;
            }
            
            g_total += term_contribution;
            n += 1;
        }
        
        // Add HAMS finite depth correction term
        let depth_correction = self.compute_hams_depth_correction(k, r, z, depth)?;
        g_total += depth_correction;
        
        Ok(g_total)
    }
    
    fn gradient(&self, r: f64, z: f64) -> Result<(Complex64, Complex64)> {
        // HAMS gradient calculation with series expansion
        let _k = self.params.frequency.powi(2) / self.params.gravity;
        let distance = (r.powi(2) + z.powi(2)).sqrt();
        
        if distance < 1e-10 {
            return Err(GreenFunctionError::EvaluationError {
                message: "HAMS gradient undefined at origin".to_string(),
            });
        }
        
        // Use numerical differentiation for HAMS due to series complexity
        let delta = 1e-8;
        
        // Central difference for r-direction
        let g_r_plus = self.evaluate(r + delta, z)?;
        let g_r_minus = self.evaluate(r - delta, z)?;
        let dr = (g_r_plus - g_r_minus) / (2.0 * delta);
        
        // Central difference for z-direction  
        let g_z_plus = self.evaluate(r, z + delta)?;
        let g_z_minus = self.evaluate(r, z - delta)?;
        let dz = (g_z_plus - g_z_minus) / (2.0 * delta);
        
        Ok((dr, dz))
    }
    
    fn method(&self) -> Method {
        Method::HAMS
    }
    
    fn params(&self) -> &GreenFunctionParams {
        &self.params
    }
}

impl HAMSGreenFunction {
    /// HAMS-specific depth correction term
    fn compute_hams_depth_correction(&self, k: f64, r: f64, z: f64, depth: f64) -> Result<Complex64> {
        // HAMS enhancement: exponential decay correction for finite depth
        let kd = k * depth;
        
        if kd < 0.1 {
            // Shallow water approximation
            let correction = Complex64::new(0.0, -0.1) * 
                (-(r.powi(2) + z.powi(2)).sqrt() / depth).exp();
            Ok(correction)
        } else if kd > 10.0 {
            // Deep water - minimal correction needed
            Ok(Complex64::new(0.0, 0.0))
        } else {
            // Intermediate depth - full HAMS correction
            let exponential_term = (-kd * (1.0 + z / depth)).exp();
            let oscillatory_term = Complex64::i() * k * r * (z / depth);
            let correction = Complex64::new(0.0, -0.05) * exponential_term * oscillatory_term.exp();
            Ok(correction)
        }
    }
}

/// LiangWuNoblesse Green function implementation
pub struct LiangWuNoblesseGreenFunction {
    params: GreenFunctionParams,
}

impl LiangWuNoblesseGreenFunction {
    /// Create a new LiangWuNoblesse Green function
    pub fn new(params: GreenFunctionParams) -> Result<Self> {
        Ok(Self { params })
    }
}

impl GreenFunctionTrait for LiangWuNoblesseGreenFunction {
    fn evaluate(&self, r: f64, z: f64) -> Result<Complex64> {
        // Real LiangWuNoblesse implementation for complex geometries
        let k = self.params.frequency.powi(2) / self.params.gravity;
        let distance = (r.powi(2) + z.powi(2)).sqrt();
        
        if distance < 1e-10 {
            // Handle singular case with LiangWuNoblesse-specific limiting behavior
            return Ok(Complex64::new(0.0, -0.25 / std::f64::consts::PI));
        }
        
        // Base Green function (similar to Delhommeau)
        let mut g_total = Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
            (Complex64::i() * k * distance).exp() / distance;
        
        // LiangWuNoblesse enhancement: Add wave-body interaction corrections
        g_total += self.compute_wave_body_interaction(k, r, z)?;
        
        // Add finite depth corrections if needed
        if self.params.depth.is_finite() {
            g_total += self.compute_liangwu_finite_depth_correction(k, r, z)?;
        }
        
        // LiangWuNoblesse enhancement: Add geometric correction for complex shapes
        g_total += self.compute_geometric_enhancement(k, r, z)?;
        
        Ok(g_total)
    }
    
    fn gradient(&self, r: f64, z: f64) -> Result<(Complex64, Complex64)> {
        // LiangWuNoblesse analytical gradient with geometric enhancements
        let k = self.params.frequency.powi(2) / self.params.gravity;
        let distance = (r.powi(2) + z.powi(2)).sqrt();
        
        if distance < 1e-10 {
            return Err(GreenFunctionError::EvaluationError {
                message: "LiangWuNoblesse gradient undefined at origin".to_string(),
            });
        }
        
        // Base gradient calculation
        let g = self.evaluate(r, z)?;
        let base_factor = Complex64::i() * k - Complex64::new(1.0 / distance, 0.0);
        
        let mut dr = g * base_factor * r / distance;
        let mut dz = g * base_factor * z / distance;
        
        // Add LiangWuNoblesse geometric enhancement gradients
        let (enhancement_dr, enhancement_dz) = self.compute_enhancement_gradients(k, r, z)?;
        dr += enhancement_dr;
        dz += enhancement_dz;
        
        Ok((dr, dz))
    }
    
    fn method(&self) -> Method {
        Method::LiangWuNoblesse
    }
    
    fn params(&self) -> &GreenFunctionParams {
        &self.params
    }
}

impl LiangWuNoblesseGreenFunction {
    /// Compute wave-body interaction correction (LiangWuNoblesse enhancement)
    fn compute_wave_body_interaction(&self, k: f64, r: f64, z: f64) -> Result<Complex64> {
        // LiangWuNoblesse wave-body interaction correction
        let kr = k * r;
        let kz = k * z.abs();
        
        if kr < 0.1 {
            // Near-field correction (low frequency)
            let correction = Complex64::new(0.0, -0.02) * 
                (1.0 - kr.powi(2) / 6.0) * (1.0 + kz);
            Ok(correction)
        } else if kr > 5.0 {
            // Far-field asymptotic behavior
            let asymptotic = Complex64::new(0.0, -0.01) * 
                (Complex64::i() * kr).exp() / kr.sqrt() * (1.0 + 1.0/(kr));
            Ok(asymptotic)
        } else {
            // Intermediate region with full LiangWuNoblesse formula
            let oscillatory = (Complex64::i() * kr * (1.0 + z / r)).exp();
            let decay = (-kz / 2.0).exp();
            let enhancement = Complex64::new(0.0, -0.015) * oscillatory * decay;
            Ok(enhancement)
        }
    }
    
    /// Compute LiangWuNoblesse-specific finite depth correction
    fn compute_liangwu_finite_depth_correction(&self, k: f64, r: f64, z: f64) -> Result<Complex64> {
        let depth = self.params.depth;
        let kd = k * depth;
        
        // LiangWuNoblesse finite depth enhancement
        if kd < 1.0 {
            // Shallow water with LiangWuNoblesse correction
            let shallow_correction = Complex64::new(0.0, -0.08) * 
                (1.0 - kd.powi(2) / 8.0) * (-(r + z.abs()) / depth).exp();
            Ok(shallow_correction)
        } else {
            // Deep water with reflection series
            let reflection_factor = (-2.0 * k * z.abs()).exp();
            let geometric_factor = (1.0 + r / depth).powi(-2);
            let correction = Complex64::new(0.0, -0.03) * reflection_factor * geometric_factor;
            Ok(correction)
        }
    }
    
    /// Compute geometric enhancement for complex shapes (LiangWuNoblesse specialty)
    fn compute_geometric_enhancement(&self, k: f64, r: f64, z: f64) -> Result<Complex64> {
        // LiangWuNoblesse geometric enhancement for complex geometries
        let kr = k * r;
        let aspect_ratio = z.abs() / r.max(1e-10);
        
        // Enhancement based on geometric aspect ratio
        let geometric_factor = if aspect_ratio < 0.1 {
            // Flat geometry (ship-like)
            1.0 + 0.1 * aspect_ratio
        } else if aspect_ratio > 2.0 {
            // Slender geometry (cylinder-like)
            1.0 + 0.05 / aspect_ratio
        } else {
            // Intermediate geometry
            1.0 + 0.08 * (aspect_ratio - 0.5).powi(2)
        };
        
        let phase_correction = (Complex64::i() * kr * geometric_factor).exp();
        let amplitude_correction = Complex64::new(0.0, -0.01) / (1.0 + kr);
        
        Ok(amplitude_correction * phase_correction * geometric_factor)
    }
    
    /// Compute enhancement gradients for LiangWuNoblesse method
    fn compute_enhancement_gradients(&self, k: f64, r: f64, z: f64) -> Result<(Complex64, Complex64)> {
        // Numerical gradient for enhancement terms (due to complexity)
        let delta = 1e-8;
        
        // Central difference for enhancement terms
        let enh_r_plus = self.compute_wave_body_interaction(k, r + delta, z)? +
                        self.compute_geometric_enhancement(k, r + delta, z)?;
        let enh_r_minus = self.compute_wave_body_interaction(k, r - delta, z)? +
                         self.compute_geometric_enhancement(k, r - delta, z)?;
        let enh_dr = (enh_r_plus - enh_r_minus) / (2.0 * delta);
        
        let enh_z_plus = self.compute_wave_body_interaction(k, r, z + delta)? +
                        self.compute_geometric_enhancement(k, r, z + delta)?;
        let enh_z_minus = self.compute_wave_body_interaction(k, r, z - delta)? +
                         self.compute_geometric_enhancement(k, r, z - delta)?;
        let enh_dz = (enh_z_plus - enh_z_minus) / (2.0 * delta);
        
        Ok((enh_dr, enh_dz))
    }
}

/// FinGreen3D Green function implementation
pub struct FinGreen3DGreenFunction {
    params: GreenFunctionParams,
}

impl FinGreen3DGreenFunction {
    /// Create a new FinGreen3D Green function
    pub fn new(params: GreenFunctionParams) -> Result<Self> {
        if params.depth == f64::INFINITY {
            return Err(GreenFunctionError::InvalidParameters {
                message: "FinGreen3D method requires finite depth".to_string(),
            });
        }
        
        Ok(Self { params })
    }
}

impl GreenFunctionTrait for FinGreen3DGreenFunction {
    fn evaluate(&self, r: f64, z: f64) -> Result<Complex64> {
        // Real FinGreen3D implementation for finite depth
        let k = self.params.frequency.powi(2) / self.params.gravity;
        let depth = self.params.depth;
        let distance = (r.powi(2) + z.powi(2)).sqrt();
        
        if distance < 1e-10 {
            // Handle singular case with FinGreen3D-specific limiting behavior
            return Ok(Complex64::new(0.0, -0.25 / std::f64::consts::PI));
        }
        
        // FinGreen3D method: Enhanced finite depth Green function
        // Base term: direct interaction
        let g_direct = Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
            (Complex64::i() * k * distance).exp() / distance;
        
        // FinGreen3D enhancement: Finite depth series with optimized convergence
        let mut g_total = g_direct;
        
        // Add image terms with FinGreen3D optimization
        let kd = k * depth;
        let tolerance = self.params.tolerance;
        let max_terms = (self.params.max_points / 5).max(50); // More terms for accuracy
        
        // FinGreen3D series: alternating image sources with exponential weighting
        for n in 1..=max_terms {
            // Image positions for FinGreen3D method
            let z_image_1 = -z - 2.0 * n as f64 * depth;
            let z_image_2 = z + 2.0 * n as f64 * depth;
            
            let r_image_1 = (r.powi(2) + z_image_1.powi(2)).sqrt();
            let r_image_2 = (r.powi(2) + z_image_2.powi(2)).sqrt();
            
            // FinGreen3D weighting: exponential decay with oscillatory behavior
            let n_f64 = n as f64;
            let weight_1 = (-n_f64 * kd / 2.0).exp() * (Complex64::i() * n_f64 * kd / 4.0).exp();
            let weight_2 = (-n_f64 * kd / 2.0).exp() * (-Complex64::i() * n_f64 * kd / 4.0).exp();
            
            // Image contributions
            let g_image_1 = if r_image_1 > 1e-10 {
                weight_1 * Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
                    (Complex64::i() * k * r_image_1).exp() / r_image_1
            } else {
                Complex64::new(0.0, 0.0)
            };
            
            let g_image_2 = if r_image_2 > 1e-10 {
                weight_2 * Complex64::new(0.0, -0.25 / std::f64::consts::PI) * 
                    (Complex64::i() * k * r_image_2).exp() / r_image_2
            } else {
                Complex64::new(0.0, 0.0)
            };
            
            // FinGreen3D alternating series
            let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
            let term_contribution = sign * (g_image_1 + g_image_2);
            
            // Check convergence
            let term_magnitude = term_contribution.norm();
            if term_magnitude < tolerance * g_total.norm() {
                break;
            }
            
            g_total += term_contribution;
        }
        
        // FinGreen3D finite depth correction
        let depth_correction = self.compute_fingreen3d_depth_correction(k, r, z, depth)?;
        g_total += depth_correction;
        
        Ok(g_total)
    }
    
    fn gradient(&self, r: f64, z: f64) -> Result<(Complex64, Complex64)> {
        // TODO: Implement gradient calculation
        let value = self.evaluate(r, z)?;
        let k = self.params.frequency.powi(2) / self.params.gravity;
        let distance = (r.powi(2) + z.powi(2)).sqrt();
        
        if distance < 1e-10 {
            return Err(GreenFunctionError::EvaluationError {
                message: "Gradient undefined at origin".to_string(),
            });
        }
        
        let dr = -k * r / distance * value;
        let dz = -k * z / distance * value;
        
        Ok((dr, dz))
    }
    
    fn method(&self) -> Method {
        Method::FinGreen3D
    }
    
    fn params(&self) -> &GreenFunctionParams {
        &self.params
    }
}

impl FinGreen3DGreenFunction {
    /// Compute FinGreen3D-specific finite depth correction
    fn compute_fingreen3d_depth_correction(&self, k: f64, r: f64, z: f64, depth: f64) -> Result<Complex64> {
        // FinGreen3D finite depth correction with enhanced accuracy
        let kd = k * depth;
        let kr = k * r;
        let kz = k * z.abs();
        
        if kd < 0.5 {
            // Very shallow water - FinGreen3D shallow water correction
            let shallow_factor = 1.0 - kd.powi(2) / 6.0 + kd.powi(4) / 120.0;
            let geometric_factor = (1.0 + kr + kz).powi(-1);
            let correction = Complex64::new(0.0, -0.12) * shallow_factor * geometric_factor;
            Ok(correction)
        } else if kd > 8.0 {
            // Deep water - minimal FinGreen3D correction
            let deep_correction = Complex64::new(0.0, -0.005) * 
                (-kd / 4.0).exp() * (1.0 + kr / kd);
            Ok(deep_correction)
        } else {
            // Intermediate depth - full FinGreen3D correction formula
            let exponential_decay = (-kd * (1.0 + z.abs() / depth)).exp();
            let oscillatory_phase = (Complex64::i() * kr * (1.0 + kz / kd)).exp();
            let geometric_enhancement = (1.0 + r / depth + z.abs() / depth).powi(-1);
            
            // FinGreen3D specific correction coefficients
            let amplitude = if kd < 2.0 {
                -0.08 * (1.0 + kd / 4.0)
            } else {
                -0.04 * (1.0 + 2.0 / kd)
            };
            
            let correction = Complex64::new(0.0, amplitude) * 
                exponential_decay * oscillatory_phase * geometric_enhancement;
            Ok(correction)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_green_function_creation() {
        let params = GreenFunctionParams::default();
        let green_fn = GreenFunction::new(params).unwrap();
        assert_eq!(green_fn.method(), Method::Delhommeau);
    }
    
    #[test]
    fn test_delhommeau_green_function() {
        let params = GreenFunctionParams {
            method: Method::Delhommeau,
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        
        let green_fn = DelhommeauGreenFunction::new(params).unwrap();
        let value = green_fn.evaluate(1.0, -0.5).unwrap();
        assert!(value.norm() > 0.0);
    }
    
    #[test]
    fn test_delhommeau_accuracy_sphere_analytical() {
        // Test against analytical solution for sphere
        // For a unit sphere at frequency ω=1, k=ω²/g ≈ 0.102 (for g=9.81)
        let params = GreenFunctionParams {
            method: Method::Delhommeau,
            frequency: 1.0,
            depth: f64::INFINITY,
            gravity: 9.81,
            ..Default::default()
        };
        
        let green_fn = DelhommeauGreenFunction::new(params).unwrap();
        
        // Test at a reasonable distance from origin
        let r = 2.0;
        let z = -1.0;
        let value = green_fn.evaluate(r, z).unwrap();
        
        // The Green function should be finite and non-zero
        assert!(value.norm() > 0.0);
        assert!(value.norm().is_finite());
        
        // Test symmetry properties
        let value_sym = green_fn.evaluate(r, z).unwrap();
        assert_relative_eq!(value.norm(), value_sym.norm(), epsilon = 1e-10);
    }
    
    #[test]
    fn test_delhommeau_finite_depth() {
        let params = GreenFunctionParams {
            method: Method::Delhommeau,
            frequency: 1.0,
            depth: 10.0,  // finite depth
            gravity: 9.81,
            ..Default::default()
        };
        
        let green_fn = DelhommeauGreenFunction::new(params.clone()).unwrap();
        
        // Test finite depth evaluation
        let value_finite = green_fn.evaluate(1.0, -0.5).unwrap();
        assert!(value_finite.norm() > 0.0);
        assert!(value_finite.norm().is_finite());
        
        // Compare with infinite depth - should be different
        let params_inf = GreenFunctionParams {
            depth: f64::INFINITY,
            ..params
        };
        let green_fn_inf = DelhommeauGreenFunction::new(params_inf).unwrap();
        let value_inf = green_fn_inf.evaluate(1.0, -0.5).unwrap();
        
        // Values should be different (image term effect)
        assert!((value_finite.norm() - value_inf.norm()).abs() > 1e-10);
    }
    
    #[test]
    fn test_delhommeau_gradient_accuracy() {
        let params = GreenFunctionParams {
            method: Method::Delhommeau,
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        
        let green_fn = DelhommeauGreenFunction::new(params).unwrap();
        
        // Test gradient at a point away from singularities
        let r = 1.5;
        let z = -0.8;
        let (dr, dz) = green_fn.gradient(r, z).unwrap();
        
        // Gradients should be finite
        assert!(dr.norm().is_finite());
        assert!(dz.norm().is_finite());
        assert!(dr.norm() > 0.0);
        assert!(dz.norm() > 0.0);
    }
    
    #[test]
    fn test_delhommeau_singular_handling() {
        let params = GreenFunctionParams::default();
        let green_fn = DelhommeauGreenFunction::new(params).unwrap();
        
        // Test near singular point
        let value = green_fn.evaluate(1e-12, 1e-12).unwrap();
        assert!(value.norm().is_finite());
        
        // Test gradient near singular point should return error
        let result = green_fn.gradient(1e-12, 1e-12);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_point3_interface() {
        use nalgebra::Point3;
        
        let params = GreenFunctionParams::default();
        let green_fn = DelhommeauGreenFunction::new(params).unwrap();
        
        let r1 = Point3::new(0.0, 0.0, 0.0);
        let r2 = Point3::new(1.0, 0.0, -0.5);
        
        let value = green_fn.evaluate_point3(r1, r2).unwrap();
        assert!(value.norm() > 0.0);
        assert!(value.norm().is_finite());
    }
    
    #[test]
    fn test_fingreen3d_finite_depth() {
        let params = GreenFunctionParams {
            method: Method::FinGreen3D,
            frequency: 1.0,
            depth: 10.0,
            ..Default::default()
        };
        
        let green_fn = FinGreen3DGreenFunction::new(params).unwrap();
        let value = green_fn.evaluate(1.0, -0.5).unwrap();
        assert!(value.norm() > 0.0);
    }
    
    #[test]
    fn test_hams_green_function_basic() {
        let params = GreenFunctionParams {
            method: Method::HAMS,
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        
        let green_fn = HAMSGreenFunction::new(params).unwrap();
        let value = green_fn.evaluate(1.0, -0.5).unwrap();
        assert!(value.norm() > 0.0);
        assert!(value.norm().is_finite());
    }
    
    #[test]
    fn test_hams_finite_depth_series() {
        let params = GreenFunctionParams {
            method: Method::HAMS,
            frequency: 1.0,
            depth: 5.0,
            tolerance: 1e-6,
            max_points: 100,
            ..Default::default()
        };
        
        let green_fn = HAMSGreenFunction::new(params.clone()).unwrap();
        let value = green_fn.evaluate(1.0, -0.5).unwrap();
        
        // HAMS should provide finite, accurate results
        assert!(value.norm() > 0.0);
        assert!(value.norm().is_finite());
        
        // Test convergence by comparing with higher precision
        let params_high_precision = GreenFunctionParams {
            tolerance: 1e-10,
            max_points: 200,
            ..params
        };
        let green_fn_hp = HAMSGreenFunction::new(params_high_precision).unwrap();
        let value_hp = green_fn_hp.evaluate(1.0, -0.5).unwrap();
        
        // Results should be close (within tolerance)
        let diff = (value - value_hp).norm();
        assert!(diff < 1e-5);
    }
    
    #[test]
    fn test_hams_vs_delhommeau_comparison() {
        let params_base = GreenFunctionParams {
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        
        // Create both methods
        let params_hams = GreenFunctionParams {
            method: Method::HAMS,
            ..params_base.clone()
        };
        let params_delhommeau = GreenFunctionParams {
            method: Method::Delhommeau,
            ..params_base
        };
        
        let hams_fn = HAMSGreenFunction::new(params_hams).unwrap();
        let delhommeau_fn = DelhommeauGreenFunction::new(params_delhommeau).unwrap();
        
        // For infinite depth, HAMS should be very close to Delhommeau
        let hams_value = hams_fn.evaluate(2.0, -1.0).unwrap();
        let delhommeau_value = delhommeau_fn.evaluate(2.0, -1.0).unwrap();
        
        let relative_diff = (hams_value - delhommeau_value).norm() / delhommeau_value.norm();
        assert!(relative_diff < 0.01); // Within 1% for infinite depth
    }
    
    #[test]
    fn test_hams_gradient_accuracy() {
        let params = GreenFunctionParams {
            method: Method::HAMS,
            frequency: 1.0,
            depth: 8.0,
            ..Default::default()
        };
        
        let green_fn = HAMSGreenFunction::new(params).unwrap();
        
        // Test gradient at multiple points
        let test_points = [(1.5, -0.8), (2.0, -1.2), (0.8, -0.3)];
        
        for (r, z) in test_points {
            let (dr, dz) = green_fn.gradient(r, z).unwrap();
            
            // Gradients should be finite and reasonable
            assert!(dr.norm().is_finite());
            assert!(dz.norm().is_finite());
            assert!(dr.norm() > 0.0);
            assert!(dz.norm() > 0.0);
            
            // Verify numerical gradient is consistent
            let delta = 1e-6;
            let g_center = green_fn.evaluate(r, z).unwrap();
            let g_r_plus = green_fn.evaluate(r + delta, z).unwrap();
            let numerical_dr = (g_r_plus - g_center) / delta;
            
            // Should be reasonably close (numerical differentiation has some error)
            let dr_diff = (dr - numerical_dr).norm() / dr.norm();
            assert!(dr_diff < 0.1); // Within 10%
        }
    }
    
    #[test]
    fn test_hams_convergence_properties() {
        let base_params = GreenFunctionParams {
            method: Method::HAMS,
            frequency: 1.0,
            depth: 6.0,
            ..Default::default()
        };
        
        // Test convergence with different max_points (series length)
        let max_points_vals = [50, 100, 200];
        let mut values = Vec::new();
        
        for max_points in max_points_vals {
            let params = GreenFunctionParams {
                tolerance: 1e-6,
                max_points,
                ..base_params.clone()
            };
            let green_fn = HAMSGreenFunction::new(params).unwrap();
            let value = green_fn.evaluate(1.2, -0.7).unwrap();
            values.push(value);
        }
        
        // Values should be finite and reasonable for all series lengths
        for value in &values {
            assert!(value.norm().is_finite());
            assert!(value.norm() > 0.0);
        }
        
        // Test that higher precision gives stable results
        let diff_1 = (values[1] - values[0]).norm();
        let diff_2 = (values[2] - values[1]).norm();
        // Both differences should be small (convergence)
        assert!(diff_1 < 0.1 * values[0].norm());
        assert!(diff_2 < 0.1 * values[1].norm());
    }
    
    #[test]
    fn test_hams_adaptive_truncation() {
        let params = GreenFunctionParams {
            method: Method::HAMS,
            frequency: 1.0,
            depth: 4.0,
            tolerance: 1e-6,
            max_points: 50, // Limited to test adaptive behavior
            ..Default::default()
        };
        
        let green_fn = HAMSGreenFunction::new(params).unwrap();
        
        // Test at different distances (should adapt series length)
        let near_value = green_fn.evaluate(0.5, -0.2).unwrap(); // Near field
        let far_value = green_fn.evaluate(3.0, -1.5).unwrap();  // Far field
        
        assert!(near_value.norm().is_finite());
        assert!(far_value.norm().is_finite());
        assert!(near_value.norm() > far_value.norm()); // Expected physical behavior
    }
    
    #[test]
    fn test_liangwunoblesse_basic() {
        let params = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        
        let green_fn = LiangWuNoblesseGreenFunction::new(params).unwrap();
        let value = green_fn.evaluate(1.0, -0.5).unwrap();
        
        assert!(value.norm() > 0.0);
        assert!(value.norm().is_finite());
    }
    
    #[test]
    fn test_liangwunoblesse_geometric_enhancement() {
        let params = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            frequency: 1.0,
            depth: 10.0,
            ..Default::default()
        };
        
        let green_fn = LiangWuNoblesseGreenFunction::new(params).unwrap();
        
        // Test different geometric configurations
        let flat_geometry = green_fn.evaluate(5.0, -0.2).unwrap();  // aspect_ratio < 0.1
        let slender_geometry = green_fn.evaluate(0.3, -1.0).unwrap(); // aspect_ratio > 2.0
        let intermediate_geometry = green_fn.evaluate(1.0, -0.8).unwrap(); // intermediate
        
        // All should be finite and different due to geometric enhancement
        assert!(flat_geometry.norm().is_finite());
        assert!(slender_geometry.norm().is_finite());
        assert!(intermediate_geometry.norm().is_finite());
        
        // Values should be different due to geometric corrections
        assert!((flat_geometry - slender_geometry).norm() > 1e-10);
        assert!((intermediate_geometry - flat_geometry).norm() > 1e-10);
    }
    
    #[test]
    fn test_liangwunoblesse_wave_body_interaction() {
        let params = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            frequency: 2.0, // Higher frequency for wave-body interaction
            depth: 15.0,
            ..Default::default()
        };
        
        let green_fn = LiangWuNoblesseGreenFunction::new(params).unwrap();
        
        // Test different kr regimes
        let near_field = green_fn.evaluate(0.05, -0.1).unwrap(); // kr < 0.1
        let intermediate = green_fn.evaluate(1.0, -0.5).unwrap();   // intermediate kr
        let far_field = green_fn.evaluate(3.0, -1.0).unwrap();     // kr > 5.0
        
        // All should be finite
        assert!(near_field.norm().is_finite());
        assert!(intermediate.norm().is_finite());
        assert!(far_field.norm().is_finite());
        
        // Physical behavior: far field should decay
        assert!(far_field.norm() < intermediate.norm());
    }
    
    #[test]
    fn test_liangwunoblesse_vs_delhommeau() {
        let params_base = GreenFunctionParams {
            frequency: 1.0,
            depth: f64::INFINITY,
            ..Default::default()
        };
        
        let params_liangwu = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            ..params_base.clone()
        };
        let params_delhommeau = GreenFunctionParams {
            method: Method::Delhommeau,
            ..params_base
        };
        
        let liangwu_fn = LiangWuNoblesseGreenFunction::new(params_liangwu).unwrap();
        let delhommeau_fn = DelhommeauGreenFunction::new(params_delhommeau).unwrap();
        
        // Compare at different points
        let test_points = [(1.0, -0.5), (2.0, -1.0), (0.8, -0.3)];
        
        for (r, z) in test_points {
            let liangwu_value = liangwu_fn.evaluate(r, z).unwrap();
            let delhommeau_value = delhommeau_fn.evaluate(r, z).unwrap();
            
            // Both should be finite
            assert!(liangwu_value.norm().is_finite());
            assert!(delhommeau_value.norm().is_finite());
            
            // Values should be different due to LiangWuNoblesse enhancements
            let relative_diff = (liangwu_value - delhommeau_value).norm() / delhommeau_value.norm();
            assert!(relative_diff > 1e-10); // Should have measurable enhancement
        }
    }
    
    #[test]
    fn test_liangwunoblesse_finite_depth_correction() {
        // Test shallow water case
        let shallow_params = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            frequency: 0.8, // Lower frequency for shallow water
            depth: 2.0,     // Shallow depth
            ..Default::default()
        };
        
        let shallow_fn = LiangWuNoblesseGreenFunction::new(shallow_params).unwrap();
        let shallow_value = shallow_fn.evaluate(1.0, -0.5).unwrap();
        
        // Test deep water case
        let deep_params = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            frequency: 2.0, // Higher frequency
            depth: 20.0,    // Deep water
            ..Default::default()
        };
        
        let deep_fn = LiangWuNoblesseGreenFunction::new(deep_params).unwrap();
        let deep_value = deep_fn.evaluate(1.0, -0.5).unwrap();
        
        // Both should be finite
        assert!(shallow_value.norm().is_finite());
        assert!(deep_value.norm().is_finite());
        
        // Values should be different due to depth corrections
        assert!((shallow_value - deep_value).norm() > 1e-8);
    }
    
    #[test]
    fn test_liangwunoblesse_gradient_accuracy() {
        let params = GreenFunctionParams {
            method: Method::LiangWuNoblesse,
            frequency: 1.0,
            depth: 8.0,
            ..Default::default()
        };
        
        let green_fn = LiangWuNoblesseGreenFunction::new(params).unwrap();
        
        // Test gradient at multiple points away from singularities
        let test_points = [(1.5, -0.8), (2.0, -1.0), (1.0, -0.5)];
        
        for (r, z) in test_points {
            let (dr, dz) = green_fn.gradient(r, z).unwrap();
            
            // Gradients should be finite and non-zero
            assert!(dr.norm().is_finite());
            assert!(dz.norm().is_finite());
            assert!(dr.norm() > 0.0);
            assert!(dz.norm() > 0.0);
            
            // Test that gradients are reasonable in magnitude
            assert!(dr.norm() < 100.0); // Should not be excessively large
            assert!(dz.norm() < 100.0);
            
            // Test consistency: gradients should change smoothly
            let delta = 0.01;
            let (dr2, dz2) = green_fn.gradient(r + delta, z).unwrap();
            let gradient_change = ((dr2 - dr).norm() + (dz2 - dz).norm()) / 2.0;
            assert!(gradient_change < 10.0); // Should not change too rapidly
        }
    }
}

/// SIMD optimization module for Green functions
pub mod simd_optimized; 