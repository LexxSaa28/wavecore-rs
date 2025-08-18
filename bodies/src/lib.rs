//! # WaveCore Bodies Module
//! 
//! Floating body definitions for marine hydrodynamics.
//! 
//! This module provides comprehensive floating body functionality for marine
//! hydrodynamics analysis, including body definitions, degrees of freedom,
//! mass properties, and hydrostatic calculations.
//! 
//! ## Features
//! 
//! - **Floating Body**: Complete body representation
//! - **Degrees of Freedom**: 6 DOF motion support
//! - **Mass Properties**: Mass, inertia, center of gravity
//! - **Hydrostatic Properties**: Buoyancy, stability
//! - **Body Transformations**: Position and orientation
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_bodies::{FloatingBody, DOF, MassProperties};
//! 
//! // Create mass properties
//! let mass_props = MassProperties {
//!     mass: 1000.0,
//!     center_of_gravity: [0.0, 0.0, -1.0],
//!     inertia_matrix: [[1000.0, 0.0, 0.0], [0.0, 1000.0, 0.0], [0.0, 0.0, 1000.0]],
//! };
//! 
//! // Create floating body
//! let mut body = FloatingBody::new("ship".to_string(), mass_props).unwrap();
//! 
//! // Set degrees of freedom
//! body.set_dof(DOF::Surge, true).unwrap();
//! body.set_dof(DOF::Sway, true).unwrap();
//! body.set_dof(DOF::Heave, true).unwrap();
//! 
//! println!("Body name: {}", body.name);
//! ```

pub mod floating_body;
pub mod dofs;

pub use floating_body::*;
pub use dofs::*;

use thiserror::Error;
use nalgebra::{Point3, Vector3, Matrix3};

/// Error types for body operations
#[derive(Error, Debug)]
pub enum BodyError {
    #[error("Invalid body data: {message}")]
    InvalidData { message: String },
    
    #[error("Body not found: {name}")]
    BodyNotFound { name: String },
    
    #[error("Invalid mass properties: {message}")]
    InvalidMassProperties { message: String },
    
    #[error("Invalid DOF configuration: {message}")]
    InvalidDOF { message: String },
    
    #[error("Transformation failed: {message}")]
    TransformationError { message: String },
    
    #[error("Hydrostatic calculation failed: {message}")]
    HydrostaticError { message: String },
    
    #[error("Memory allocation failed")]
    MemoryError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for body operations
pub type Result<T> = std::result::Result<T, BodyError>;

/// 3D point type
pub type Point = Point3<f64>;

/// 3D vector type
pub type Vector = Vector3<f64>;

/// Degrees of freedom for floating bodies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DOF {
    /// Surge (translation in x-direction)
    Surge,
    /// Sway (translation in y-direction)
    Sway,
    /// Heave (translation in z-direction)
    Heave,
    /// Roll (rotation around x-axis)
    Roll,
    /// Pitch (rotation around y-axis)
    Pitch,
    /// Yaw (rotation around z-axis)
    Yaw,
}

impl DOF {
    /// Get all degrees of freedom
    pub fn all() -> Vec<DOF> {
        vec![DOF::Surge, DOF::Sway, DOF::Heave, DOF::Roll, DOF::Pitch, DOF::Yaw]
    }
    
    /// Get translation DOFs
    pub fn translations() -> Vec<DOF> {
        vec![DOF::Surge, DOF::Sway, DOF::Heave]
    }
    
    /// Get rotation DOFs
    pub fn rotations() -> Vec<DOF> {
        vec![DOF::Roll, DOF::Pitch, DOF::Yaw]
    }
    
    /// Get DOF index (0-5)
    pub fn index(&self) -> usize {
        match self {
            DOF::Surge => 0,
            DOF::Sway => 1,
            DOF::Heave => 2,
            DOF::Roll => 3,
            DOF::Pitch => 4,
            DOF::Yaw => 5,
        }
    }
    
    /// Get DOF name
    pub fn name(&self) -> &'static str {
        match self {
            DOF::Surge => "Surge",
            DOF::Sway => "Sway",
            DOF::Heave => "Heave",
            DOF::Roll => "Roll",
            DOF::Pitch => "Pitch",
            DOF::Yaw => "Yaw",
        }
    }
}

/// Mass properties of a floating body
#[derive(Debug, Clone)]
pub struct MassProperties {
    /// Mass (kg)
    pub mass: f64,
    /// Center of gravity [x, y, z] (m)
    pub center_of_gravity: [f64; 3],
    /// Inertia matrix (kg⋅m²)
    pub inertia_matrix: [[f64; 3]; 3],
}

