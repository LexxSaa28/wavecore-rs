//! Holtrop-Mennen resistance calculation implementation
//! 
//! This module implements the industry-standard Holtrop-Mennen method for 
//! calculating ship resistance in calm water. The method is applicable to
//! displacement vessels and provides good accuracy for conventional hull forms.
//!
//! ## References
//! - Holtrop, J. and Mennen, G.G.J. (1982). "An Approximate Power Prediction Method"
//! - International Shipbuilding Progress, Vol. 29, No. 335

use crate::{
    types::*,
    errors::{Result, ResistanceError},
};
use libm::{cos, sin, tan, atan, sqrt, pow, exp, log};
use nalgebra as na;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Holtrop-Mennen resistance calculator
#[derive(Debug, Clone)]
pub struct HoltropMennenCalculator {
    config: HoltropMennenConfig,
    initialized: bool,
}

/// Configuration for Holtrop-Mennen calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoltropMennenConfig {
    pub enforce_validity_range: bool,
    pub minimum_confidence_threshold: f64,
    pub enable_form_factor_correction: bool,
    pub use_improved_appendage_resistance: bool,
}

impl Default for HoltropMennenConfig {
    fn default() -> Self {
        Self {
            enforce_validity_range: true,
            minimum_confidence_threshold: 0.6,
            enable_form_factor_correction: true,
            use_improved_appendage_resistance: true,
        }
    }
}

impl HoltropMennenCalculator {
    /// Create a new Holtrop-Mennen calculator
    pub fn new() -> Self {
        Self::with_config(HoltropMennenConfig::default())
    }

    /// Create calculator with custom configuration
    pub fn with_config(config: HoltropMennenConfig) -> Self {
        Self {
            config,
            initialized: true,
        }
    }

    /// Check if calculator is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Calculate resistance using Holtrop-Mennen method
    pub fn calculate_resistance(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
    ) -> Result<HoltropMennenResult> {
        info!("Starting Holtrop-Mennen resistance calculation for {}", vessel.name);

        // Validate inputs
        self.validate_inputs(vessel, conditions)?;

        // Check applicability
        let applicability = self.assess_applicability(vessel, conditions)?;
        if self.config.enforce_validity_range && applicability < self.config.minimum_confidence_threshold {
            return Err(ResistanceError::holtrop_mennen_not_applicable(
                format!("Applicability confidence {:.2} below threshold {:.2}",
                        applicability, self.config.minimum_confidence_threshold)
            ));
        }

        // Calculate dimensional parameters
        let params = self.calculate_dimensional_parameters(vessel, conditions)?;
        debug!("Dimensional parameters calculated: Fn={:.4}, Re={:.2e}", params.froude_number, params.reynolds_number);

        // Calculate resistance components
        let frictional_resistance = self.calculate_frictional_resistance(&params)?;
        let appendage_resistance = self.calculate_appendage_resistance(vessel, &params)?;
        let wave_resistance = self.calculate_wave_resistance(vessel, &params)?;
        let bulbous_bow_resistance = self.calculate_bulbous_bow_resistance(vessel, &params)?;
        let transom_resistance = self.calculate_transom_resistance(vessel, &params)?;
        let model_ship_correlation = self.calculate_model_ship_correlation(vessel, &params)?;

        // Total resistance
        let total_resistance = frictional_resistance + appendage_resistance + wave_resistance 
                             + bulbous_bow_resistance + transom_resistance + model_ship_correlation;

        // Resistance coefficient
        let resistance_coefficient = total_resistance / (0.5 * params.water_density * 
                                                       params.speed_ms.powi(2) * params.wetted_surface_area);

        // Effective power
        let effective_power = total_resistance * params.speed_ms / 1000.0; // kW

        // Validation flags
        let validation_flags = self.generate_validation_flags(vessel, conditions, applicability)?;

        info!("Holtrop-Mennen calculation completed: RT={:.0} N, PE={:.0} kW", 
              total_resistance, effective_power);

        Ok(HoltropMennenResult {
            total_resistance,
            frictional_resistance,
            appendage_resistance,
            wave_resistance,
            bulbous_bow_resistance,
            transom_resistance,
            model_ship_correlation,
            resistance_coefficient,
            effective_power,
            validation_flags,
        })
    }

