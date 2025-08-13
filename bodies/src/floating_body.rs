//! Floating body definitions

use super::*;
use wavecore_meshes::Mesh;

/// Floating body representation
#[derive(Debug, Clone)]
pub struct FloatingBody {
    pub name: String,
    pub mass_properties: MassProperties,
    pub hydrostatic_properties: HydrostaticProperties,
    pub pose: BodyPose,
    pub dofs: std::collections::HashMap<DOF, bool>,
    pub mesh: Option<Mesh>,
}

impl FloatingBody {
    /// Create a new floating body
    pub fn new(name: String, mass_properties: MassProperties) -> Result<Self> {
        Ok(Self {
            name,
            mass_properties,
            hydrostatic_properties: HydrostaticProperties::default(),
            pose: BodyPose::default(),
            dofs: std::collections::HashMap::new(),
            mesh: None,
        })
    }
    
    /// Create a new floating body with mesh
    pub fn with_mesh(name: String, mass_properties: MassProperties, mesh: Mesh) -> Result<Self> {
        Ok(Self {
            name,
            mass_properties,
            hydrostatic_properties: HydrostaticProperties::default(),
            pose: BodyPose::default(),
            dofs: std::collections::HashMap::new(),
            mesh: Some(mesh),
        })
    }
    
    /// Set degree of freedom
    pub fn set_dof(&mut self, dof: DOF, enabled: bool) -> Result<()> {
        self.dofs.insert(dof, enabled);
        Ok(())
    }
    
    /// Check if degree of freedom is enabled
    pub fn is_dof_enabled(&self, dof: &DOF) -> bool {
        *self.dofs.get(dof).unwrap_or(&false)
    }
    
    /// Get mesh reference
    pub fn mesh(&self) -> Result<&Mesh> {
        self.mesh.as_ref().ok_or_else(|| BodyError::InvalidData {
            message: "Body has no mesh attached".to_string(),
        })
    }
    
    /// Get mutable mesh reference
    pub fn mesh_mut(&mut self) -> Result<&mut Mesh> {
        self.mesh.as_mut().ok_or_else(|| BodyError::InvalidData {
            message: "Body has no mesh attached".to_string(),
        })
    }
    
    /// Set mesh
    pub fn set_mesh(&mut self, mesh: Mesh) {
        self.mesh = Some(mesh);
    }
    
    /// Remove mesh
    pub fn remove_mesh(&mut self) {
        self.mesh = None;
    }
    
    /// Check if body has mesh
    pub fn has_mesh(&self) -> bool {
        self.mesh.is_some()
    }
} 