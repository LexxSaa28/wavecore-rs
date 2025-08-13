//! Mesh collections

use super::*;
use std::collections::HashMap;

/// Mesh collection
pub struct MeshCollection {
    meshes: HashMap<String, Mesh>,
}

impl MeshCollection {
    /// Create a new mesh collection
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
        }
    }
    
    /// Add a mesh to the collection
    pub fn add_mesh(&mut self, name: String, mesh: Mesh) -> Result<()> {
        self.meshes.insert(name, mesh);
        Ok(())
    }
    
    /// Get a mesh from the collection
    pub fn get_mesh(&self, name: &str) -> Result<&Mesh> {
        self.meshes.get(name).ok_or_else(|| MeshError::MeshNotFound {
            name: name.to_string(),
        })
    }
    
    /// Remove a mesh from the collection
    pub fn remove_mesh(&mut self, name: &str) -> Result<()> {
        self.meshes.remove(name).ok_or_else(|| MeshError::MeshNotFound {
            name: name.to_string(),
        })?;
        Ok(())
    }
    
    /// Get all mesh names
    pub fn mesh_names(&self) -> Vec<String> {
        self.meshes.keys().cloned().collect()
    }
} 