    /// Assess applicability of Holtrop-Mennen method
    pub fn assess_applicability(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
    ) -> Result<f64> {
        let mut validity_scores = Vec::new();

        // Length range: 15m ≤ L ≤ 450m
        let length_score = if vessel.hull.length_between_perpendiculars >= 15.0 && 
                              vessel.hull.length_between_perpendiculars <= 450.0 {
            1.0
        } else {
            let deviation = if vessel.hull.length_between_perpendiculars < 15.0 {
                (15.0 - vessel.hull.length_between_perpendiculars) / 15.0
            } else {
                (vessel.hull.length_between_perpendiculars - 450.0) / 450.0
            };
            (1.0 - deviation.min(1.0)).max(0.0)
        };
        validity_scores.push(length_score);

        // Block coefficient range: 0.4 ≤ CB ≤ 0.85
        let cb_score = if vessel.hull.block_coefficient >= 0.4 && vessel.hull.block_coefficient <= 0.85 {
            1.0
        } else {
            let deviation = if vessel.hull.block_coefficient < 0.4 {
                (0.4 - vessel.hull.block_coefficient) / 0.4
            } else {
                (vessel.hull.block_coefficient - 0.85) / 0.85
            };
            (1.0 - deviation.min(1.0)).max(0.0)
        };
        validity_scores.push(cb_score);

        // Prismatic coefficient range: 0.55 ≤ CP ≤ 0.85
        let cp_score = if vessel.hull.prismatic_coefficient >= 0.55 && vessel.hull.prismatic_coefficient <= 0.85 {
            1.0
        } else {
            let deviation = if vessel.hull.prismatic_coefficient < 0.55 {
                (0.55 - vessel.hull.prismatic_coefficient) / 0.55
            } else {
                (vessel.hull.prismatic_coefficient - 0.85) / 0.85
            };
            (1.0 - deviation.min(1.0)).max(0.0)
        };
        validity_scores.push(cp_score);

        // Froude number range: 0.1 ≤ Fn ≤ 0.8
        let speed_ms = conditions.speed_knots * 0.5144;
        let froude_number = speed_ms / sqrt(9.81 * vessel.hull.length_between_perpendiculars);
        let fn_score = if froude_number >= 0.1 && froude_number <= 0.8 {
            1.0
        } else {
            let deviation = if froude_number < 0.1 {
                (0.1 - froude_number) / 0.1
            } else {
                (froude_number - 0.8) / 0.8
            };
            (1.0 - deviation.min(1.0)).max(0.0)
        };
        validity_scores.push(fn_score);

        // L/B ratio: 3.9 ≤ L/B ≤ 14.9
        let lb_ratio = vessel.hull.length_between_perpendiculars / vessel.hull.beam;
        let lb_score = if lb_ratio >= 3.9 && lb_ratio <= 14.9 {
            1.0
        } else {
            let deviation = if lb_ratio < 3.9 {
                (3.9 - lb_ratio) / 3.9
            } else {
                (lb_ratio - 14.9) / 14.9
            };
            (1.0 - deviation.min(1.0)).max(0.0)
        };
        validity_scores.push(lb_score);

        // B/T ratio: 2.1 ≤ B/T ≤ 4.0
        let bt_ratio = vessel.hull.beam / vessel.hull.draft;
        let bt_score = if bt_ratio >= 2.1 && bt_ratio <= 4.0 {
            1.0
        } else {
            let deviation = if bt_ratio < 2.1 {
                (2.1 - bt_ratio) / 2.1
            } else {
                (bt_ratio - 4.0) / 4.0
            };
            (1.0 - deviation.min(1.0)).max(0.0)
        };
        validity_scores.push(bt_score);

        // Calculate weighted average
        let total_score: f64 = validity_scores.iter().sum();
        let applicability = total_score / validity_scores.len() as f64;

        debug!("Applicability assessment: {:.3} (length: {:.3}, CB: {:.3}, CP: {:.3}, Fn: {:.3}, L/B: {:.3}, B/T: {:.3})",
               applicability, length_score, cb_score, cp_score, fn_score, lb_score, bt_score);

        Ok(applicability)
    }