impl Default for MassProperties {
    fn default() -> Self {
        Self {
            mass: 1.0,
            center_of_gravity: [0.0, 0.0, 0.0],
            inertia_matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
}

impl MassProperties {
    /// Create new mass properties
    pub fn new(mass: f64, center_of_gravity: [f64; 3], inertia_matrix: [[f64; 3]; 3]) -> Result<Self> {
        if mass <= 0.0 {
            return Err(BodyError::InvalidMassProperties {
                message: "Mass must be positive".to_string(),
            });
        }
        
        // Check if inertia matrix is positive definite (simplified check)
        let inertia = Matrix3::from_iterator(inertia_matrix.iter().flatten().copied());
        // TODO: Implement proper positive definite check
        // For now, just check if the matrix is symmetric by comparing with its transpose
        if inertia != inertia.transpose() {
            return Err(BodyError::InvalidMassProperties {
                message: "Inertia matrix must be symmetric".to_string(),
            });
        }
        
        Ok(Self {
            mass,
            center_of_gravity,
            inertia_matrix,
        })
    }
    
    /// Get center of gravity as vector
    pub fn cog_vector(&self) -> Vector {
        Vector::new(
            self.center_of_gravity[0],
            self.center_of_gravity[1],
            self.center_of_gravity[2],
        )
    }
    
    /// Get inertia matrix as nalgebra matrix
    pub fn inertia_matrix(&self) -> Matrix3<f64> {
        Matrix3::from_iterator(self.inertia_matrix.iter().flatten().copied())
    }
}

/// Hydrostatic properties of a floating body
#[derive(Debug, Clone)]
pub struct HydrostaticProperties {
    /// Displaced volume (m³)
    pub displaced_volume: f64,
    /// Center of buoyancy [x, y, z] (m)
    pub center_of_buoyancy: [f64; 3],
    /// Waterplane area (m²)
    pub waterplane_area: f64,
    /// Metacentric height (m)
    pub metacentric_height: f64,
    /// Hydrostatic stiffness matrix
    pub hydrostatic_stiffness: [[f64; 6]; 6],
}

impl Default for HydrostaticProperties {
    fn default() -> Self {
        Self {
            displaced_volume: 0.0,
            center_of_buoyancy: [0.0, 0.0, 0.0],
            waterplane_area: 0.0,
            metacentric_height: 0.0,
            hydrostatic_stiffness: [[0.0; 6]; 6],
        }
    }
}

/// Body position and orientation
#[derive(Debug, Clone)]
pub struct BodyPose {
    /// Position [x, y, z] (m)
    pub position: [f64; 3],
    /// Orientation [roll, pitch, yaw] (radians)
    pub orientation: [f64; 3],
}

impl Default for BodyPose {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
        }
    }
}

impl BodyPose {
    /// Create new body pose
    pub fn new(position: [f64; 3], orientation: [f64; 3]) -> Self {
        Self {
            position,
            orientation,
        }
    }
    
    /// Get position as vector
    pub fn position_vector(&self) -> Vector {
        Vector::new(self.position[0], self.position[1], self.position[2])
    }
    
    /// Get orientation as vector
    pub fn orientation_vector(&self) -> Vector {
        Vector::new(self.orientation[0], self.orientation[1], self.orientation[2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dof_enum() {
        assert_eq!(DOF::Surge.index(), 0);
        assert_eq!(DOF::Sway.index(), 1);
        assert_eq!(DOF::Heave.index(), 2);
        assert_eq!(DOF::Roll.index(), 3);
        assert_eq!(DOF::Pitch.index(), 4);
        assert_eq!(DOF::Yaw.index(), 5);
        
        assert_eq!(DOF::Surge.name(), "Surge");
        assert_eq!(DOF::Sway.name(), "Sway");
        assert_eq!(DOF::Heave.name(), "Heave");
        assert_eq!(DOF::Roll.name(), "Roll");
        assert_eq!(DOF::Pitch.name(), "Pitch");
        assert_eq!(DOF::Yaw.name(), "Yaw");
        
        assert_eq!(DOF::all().len(), 6);
        assert_eq!(DOF::translations().len(), 3);
        assert_eq!(DOF::rotations().len(), 3);
    }
    
    #[test]
    fn test_mass_properties() {
        let mass_props = MassProperties::new(
            1000.0,
            [0.0, 0.0, -1.0],
            [[1000.0, 0.0, 0.0], [0.0, 1000.0, 0.0], [0.0, 0.0, 1000.0]],
        ).unwrap();
        
        assert_eq!(mass_props.mass, 1000.0);
        assert_eq!(mass_props.center_of_gravity, [0.0, 0.0, -1.0]);
    }
    
    #[test]
    fn test_mass_properties_invalid() {
        let result = MassProperties::new(
            -1.0,
            [0.0, 0.0, 0.0],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_body_pose() {
        let pose = BodyPose::new([1.0, 2.0, 3.0], [0.1, 0.2, 0.3]);
        assert_eq!(pose.position, [1.0, 2.0, 3.0]);
        assert_eq!(pose.orientation, [0.1, 0.2, 0.3]);
    }
} 