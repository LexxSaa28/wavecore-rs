//! Green function utilities

use super::*;

/// Green function utilities
pub struct GreenFunctionUtils;

impl GreenFunctionUtils {
    /// Create a new Green function utilities instance
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate distance between two points
    pub fn distance(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> f64 {
        ((x2 - x1).powi(2) + (y2 - y1).powi(2) + (z2 - z1).powi(2)).sqrt()
    }
} 