    /// Calculate dimensional parameters needed for resistance calculation
    fn calculate_dimensional_parameters(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
    ) -> Result<DimensionalParameters> {
        let speed_ms = conditions.speed_knots * 0.5144; // Convert knots to m/s
        let lwl = vessel.hull.length_waterline;
        let lbp = vessel.hull.length_between_perpendiculars;
        let beam = vessel.hull.beam;
        let draft = conditions.draft;
        let displacement_volume = conditions.displacement / conditions.water_density * 1000.0; // m³

        // Froude number
        let froude_number = speed_ms / sqrt(9.81 * lbp);

        // Reynolds number
        let reynolds_number = speed_ms * lbp / conditions.kinematic_viscosity;

        // Wetted surface area (Holtrop approximation)
        let wetted_surface_area = lwl * (2.0 * draft + beam) * sqrt(vessel.hull.midship_coefficient) * 
                                 (0.453 + 0.4425 * vessel.hull.block_coefficient - 
                                  0.2862 * vessel.hull.midship_coefficient - 
                                  0.003467 * beam / draft + 0.3696 * vessel.hull.waterplane_coefficient) + 
                                 2.38 * vessel.hull.transom_area / vessel.hull.block_coefficient;

        // Length-displacement ratio
        let length_displacement_ratio = lbp / pow(displacement_volume, 1.0/3.0);

        // Beam-draft ratio
        let beam_draft_ratio = beam / draft;

        // Longitudinal prismatic coefficient
        let longitudinal_prismatic_coefficient = displacement_volume / (lwl * vessel.hull.midship_coefficient * beam * draft);

        Ok(DimensionalParameters {
            speed_ms,
            froude_number,
            reynolds_number,
            wetted_surface_area,
            beam_draft_ratio,
            longitudinal_prismatic_coefficient,
            displacement_volume,
            water_density: conditions.water_density,
        })
    }

    /// Calculate frictional resistance
    fn calculate_frictional_resistance(&self, params: &DimensionalParameters) -> Result<f64> {
        // ITTC 1957 friction formula
        let cf = 0.075 / (log(params.reynolds_number) / log(10.0) - 2.0).powi(2);
        
        // Form factor (approximation for preliminary design)
        let form_factor = if self.config.enable_form_factor_correction {
            1.0 + 0.93 * params.beam_draft_ratio.powf(-0.92497) * 
            (0.95 - params.longitudinal_prismatic_coefficient).powf(-0.521448) * 
            (1.0 - params.longitudinal_prismatic_coefficient + 0.0225).powf(0.6906)
        } else {
            1.0
        };

        let frictional_resistance = 0.5 * params.water_density * params.speed_ms.powi(2) * 
                                  params.wetted_surface_area * cf * form_factor;

        debug!("Frictional resistance: CF={:.6}, k={:.3}, RF={:.0} N", cf, form_factor, frictional_resistance);

        Ok(frictional_resistance)
    }

