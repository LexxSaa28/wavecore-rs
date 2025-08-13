//! Predefined mesh geometries

use super::*;

/// Predefined geometry generator
pub struct PredefinedGeometry;

impl PredefinedGeometry {
    /// Create a sphere mesh
    pub fn sphere(radius: f64, num_phi: usize, num_theta: usize) -> Result<Mesh> {
        if num_phi < 3 || num_theta < 2 {
            return Err(MeshError::InvalidGeometry {
                message: "Sphere requires at least 3 phi divisions and 2 theta divisions".to_string(),
            });
        }
        
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        
        // Generate vertices
        for i in 0..=num_phi {
            let phi = 2.0 * std::f64::consts::PI * i as f64 / num_phi as f64;
            for j in 0..=num_theta {
                let theta = std::f64::consts::PI * j as f64 / num_theta as f64;
                
                let x = radius * theta.sin() * phi.cos();
                let y = radius * theta.sin() * phi.sin();
                let z = radius * theta.cos();
                
                vertices.push(Point::new(x, y, z));
            }
        }
        
        // Generate faces
        for i in 0..num_phi {
            for j in 0..num_theta {
                let v0 = i * (num_theta + 1) + j;
                let v1 = (i + 1) * (num_theta + 1) + j;
                let v2 = (i + 1) * (num_theta + 1) + j + 1;
                let v3 = i * (num_theta + 1) + j + 1;
                
                faces.push([v0, v1, v2]);
                faces.push([v0, v2, v3]);
            }
        }
        
        Mesh::new(vertices, faces)
    }
} 