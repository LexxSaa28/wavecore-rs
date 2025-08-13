use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Reference data database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceDatabase {
    pub data: HashMap<String, ReferenceData>,
}

/// Reference data for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceData {
    pub name: String,
    pub description: String,
    pub source: String,
    pub data: HashMap<String, Vec<f64>>,
}

impl ReferenceDatabase {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn add_reference(&mut self, name: String, reference: ReferenceData) {
        self.data.insert(name, reference);
    }
    
    pub fn get_reference(&self, name: &str) -> Option<&ReferenceData> {
        self.data.get(name)
    }
} 