    /// Calculate appendage resistance
    fn calculate_appendage_resistance(
        &self,
        vessel: &VesselParameters,
        params: &DimensionalParameters,
    ) -> Result<f64> {
        if vessel.appendages.is_empty() {
            return Ok(0.0);
        }

        let mut total_appendage_resistance = 0.0;

        for appendage in &vessel.appendages {
            // Appendage resistance coefficient (typical values)
            let appendage_cf = match appendage.appendage_type {
                AppendageType::Rudder => 0.008,
                AppendageType::Skeg => 0.006,
                AppendageType::Bracket => 0.040,
                AppendageType::Shaft => 0.006,
                AppendageType::BossArms => 0.020,
                AppendageType::Other(_) => appendage.drag_coefficient,
            };

            let appendage_resistance = 0.5 * params.water_density * params.speed_ms.powi(2) * 
                                     appendage.area * appendage_cf;
            
            total_appendage_resistance += appendage_resistance;

            debug!("Appendage {:?}: area={:.1} m², CF={:.4}, R={:.0} N", 
                   appendage.appendage_type, appendage.area, appendage_cf, appendage_resistance);
        }

        Ok(total_appendage_resistance)
    }

    /// Calculate wave resistance
    fn calculate_wave_resistance(
        &self,
        vessel: &VesselParameters,
        params: &DimensionalParameters,
    ) -> Result<f64> {
        let lbp = vessel.hull.length_between_perpendiculars;
        let beam = vessel.hull.beam;
        let draft = vessel.hull.draft;
        let _cb = vessel.hull.block_coefficient;
        let cp = vessel.hull.prismatic_coefficient;
        let _cwp = vessel.hull.waterplane_coefficient;
        let _lcb = vessel.hull.longitudinal_center_buoyancy / 100.0; // Convert from % to fraction
        let ie = vessel.hull.half_angle_entrance * std::f64::consts::PI / 180.0; // Convert to radians

        // Calculate c1 coefficient
        let c1 = 2223105.0 * pow(lbp.powf(3.78613) * (draft / beam).powf(1.07961) * 
                                (90.0 - ie * 180.0 / std::f64::consts::PI).powf(-1.37565), 0.01);

        // Calculate c2 coefficient  
        let c2 = exp(-1.89 * sqrt(c1));

        // Calculate c5 coefficient
        let c5 = 1.0 - 0.8 * vessel.hull.transom_area / (beam * draft * vessel.hull.midship_coefficient);

        // Calculate m1 coefficient
        let m1 = 0.0140407 * lbp / draft - 1.75254 * pow(params.displacement_volume, 1.0/3.0) / lbp - 
                 4.79323 * beam / lbp - c16(cp);

        // Calculate m2 coefficient
        let m2 = c17(cp) * c2 * exp(-0.1 * params.froude_number.powi(-2));

        // Calculate λ (lambda) - wave resistance factor
        let lambda = if lbp / beam < 12.0 {
            1.446 * cp - 0.03 * lbp / beam
        } else {
            1.446 * cp - 0.36
        };

        // Wave resistance coefficient
        let cw = c1 * c2 * c5 * exp(m1 * params.froude_number.powf(0.9) + m2 * cos(lambda * params.froude_number.powi(-2)));

        let wave_resistance = 0.5 * params.water_density * params.speed_ms.powi(2) * 
                             params.wetted_surface_area * cw;

        debug!("Wave resistance: CW={:.6}, RW={:.0} N", cw, wave_resistance);

        Ok(wave_resistance)
    }

    /// Calculate bulbous bow resistance
    fn calculate_bulbous_bow_resistance(
        &self,
        vessel: &VesselParameters,
        params: &DimensionalParameters,
    ) -> Result<f64> {
        if let Some(bulb) = &vessel.hull.bulbous_bow {
            let _beam = vessel.hull.beam;
            let _draft = vessel.hull.draft;
            let tf = vessel.hull.draft; // Forward draft (approximation)

            // Bulbous bow resistance coefficient
            let fni = params.speed_ms / sqrt(9.81 * (tf - bulb.bulb_center_height - 0.25 * sqrt(bulb.bulb_area)) + 0.15 * params.speed_ms.powi(2));
            
            let rb = if fni < 0.2 {
                0.11 * exp(-3.0 * fni.powi(2)) * bulb.bulb_area.powf(1.5) * params.water_density * 9.81 / 
                (bulb.bulb_area + params.wetted_surface_area)
            } else if fni < 0.55 {
                0.11 * exp(-3.0 * fni.powi(2)) * bulb.bulb_area.powf(1.5) * params.water_density * 9.81 / 
                (bulb.bulb_area + params.wetted_surface_area) * (1.0 - (fni - 0.2) / 0.35)
            } else {
                0.0
            };

            debug!("Bulbous bow resistance: FnI={:.3}, RB={:.0} N", fni, rb);
            Ok(rb)
        } else {
            Ok(0.0)
        }
    }

