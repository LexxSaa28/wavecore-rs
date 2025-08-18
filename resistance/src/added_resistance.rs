//! Added resistance in waves calculation module
//! 
//! This module implements added resistance calculations using Response Amplitude
//! Operators (RAO) and wave spectrum integration.

use crate::{
    types::*,
    errors::{Result, ResistanceError},
};
use ndarray::{Array1, Array2};
use nalgebra as na;
use libm::{cos, sin, sqrt, pow, exp, log};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Added resistance calculator
#[derive(Debug, Clone)]
pub struct AddedResistanceCalculator {
    config: AddedResistanceConfig,
    initialized: bool,
}

/// Configuration for added resistance calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddedResistanceConfig {
    pub use_rao_integration: bool,
    pub frequency_resolution: f64,  // rad/s
    pub max_frequency: f64,         // rad/s
    pub include_short_wave_effects: bool,
    pub use_empirical_fallback: bool,
}

impl Default for AddedResistanceConfig {
    fn default() -> Self {
        Self {
            use_rao_integration: true,
            frequency_resolution: 0.05,  // 0.05 rad/s
            max_frequency: 3.0,          // 3.0 rad/s
            include_short_wave_effects: true,
            use_empirical_fallback: true,
        }
    }
}

impl AddedResistanceCalculator {
    /// Create a new added resistance calculator
    pub fn new() -> Self {
        Self::with_config(AddedResistanceConfig::default())
    }

    /// Create calculator with custom configuration
    pub fn with_config(config: AddedResistanceConfig) -> Self {
        Self {
            config,
            initialized: true,
        }
    }

    /// Check if calculator is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Calculate added resistance from RAO and wave spectrum
    pub fn calculate_from_rao(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
        wave_spectrum: &WaveSpectrum,
    ) -> Result<AddedResistanceResult> {
        info!("Calculating added resistance for {}", vessel.name);

        // Try RAO-based calculation first
        if self.config.use_rao_integration {
            match self.calculate_rao_based_resistance(vessel, conditions, wave_spectrum) {
                Ok(result) => return Ok(result),
                Err(e) if e.is_recoverable() && self.config.use_empirical_fallback => {
                    warn!("RAO calculation failed: {}. Falling back to empirical method.", e);
                    return self.calculate_empirical_resistance(vessel, conditions, wave_spectrum);
                }
                Err(e) => return Err(e),
            }
        }

        // Fallback to empirical method
        self.calculate_empirical_resistance(vessel, conditions, wave_spectrum)
    }

    /// Calculate added resistance using RAO integration
    fn calculate_rao_based_resistance(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
        wave_spectrum: &WaveSpectrum,
    ) -> Result<AddedResistanceResult> {
        // Generate or use existing RAO data
        let rao_data = self.generate_rao_data(vessel, conditions)?;

        // Perform spectrum integration
        let added_resistance = self.integrate_rao_spectrum(&rao_data, wave_spectrum)?;
        
        // Calculate mean and oscillatory components
        let mean_component = added_resistance * 0.8; // Typical split
        let oscillatory_component = added_resistance * 0.2;

        debug!("RAO-based added resistance: {:.0} N (mean: {:.0}, osc: {:.0})", 
               added_resistance, mean_component, oscillatory_component);

        Ok(AddedResistanceResult {
            total_resistance: added_resistance,
            mean_added_resistance: mean_component,
            oscillatory_component,
            rao_data: Some(rao_data),
            integration_method: "RAO-Spectrum Integration".to_string(),
        })
    }

