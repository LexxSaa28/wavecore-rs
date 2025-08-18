//! Ship example

use anyhow::Result;
use log::info;

/// Ship example
pub struct ShipExample;

impl ShipExample {
    /// Create a new ship example
    pub fn new() -> Self {
        Self
    }
    
    /// Run the ship example
    pub async fn run(&self) -> Result<()> {
        info!("Running ship example...");
        // TODO: Implement ship example
        Ok(())
    }
} 