    /// Calculate transom resistance
    fn calculate_transom_resistance(
        &self,
        vessel: &VesselParameters,
        params: &DimensionalParameters,
    ) -> Result<f64> {
        if vessel.hull.transom_area > 0.0 {
            let beam = vessel.hull.beam;
            let cwp = vessel.hull.waterplane_coefficient;
            
            let fnt = params.speed_ms / sqrt(2.0 * 9.81 * vessel.hull.transom_area / (beam + beam * cwp));
            
            let c6 = if fnt < 5.0 {
                0.2 * (1.0 - 0.2 * fnt)
            } else {
                0.0
            };

            let transom_resistance = 0.5 * params.water_density * params.speed_ms.powi(2) * 
                                   vessel.hull.transom_area * c6;

            debug!("Transom resistance: FnT={:.3}, C6={:.3}, RTR={:.0} N", fnt, c6, transom_resistance);
            Ok(transom_resistance)
        } else {
            Ok(0.0)
        }
    }

    /// Calculate model-ship correlation resistance
    fn calculate_model_ship_correlation(
        &self,
        vessel: &VesselParameters,
        params: &DimensionalParameters,
    ) -> Result<f64> {
        let lbp = vessel.hull.length_between_perpendiculars;
        let beam = vessel.hull.beam;
        let cp = vessel.hull.prismatic_coefficient;
        let cwp = vessel.hull.waterplane_coefficient;

        // Model-ship correlation coefficient (ca)
        let ca = 0.006 * (lbp + 100.0).powf(-0.16) - 0.00205 + 
                0.003 * sqrt(lbp / 7.5) * pow(vessel.hull.block_coefficient, 4.0) * c2(cp) * (0.04 - c4(cwp));

        let model_ship_correlation = 0.5 * params.water_density * params.speed_ms.powi(2) * 
                                   params.wetted_surface_area * ca;

        debug!("Model-ship correlation: CA={:.6}, RA={:.0} N", ca, model_ship_correlation);

        Ok(model_ship_correlation)
    }

    /// Validate input parameters
    fn validate_inputs(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
    ) -> Result<()> {
        let mut issues = Vec::new();

        // Vessel validation
        if vessel.hull.length_between_perpendiculars <= 0.0 {
            issues.push("Length between perpendiculars must be positive".to_string());
        }
        if vessel.hull.beam <= 0.0 {
            issues.push("Beam must be positive".to_string());
        }
        if vessel.hull.draft <= 0.0 {
            issues.push("Draft must be positive".to_string());
        }
        if vessel.hull.block_coefficient <= 0.0 || vessel.hull.block_coefficient > 1.0 {
            issues.push("Block coefficient must be between 0 and 1".to_string());
        }
        if vessel.hull.prismatic_coefficient <= 0.0 || vessel.hull.prismatic_coefficient > 1.0 {
            issues.push("Prismatic coefficient must be between 0 and 1".to_string());
        }

        // Operating conditions validation
        if conditions.speed_knots < 0.0 {
            issues.push("Speed cannot be negative".to_string());
        }
        if conditions.draft <= 0.0 {
            issues.push("Draft must be positive".to_string());
        }
        if conditions.displacement <= 0.0 {
            issues.push("Displacement must be positive".to_string());
        }
        if conditions.water_density <= 0.0 {
            issues.push("Water density must be positive".to_string());
        }
        if conditions.kinematic_viscosity <= 0.0 {
            issues.push("Kinematic viscosity must be positive".to_string());
        }

        if !issues.is_empty() {
            return Err(ResistanceError::validation_failed(issues));
        }

        Ok(())
    }

