//! Airy wave theory implementation

use super::*;

/// Airy wave parameters
pub struct AiryWaveParams {
    pub amplitude: f64,
    pub frequency: f64,
    pub wave_number: f64,
    pub phase: f64,
}

impl AiryWaveParams {
    /// Create new Airy wave parameters
    pub fn new(amplitude: f64, frequency: f64, wave_number: f64) -> Self {
        Self {
            amplitude,
            frequency,
            wave_number,
            phase: 0.0,
        }
    }
}

/// Airy wave theory implementation
pub struct AiryWaveTheory;

impl AiryWaveTheory {
    /// Calculate wave elevation at given position and time
    pub fn elevation(&self, params: &AiryWaveParams, x: f64, y: f64, t: f64) -> f64 {
        params.amplitude * (params.wave_number * x - params.frequency * t + params.phase).cos()
    }
} 