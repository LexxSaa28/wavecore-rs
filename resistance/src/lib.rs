//! # WaveCore Resistance Calculation Module
//! 
//! Comprehensive ship resistance calculation implementing multiple methods including:
//! - Holtrop-Mennen empirical resistance calculation
//! - Added resistance in waves via RAO integration
//! - Wind resistance and superstructure effects
//! - Calibration and validation framework
//! 
//! ## Features
//! 
//! - **Holtrop-Mennen Method**: Industry-standard empirical resistance calculation
//! - **Wave Added Resistance**: RAO-based spectral integration 
//! - **Wind Resistance**: Superstructure windage calculation
//! - **Validation Suite**: Comprehensive benchmark testing
//! - **High Performance**: Optimized mathematical computations
//! 
//! ## Example
//! 
//! ```rust
//! use wavecore_resistance::{ResistanceCalculator, VesselParameters, OperatingConditions, EnvironmentalConditions};
//! 
//! let calculator = ResistanceCalculator::new();
//! let vessel = VesselParameters::default_container_ship();
//! let conditions = OperatingConditions {
//!     speed_knots: 18.0,
//!     draft: 11.5,
//!     displacement: 52000.0,
//!     trim: 0.0,
//!     heel_angle: 0.0,
//!     water_density: 1025.0,
//!     kinematic_viscosity: 1.188e-6,
//! };
//! let environment = EnvironmentalConditions::calm_sea();
//! 
//! let result = calculator.calculate_total_resistance(&vessel, &conditions, &environment).unwrap();
//! println!("Total resistance: {:.2} kN", result.total_resistance / 1000.0);
//! ```

pub mod holtrop_mennen;
pub mod windage;
pub mod added_resistance;
pub mod validation;
pub mod types;
pub mod errors;

pub use holtrop_mennen::*;
pub use windage::*;
pub use added_resistance::*;
pub use validation::*;
pub use types::*;
pub use errors::*;

use nalgebra as na;

/// Main resistance calculator providing unified interface to all calculation methods
#[derive(Debug, Clone)]
pub struct ResistanceCalculator {
    pub holtrop_calculator: HoltropMennenCalculator,
    pub windage_calculator: WindageCalculator,
    pub added_resistance_calculator: AddedResistanceCalculator,
    pub validation_suite: ValidationSuite,
}

impl ResistanceCalculator {
    /// Create a new resistance calculator with default configuration
    pub fn new() -> Self {
        Self {
            holtrop_calculator: HoltropMennenCalculator::new(),
            windage_calculator: WindageCalculator::new(),
            added_resistance_calculator: AddedResistanceCalculator::new(),
            validation_suite: ValidationSuite::new(),
        }
    }

    /// Calculate total resistance including all components
    pub fn calculate_total_resistance(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
        environment: &EnvironmentalConditions,
    ) -> Result<TotalResistanceResult> {
        tracing::info!("Calculating total resistance for vessel: {}", vessel.name);

        // Calm water resistance (Holtrop-Mennen)
        let calm_water = self.holtrop_calculator
            .calculate_resistance(vessel, conditions)?;

        // Added resistance in waves
        let added_resistance = if environment.has_waves() {
            if let Some(ref wave_spectrum) = environment.wave_spectrum {
                self.added_resistance_calculator
                    .calculate_from_rao(vessel, conditions, wave_spectrum)?
            } else {
                AddedResistanceResult::zero()
            }
        } else {
            AddedResistanceResult::zero()
        };

        // Wind resistance
        let wind_resistance = if environment.has_wind() {
            if let Some(ref wind_conditions) = environment.wind_conditions {
                self.windage_calculator
                    .calculate_wind_resistance(vessel, wind_conditions)?
            } else {
                WindResistance::zero()
            }
        } else {
            WindResistance::zero()
        };

        // Combine all resistance components
        let total_resistance = calm_water.total_resistance 
            + added_resistance.total_resistance
            + wind_resistance.longitudinal_force;

        // Calculate required power
        let speed_ms = conditions.speed_knots * 0.5144; // knots to m/s
        let effective_power = total_resistance * speed_ms; // Watts
        let brake_power = effective_power / (vessel.propulsion.propeller_efficiency * 
                                           vessel.propulsion.shaft_efficiency *
                                           vessel.hull.hull_efficiency);

        // Confidence assessment
        let confidence = self.calculate_confidence_score(vessel, conditions, environment)?;

        Ok(TotalResistanceResult {
            total_resistance,
            breakdown: ResistanceBreakdown {
                calm_water_resistance: calm_water,
                added_resistance,
                wind_resistance,
            },
            power_requirements: PowerRequirements {
                effective_power,
                brake_power,
                propeller_efficiency: vessel.propulsion.propeller_efficiency,
            },
            confidence,
            calculation_metadata: CalculationMetadata {
                method: "Holtrop-Mennen + RAO + Windage".to_string(),
                vessel_type: vessel.vessel_type.clone(),
                conditions: conditions.clone(),
                environment: environment.clone(),
                timestamp: chrono::Utc::now(),
            },
        })
    }

