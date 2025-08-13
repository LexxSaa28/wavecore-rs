//! # WaveCore Meshes Module
//! 
//! Mesh operations for marine hydrodynamics.
//! 
//! This module provides comprehensive mesh functionality for boundary element method
//! computations, including mesh data structures, operations, collections, and
//! predefined marine geometries.
//! 
//! ## Features
//! 
//! - **Mesh Data Structures**: Efficient mesh representation
//! - **Mesh Operations**: Transformations, validation, optimization
//! - **Mesh Collections**: Multiple mesh management
//! - **Predefined Geometries**: Sphere, cylinder, ship hulls
//! - **Quality Checks**: Mesh validation and optimization
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_meshes::{Mesh, MeshCollection, PredefinedGeometry};
//! 
//! // Create a sphere mesh
//! let sphere = PredefinedGeometry::sphere(1.0, 32, 16)?;
//! 
//! // Create mesh collection
//! let mut collection = MeshCollection::new();
//! collection.add_mesh("sphere", sphere)?;
//! 
//! // Validate mesh quality
//! let quality = collection.validate("sphere")?;
//! println!("Mesh quality: {:?}", quality);
//! ```

pub mod mesh;
pub mod collections;
pub mod predefined;
pub mod refinement;
pub mod quality;

pub use mesh::*;
pub use collections::*;
pub use predefined::*;

use thiserror::Error;
use nalgebra::{Point3, Vector3};

/// Error types for mesh operations
#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Invalid mesh data: {message}")]
    InvalidData { message: String },
    
    #[error("Mesh validation failed: {message}")]
    ValidationError { message: String },
    
    #[error("Mesh not found: {name}")]
    MeshNotFound { name: String },
    
    #[error("Invalid geometry parameters: {message}")]
    InvalidGeometry { message: String },
    
    #[error("Transformation failed: {message}")]
    TransformationError { message: String },
    
    #[error("Memory allocation failed")]
    MemoryError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for mesh operations
pub type Result<T> = std::result::Result<T, MeshError>;

/// 3D point type
pub type Point = Point3<f64>;

/// 3D vector type
pub type Vector = Vector3<f64>;

/// Mesh quality metrics
#[derive(Debug, Clone)]
pub struct MeshQuality {
    /// Minimum element quality (0-1)
    pub min_quality: f64,
    /// Maximum element quality (0-1)
    pub max_quality: f64,
    /// Average element quality (0-1)
    pub average_quality: f64,
    /// Number of degenerate elements
    pub degenerate_elements: usize,
    /// Number of inverted elements
    pub inverted_elements: usize,
    /// Total number of elements
    pub total_elements: usize,
}

impl Default for MeshQuality {
    fn default() -> Self {
        Self {
            min_quality: 1.0,
            max_quality: 1.0,
            average_quality: 1.0,
            degenerate_elements: 0,
            inverted_elements: 0,
            total_elements: 0,
        }
    }
}

/// Predefined geometry types
#[derive(Debug, Clone, Copy)]
pub enum GeometryType {
    /// Sphere geometry
    Sphere,
    /// Cylinder geometry
    Cylinder,
    /// Box geometry
    Box,
    /// Ship hull geometry
    ShipHull,
    /// Custom geometry
    Custom,
}

/// Mesh transformation types
#[derive(Debug, Clone)]
pub enum Transformation {
    /// Translation by vector
    Translation(Vector),
    /// Rotation around axis
    Rotation { axis: Vector, angle: f64 },
    /// Scaling by factors
    Scaling { x: f64, y: f64, z: f64 },
    /// Combined transformation
    Combined(Vec<Transformation>),
}

/// Mesh statistics
#[derive(Debug, Clone)]
pub struct MeshStats {
    /// Number of vertices
    pub vertices: usize,
    /// Number of faces
    pub faces: usize,
    /// Number of edges
    pub edges: usize,
    /// Bounding box
    pub bounding_box: (Point, Point),
    /// Surface area
    pub surface_area: f64,
    /// Volume
    pub volume: f64,
}

impl Default for MeshStats {
    fn default() -> Self {
        Self {
            vertices: 0,
            faces: 0,
            edges: 0,
            bounding_box: (Point::origin(), Point::origin()),
            surface_area: 0.0,
            volume: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mesh_quality_default() {
        let quality = MeshQuality::default();
        assert_eq!(quality.min_quality, 1.0);
        assert_eq!(quality.max_quality, 1.0);
        assert_eq!(quality.average_quality, 1.0);
        assert_eq!(quality.degenerate_elements, 0);
        assert_eq!(quality.inverted_elements, 0);
        assert_eq!(quality.total_elements, 0);
    }
    
    #[test]
    fn test_mesh_stats_default() {
        let stats = MeshStats::default();
        assert_eq!(stats.vertices, 0);
        assert_eq!(stats.faces, 0);
        assert_eq!(stats.edges, 0);
        assert_eq!(stats.surface_area, 0.0);
        assert_eq!(stats.volume, 0.0);
    }
    
    #[test]
    fn test_transformation_types() {
        let translation = Transformation::Translation(Vector::new(1.0, 2.0, 3.0));
        let rotation = Transformation::Rotation {
            axis: Vector::new(0.0, 0.0, 1.0),
            angle: std::f64::consts::PI / 2.0,
        };
        let scaling = Transformation::Scaling { x: 2.0, y: 2.0, z: 2.0 };
        
        // Just test that they can be created
        assert!(matches!(translation, Transformation::Translation(_)));
        assert!(matches!(rotation, Transformation::Rotation { .. }));
        assert!(matches!(scaling, Transformation::Scaling { .. }));
    }
} 