    /// Calculate added resistance using empirical methods
    fn calculate_empirical_resistance(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
        wave_spectrum: &WaveSpectrum,
    ) -> Result<AddedResistanceResult> {
        info!("Using empirical added resistance calculation");

        // STAWAVE-2 empirical method (simplified)
        let speed_ms = conditions.speed_knots * 0.5144;
        let lbp = vessel.hull.length_between_perpendiculars;
        let beam = vessel.hull.beam;
        let cb = vessel.hull.block_coefficient;
        let displacement = conditions.displacement;

        // Wave parameters
        let hs = wave_spectrum.significant_wave_height;
        let tp = wave_spectrum.peak_period;
        let wave_direction = wave_spectrum.wave_direction;

        // Encounter angle effect
        let encounter_angle_rad = wave_direction * std::f64::consts::PI / 180.0;
        let encounter_factor = cos(encounter_angle_rad).abs();

        // Non-dimensional parameters
        let froude_number = speed_ms / sqrt(9.81 * lbp);
        let wave_steepness = 2.0 * std::f64::consts::PI * hs / (9.81 * tp.powi(2));

        // Empirical formula for added resistance
        let raw_coefficient = 4.0 * 9.81 * conditions.water_density * 
                             hs.powi(2) / lbp.powi(2) * 
                             (1.0 + 2.0 * froude_number.powi(2)) *
                             encounter_factor.powi(2) *
                             (1.0 + wave_steepness);

        // Scale by displacement and form factors
        let form_factor = 1.0 + 0.5 * cb + 0.1 * (beam / lbp);
        let added_resistance = raw_coefficient * displacement * 9.81 * form_factor;

        // Split into components
        let mean_component = added_resistance * 0.75;
        let oscillatory_component = added_resistance * 0.25;

        debug!("Empirical added resistance: {:.0} N (Fn={:.3}, encounter={:.0}°)", 
               added_resistance, froude_number, wave_direction);

        Ok(AddedResistanceResult {
            total_resistance: added_resistance,
            mean_added_resistance: mean_component,
            oscillatory_component,
            rao_data: None,
            integration_method: "Empirical (STAWAVE-2 simplified)".to_string(),
        })
    }

    /// Generate RAO data for the vessel
    fn generate_rao_data(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
    ) -> Result<RAOData> {
        // Generate frequency range
        let frequencies = Array1::range(
            0.1, 
            self.config.max_frequency, 
            self.config.frequency_resolution
        );
        let n_freq = frequencies.len();

        // Simplified RAO calculation (in practice, this would come from BEM or experiments)
        let mut surge_rao = Array1::zeros(n_freq);
        let mut heave_rao = Array1::zeros(n_freq);
        let mut pitch_rao = Array1::zeros(n_freq);
        let mut added_resistance_rao = Array1::zeros(n_freq);

        let lbp = vessel.hull.length_between_perpendiculars;
        let beam = vessel.hull.beam;
        let cb = vessel.hull.block_coefficient;
        let speed_ms = conditions.speed_knots * 0.5144;

        for (i, &omega) in frequencies.iter().enumerate() {
            // Natural frequency estimate
            let omega_n = sqrt(9.81 / lbp); // Simplified
            
            // Encounter frequency
            let _omega_e = omega - omega.powi(2) * speed_ms / 9.81;
            
            // Simplified RAO calculations
            let response_factor = 1.0 / (1.0 + (omega / omega_n).powi(4));
            
            surge_rao[i] = response_factor * 0.5;
            heave_rao[i] = response_factor * 1.0;
            pitch_rao[i] = response_factor * 0.3 * beam / lbp;
            
            // Added resistance RAO (simplified Gerritsma-Beukelman)
            let dimensionless_freq = omega.powi(2) * lbp / 9.81;
            added_resistance_rao[i] = if dimensionless_freq > 0.1 {
                8.0 * cb * conditions.water_density * 9.81 * 
                omega.powi(2) / dimensionless_freq.powi(2) * 
                response_factor
            } else {
                0.0
            };
        }

        debug!("Generated RAO data for {} frequencies", n_freq);

        Ok(RAOData {
            frequencies,
            surge_rao,
            heave_rao,
            pitch_rao,
            added_resistance_rao,
        })
    }