    /// Generate validation flags
    fn generate_validation_flags(
        &self,
        vessel: &VesselParameters,
        conditions: &OperatingConditions,
        applicability: f64,
    ) -> Result<ValidationFlags> {
        let lbp = vessel.hull.length_between_perpendiculars;
        let cb = vessel.hull.block_coefficient;
        let cp = vessel.hull.prismatic_coefficient;
        let speed_ms = conditions.speed_knots * 0.5144;
        let froude_number = speed_ms / sqrt(9.81 * lbp);

        Ok(ValidationFlags {
            length_range_valid: lbp >= 15.0 && lbp <= 450.0,
            speed_range_valid: froude_number >= 0.1 && froude_number <= 0.8,
            cb_range_valid: cb >= 0.4 && cb <= 0.85,
            cp_range_valid: cp >= 0.55 && cp <= 0.85,
            overall_validity: applicability,
        })
    }
}

/// Helper coefficients for Holtrop-Mennen calculations
fn c16(cp: f64) -> f64 {
    if cp < 0.8 {
        8.07981 * cp - 13.8673 * cp.powi(2) + 6.984388 * cp.powi(3)
    } else {
        1.73014 - 0.7067 * cp
    }
}

fn c17(cp: f64) -> f64 {
    if cp < 0.7 {
        6.919385 - 7.23014 * cp + 2.441481 * cp.powi(2)
    } else {
        -0.4 + 1.0 * cp
    }
}

fn c2(cp: f64) -> f64 {
    if cp < 0.7 {
        0.20 - 0.28571 * cp
    } else {
        0.30 - 0.71429 * (cp - 0.7)
    }
}

fn c4(cwp: f64) -> f64 {
    cwp
}

/// Dimensional parameters for calculations
#[derive(Debug, Clone)]
struct DimensionalParameters {
    speed_ms: f64,
    froude_number: f64,
    reynolds_number: f64,
    wetted_surface_area: f64,
    beam_draft_ratio: f64,
    longitudinal_prismatic_coefficient: f64,
    displacement_volume: f64,
    water_density: f64,
}

impl Default for HoltropMennenCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_holtrop_mennen_calculator_creation() {
        let calculator = HoltropMennenCalculator::new();
        assert!(calculator.is_initialized());
    }

    #[test]
    fn test_applicability_assessment() {
        let calculator = HoltropMennenCalculator::new();
        let vessel = VesselParameters::default_container_ship();
        let conditions = OperatingConditions::default();

        let applicability = calculator.assess_applicability(&vessel, &conditions);
        assert!(applicability.is_ok());
        
        let score = applicability.unwrap();
        assert!(score > 0.5); // Should be reasonable for default container ship
    }

    #[test]
    fn test_resistance_calculation() {
        let calculator = HoltropMennenCalculator::new();
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

        let result = calculator.calculate_resistance(&vessel, &conditions);
        assert!(result.is_ok());
        
        let resistance_result = result.unwrap();
        assert!(resistance_result.total_resistance > 0.0);
        assert!(resistance_result.frictional_resistance > 0.0);
        assert!(resistance_result.effective_power > 0.0);
    }

    #[test]
    fn test_input_validation() {
        let calculator = HoltropMennenCalculator::new();
        let mut vessel = VesselParameters::default_container_ship();
        vessel.hull.length_between_perpendiculars = -10.0; // Invalid
        
        let conditions = OperatingConditions::default();

        let result = calculator.calculate_resistance(&vessel, &conditions);
        assert!(result.is_err());
        
        if let Err(ResistanceError::ValidationFailed { issues }) = result {
            assert!(!issues.is_empty());
        } else {
            panic!("Expected ValidationFailed error");
        }
    }
} 