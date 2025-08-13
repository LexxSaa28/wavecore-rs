//! Degrees of freedom definitions

use super::*;

/// Degrees of freedom manager
pub struct DOFManager {
    dofs: std::collections::HashMap<DOF, bool>,
}

impl DOFManager {
    /// Create a new DOF manager
    pub fn new() -> Self {
        Self {
            dofs: std::collections::HashMap::new(),
        }
    }
    
    /// Enable all DOFs
    pub fn enable_all(&mut self) {
        for dof in DOF::all() {
            self.dofs.insert(dof, true);
        }
    }
    
    /// Disable all DOFs
    pub fn disable_all(&mut self) {
        for dof in DOF::all() {
            self.dofs.insert(dof, false);
        }
    }
    
    /// Set DOF state
    pub fn set_dof(&mut self, dof: DOF, enabled: bool) {
        self.dofs.insert(dof, enabled);
    }
    
    /// Get DOF state
    pub fn get_dof(&self, dof: &DOF) -> bool {
        *self.dofs.get(dof).unwrap_or(&false)
    }
    
    /// Get enabled DOFs
    pub fn enabled_dofs(&self) -> Vec<DOF> {
        self.dofs.iter()
            .filter(|(_, &enabled)| enabled)
            .map(|(dof, _)| *dof)
            .collect()
    }
} 