    /// Integrate RAO with wave spectrum
    fn integrate_rao_spectrum(
        &self,
        rao_data: &RAOData,
        wave_spectrum: &WaveSpectrum,
    ) -> Result<f64> {
        // Interpolate spectrum onto RAO frequencies if needed
        let spectrum_values = self.interpolate_spectrum_to_frequencies(
            &rao_data.frequencies,
            &wave_spectrum.frequencies,
            &wave_spectrum.spectral_densities,
        )?;

        // Numerical integration: ∫ RAO²(ω) * S(ω) dω
        let mut added_resistance = 0.0;
        for i in 0..(rao_data.frequencies.len() - 1) {
            let _omega = rao_data.frequencies[i];
            let rao_squared = rao_data.added_resistance_rao[i].powi(2);
            let spectrum_value = spectrum_values[i];
            let d_omega = rao_data.frequencies[i + 1] - rao_data.frequencies[i];
            
            added_resistance += rao_squared * spectrum_value * d_omega;
        }

        // Account for directional effects
        let directional_factor = self.calculate_directional_factor(wave_spectrum.wave_direction);
        added_resistance *= directional_factor;

        debug!("Integrated added resistance: {:.0} N", added_resistance);

        Ok(added_resistance)
    }

    /// Interpolate wave spectrum to RAO frequencies
    fn interpolate_spectrum_to_frequencies(
        &self,
        target_frequencies: &Array1<f64>,
        source_frequencies: &Array1<f64>,
        source_spectrum: &Array1<f64>,
    ) -> Result<Array1<f64>> {
        let mut interpolated = Array1::zeros(target_frequencies.len());

        for (i, &target_freq) in target_frequencies.iter().enumerate() {
            // Simple linear interpolation
            interpolated[i] = self.linear_interpolate(
                target_freq,
                source_frequencies,
                source_spectrum,
            )?;
        }

        Ok(interpolated)
    }

    /// Linear interpolation helper
    fn linear_interpolate(
        &self,
        x: f64,
        x_values: &Array1<f64>,
        y_values: &Array1<f64>,
    ) -> Result<f64> {
        if x_values.len() != y_values.len() {
            return Err(ResistanceError::calculation_error("Array length mismatch"));
        }

        // Find bounding indices
        let mut lower_idx = 0;
        let mut upper_idx = x_values.len() - 1;

        for (i, &x_val) in x_values.iter().enumerate() {
            if x_val <= x {
                lower_idx = i;
            }
            if x_val >= x && upper_idx == x_values.len() - 1 {
                upper_idx = i;
                break;
            }
        }

        if lower_idx == upper_idx {
            return Ok(y_values[lower_idx]);
        }

        // Linear interpolation
        let x1 = x_values[lower_idx];
        let x2 = x_values[upper_idx];
        let y1 = y_values[lower_idx];
        let y2 = y_values[upper_idx];

        let y = y1 + (y2 - y1) * (x - x1) / (x2 - x1);
        Ok(y)
    }

    /// Calculate directional factor for wave encounter
    fn calculate_directional_factor(&self, wave_direction: f64) -> f64 {
        let angle_rad = wave_direction * std::f64::consts::PI / 180.0;
        cos(angle_rad).abs()
    }

    /// Assess RAO data quality
    pub fn assess_rao_quality(&self, vessel: &VesselParameters) -> Result<f64> {
        // For now, return a fixed quality score
        // In practice, this would check:
        // - Frequency range coverage
        // - RAO data completeness
        // - Validation against experimental data
        // - Model applicability range

        let base_quality = 0.8;

        // Adjust based on vessel type
        let type_factor = match vessel.vessel_type.to_lowercase().as_str() {
            "container ship" => 1.0,
            "bulk carrier" => 0.95,
            "tanker" => 0.9,
            "ferry" => 0.85,
            _ => 0.7,
        };

        let quality = base_quality * type_factor;
        debug!("RAO quality assessment: {:.3}", quality);

        Ok(quality)
    }
}