    /// Calculate confidence score based on vessel parameters and conditions
    fn calculate_confidence_score(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
        environment: &EnvironmentalConditions,
    ) -> Result<f64> {
        let mut confidence_factors = Vec::new();

        // Holtrop-Mennen applicability check
        let holtrop_confidence = self.holtrop_calculator
            .assess_applicability(vessel, conditions)?;
        confidence_factors.push(holtrop_confidence);

        // RAO availability and quality
        if environment.has_waves() {
            let rao_confidence = self.added_resistance_calculator
                .assess_rao_quality(vessel)?;
            confidence_factors.push(rao_confidence);
        }

        // Wind model confidence
        if environment.has_wind() {
            let wind_confidence = self.windage_calculator
                .assess_model_confidence(vessel)?;
            confidence_factors.push(wind_confidence);
        }

        // Overall confidence is weighted average
        let total_weight: f64 = confidence_factors.len() as f64;
        let weighted_sum: f64 = confidence_factors.iter().sum();
        
        Ok(weighted_sum / total_weight)
    }

    /// Run validation benchmarks
    pub async fn run_validation_suite(&self) -> Result<ValidationReport> {
        self.validation_suite.run_all_benchmarks().await
    }
}

impl Default for ResistanceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_resistance_calculator_creation() {
        let calculator = ResistanceCalculator::new();
        assert!(calculator.holtrop_calculator.is_initialized());
        assert!(calculator.windage_calculator.is_initialized());
    }

    #[tokio::test]
    async fn test_total_resistance_calculation() {
        let calculator = ResistanceCalculator::new();
        let vessel = VesselParameters::default_container_ship();
        let conditions = OperatingConditions {
            speed_knots: 18.0,
            draft: 11.5,
            displacement: 52000.0,
            trim: 0.0,
            heel_angle: 0.0,
            water_density: 1025.0,
            kinematic_viscosity: 1.188e-6,
        };
        let environment = EnvironmentalConditions::calm_sea();

        let result = calculator.calculate_total_resistance(&vessel, &conditions, &environment);
        assert!(result.is_ok());
        
        let resistance_result = result.unwrap();
        assert!(resistance_result.total_resistance > 0.0);
        assert!(resistance_result.confidence > 0.5);
    }

    #[test]
    fn test_confidence_score_calculation() {
        let calculator = ResistanceCalculator::new();
        let vessel = VesselParameters::default_container_ship();
        let conditions = OperatingConditions::default();
        let environment = EnvironmentalConditions::calm_sea();

        let confidence = calculator.calculate_confidence_score(&vessel, &conditions, &environment);
        assert!(confidence.is_ok());
        
        let score = confidence.unwrap();
        assert!(score >= 0.0 && score <= 1.0);
    }
} 