//! Mesh data structures and operations

use super::*;

/// Panel representation for BEM computations
#[derive(Debug, Clone)]
pub struct Panel {
    /// Panel vertices (3 points for triangular panel)
    pub vertices: [Point; 3],
    /// Panel normal vector
    pub normal: Vector,
    /// Panel centroid
    pub centroid: Point,
    /// Panel area
    pub area: f64,
}

impl Panel {
    /// Create a new panel from three vertices
    pub fn new(v0: Point, v1: Point, v2: Point) -> Result<Self> {
        let vertices = [v0, v1, v2];
        
        // Calculate normal
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(&edge2);
        let area = normal.norm() * 0.5;
        
        if area < 1e-12 {
            return Err(MeshError::InvalidData {
                message: "Degenerate panel with zero area".to_string(),
            });
        }
        
        let normal = normal.normalize();
        
        // Calculate centroid
        let centroid = Point::new(
            (v0.x + v1.x + v2.x) / 3.0,
            (v0.y + v1.y + v2.y) / 3.0,
            (v0.z + v1.z + v2.z) / 3.0,
        );
        
        Ok(Self {
            vertices,
            normal,
            centroid,
            area,
        })
    }
    
    /// Get panel centroid
    pub fn centroid(&self) -> Point {
        self.centroid
    }
    
    /// Get panel normal vector
    pub fn normal(&self) -> Vector {
        self.normal
    }
    
    /// Get panel area
    pub fn area(&self) -> f64 {
        self.area
    }
    
    /// Get panel vertices
    pub fn vertices(&self) -> &[Point; 3] {
        &self.vertices
    }
    
    /// Get panel center (same as centroid)
    pub fn center(&self) -> Point {
        self.centroid
    }
}

/// Mesh representation
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Point>,
    pub faces: Vec<[usize; 3]>,
    pub normals: Vec<Vector>,
    panels: Option<Vec<Panel>>, // Cached panels for BEM
}

impl Mesh {
    /// Create a new mesh
    pub fn new(vertices: Vec<Point>, faces: Vec<[usize; 3]>) -> Result<Self> {
        if faces.is_empty() {
            return Err(MeshError::InvalidData {
                message: "Mesh must have at least one face".to_string(),
            });
        }
        
        // Calculate normals
        let normals = Self::calculate_normals(&vertices, &faces)?;
        
        Ok(Self {
            vertices,
            faces,
            normals,
            panels: None,
        })
    }
    
    /// Get panels for BEM computation (creates and caches if needed)
    pub fn panels(&mut self) -> Result<&[Panel]> {
        if self.panels.is_none() {
            let mut panels = Vec::with_capacity(self.faces.len());
            
            for face in &self.faces {
                if face[0] >= self.vertices.len() || 
                   face[1] >= self.vertices.len() || 
                   face[2] >= self.vertices.len() {
                    return Err(MeshError::InvalidData {
                        message: "Face indices out of bounds".to_string(),
                    });
                }
                
                let v0 = self.vertices[face[0]];
                let v1 = self.vertices[face[1]];
                let v2 = self.vertices[face[2]];
                
                let panel = Panel::new(v0, v1, v2)?;
                panels.push(panel);
            }
            
            self.panels = Some(panels);
        }
        
        Ok(self.panels.as_ref().unwrap())
    }
    
    /// Get panels immutably (for read-only access)
    pub fn get_panels(&self) -> Option<&[Panel]> {
        self.panels.as_deref()
    }
    
    /// Calculate face normals
    fn calculate_normals(vertices: &[Point], faces: &[[usize; 3]]) -> Result<Vec<Vector>> {
        let mut normals = Vec::with_capacity(faces.len());
        
        for face in faces {
            if face[0] >= vertices.len() || face[1] >= vertices.len() || face[2] >= vertices.len() {
                return Err(MeshError::InvalidData {
                    message: "Face indices out of bounds".to_string(),
                });
            }
            
            let v0 = vertices[face[0]];
            let v1 = vertices[face[1]];
            let v2 = vertices[face[2]];
            
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(&edge2).normalize();
            
            normals.push(normal);
        }
        
        Ok(normals)
    }
} 