impl Default for AddedResistanceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_added_resistance_calculator_creation() {
        let calculator = AddedResistanceCalculator::new();
        assert!(calculator.is_initialized());
    }

    #[test]
    fn test_empirical_resistance_calculation() {
        let calculator = AddedResistanceCalculator::new();
        let vessel = VesselParameters::default_container_ship();
        let conditions = OperatingConditions {
            speed_knots: 18.0,
            draft: 12.0,
            displacement: 52000.0,
            trim: 0.0,
            heel_angle: 0.0,
            water_density: 1025.0,
            kinematic_viscosity: 1.188e-6,
        };

        // Create test wave spectrum
        let frequencies = Array1::range(0.1, 2.0, 0.1);
        let spectral_densities = Array1::from_vec(
            frequencies.iter().map(|&f| 2.0 * (-(f as f64).powi(2))).collect()
        );

        let wave_spectrum = WaveSpectrum {
            significant_wave_height: 3.0,
            peak_period: 8.0,
            wave_direction: 45.0,
            spectrum_type: SpectrumType::JONSWAP,
            frequencies,
            spectral_densities,
        };

        let result = calculator.calculate_empirical_resistance(&vessel, &conditions, &wave_spectrum);
        assert!(result.is_ok());

        let added_resistance = result.unwrap();
        assert!(added_resistance.total_resistance > 0.0);
        assert!(added_resistance.mean_added_resistance > 0.0);
        assert!(added_resistance.oscillatory_component > 0.0);
    }

    #[test]
    fn test_rao_generation() {
        let calculator = AddedResistanceCalculator::new();
        let vessel = VesselParameters::default_container_ship();
        let conditions = OperatingConditions::default();

        let rao_result = calculator.generate_rao_data(&vessel, &conditions);
        assert!(rao_result.is_ok());

        let rao_data = rao_result.unwrap();
        assert!(rao_data.frequencies.len() > 0);
        assert_eq!(rao_data.frequencies.len(), rao_data.surge_rao.len());
        assert_eq!(rao_data.frequencies.len(), rao_data.heave_rao.len());
        assert_eq!(rao_data.frequencies.len(), rao_data.pitch_rao.len());
        assert_eq!(rao_data.frequencies.len(), rao_data.added_resistance_rao.len());
    }

    #[test]
    fn test_linear_interpolation() {
        let calculator = AddedResistanceCalculator::new();
        let x_values = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        let y_values = Array1::from_vec(vec![2.0, 4.0, 6.0, 8.0]);

        // Test exact point
        let result = calculator.linear_interpolate(2.0, &x_values, &y_values);
        assert!(result.is_ok());
        assert_relative_eq!(result.unwrap(), 4.0, epsilon = 1e-10);

        // Test interpolation point
        let result = calculator.linear_interpolate(2.5, &x_values, &y_values);
        assert!(result.is_ok());
        assert_relative_eq!(result.unwrap(), 5.0, epsilon = 1e-10);
    }

    #[test]
    fn test_directional_factor() {
        let calculator = AddedResistanceCalculator::new();

        // Head seas (0°) should give factor of 1.0
        let factor_head = calculator.calculate_directional_factor(0.0);
        assert_relative_eq!(factor_head, 1.0, epsilon = 1e-10);

        // Beam seas (90°) should give factor of 0.0
        let factor_beam = calculator.calculate_directional_factor(90.0);
        assert_relative_eq!(factor_beam, 0.0, epsilon = 1e-10);

        // 45° seas should give factor of cos(45°) ≈ 0.707
        let factor_45 = calculator.calculate_directional_factor(45.0);
        assert_relative_eq!(factor_45, 0.7071067811865476, epsilon = 1e-10);
    }

    #[test]
    fn test_rao_quality_assessment() {
        let calculator = AddedResistanceCalculator::new();
        let vessel = VesselParameters::default_container_ship();

        let quality = calculator.assess_rao_quality(&vessel);
        assert!(quality.is_ok());

        let score = quality.unwrap();
        assert!(score >= 0.0 && score <= 1.0);
        assert!(score > 0.5); // Should be reasonable for container ship
    }
} 