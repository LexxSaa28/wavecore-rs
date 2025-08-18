//! Windage resistance calculation module
//! 
//! This module implements wind resistance calculations for ships, including
//! superstructure effects and relative wind calculation.

use crate::{
    types::*,
    errors::{Result, ResistanceError},
};
use libm::{cos, sin, sqrt, atan2};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Windage resistance calculator
#[derive(Debug, Clone)]
pub struct WindageCalculator {
    config: WindageConfig,
    initialized: bool,
}

/// Configuration for windage calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindageConfig {
    pub use_relative_wind: bool,
    pub include_heel_effects: bool,
    pub apply_gust_factor: bool,
}

impl Default for WindageConfig {
    fn default() -> Self {
        Self {
            use_relative_wind: true,
            include_heel_effects: false,
            apply_gust_factor: true,
        }
    }
}

impl WindageCalculator {
    /// Create a new windage calculator
    pub fn new() -> Self {
        Self::with_config(WindageConfig::default())
    }

    /// Create calculator with custom configuration
    pub fn with_config(config: WindageConfig) -> Self {
        Self {
            config,
            initialized: true,
        }
    }

    /// Check if calculator is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Calculate wind resistance
    pub fn calculate_wind_resistance(
        &self,
        vessel: &VesselParameters,
        wind_conditions: &WindConditions,
    ) -> Result<WindResistance> {
        info!("Calculating wind resistance for {}", vessel.name);

        // Calculate relative wind if vessel speed is considered
        let relative_wind = if self.config.use_relative_wind {
            self.calculate_relative_wind(wind_conditions, 0.0)? // Assuming vessel speed is in wind conditions context
        } else {
            RelativeWind {
                speed: wind_conditions.wind_speed,
                direction: wind_conditions.wind_direction,
            }
        };

        // Calculate wind force components
        let (fx, fy, mz) = self.calculate_wind_forces(vessel, &relative_wind, wind_conditions)?;

        debug!("Wind forces calculated: FX={:.0} N, FY={:.0} N, MZ={:.0} N⋅m", fx, fy, mz);

        Ok(WindResistance {
            longitudinal_force: fx,
            lateral_force: fy,
            yaw_moment: mz,
            relative_wind_speed: relative_wind.speed,
            relative_wind_angle: relative_wind.direction,
        })
    }

    /// Calculate relative wind considering vessel motion
    fn calculate_relative_wind(
        &self,
        wind_conditions: &WindConditions,
        vessel_speed_ms: f64,
    ) -> Result<RelativeWind> {
        let wind_speed = wind_conditions.wind_speed;
        let wind_direction_rad = wind_conditions.wind_direction * std::f64::consts::PI / 180.0;

        // Wind velocity components (in vessel coordinate system)
        let wind_u = wind_speed * cos(wind_direction_rad); // Along vessel
        let wind_v = wind_speed * sin(wind_direction_rad); // Across vessel

        // Relative wind components (subtract vessel velocity)
        let relative_u = wind_u - vessel_speed_ms;
        let relative_v = wind_v;

        // Relative wind speed and direction
        let relative_speed = sqrt(relative_u * relative_u + relative_v * relative_v);
        let relative_direction = atan2(relative_v, relative_u) * 180.0 / std::f64::consts::PI;

        Ok(RelativeWind {
            speed: relative_speed,
            direction: relative_direction,
        })
    }

    /// Calculate wind forces on vessel
    fn calculate_wind_forces(
        &self,
        vessel: &VesselParameters,
        relative_wind: &RelativeWind,
        wind_conditions: &WindConditions,
    ) -> Result<(f64, f64, f64)> {
        let superstructure = &vessel.superstructure;
        let wind_angle_rad = relative_wind.direction * std::f64::consts::PI / 180.0;
        let dynamic_pressure = 0.5 * wind_conditions.air_density * relative_wind.speed.powi(2);

        // Apply gust factor if enabled
        let gust_factor = if self.config.apply_gust_factor {
            wind_conditions.gust_factor
        } else {
            1.0
        };

        // Longitudinal force (drag in heading direction)
        let cd_longitudinal = self.interpolate_drag_coefficient(
            wind_angle_rad.abs(),
            superstructure.drag_coefficient_head,
            superstructure.drag_coefficient_beam,
        );
        
        let projected_area_x = superstructure.frontal_area * cos(wind_angle_rad).abs() +
                              superstructure.lateral_area * sin(wind_angle_rad).abs();
        
        let fx = dynamic_pressure * cd_longitudinal * projected_area_x * gust_factor * cos(wind_angle_rad);

        // Lateral force (side force)
        let cd_lateral = self.interpolate_drag_coefficient(
            (std::f64::consts::PI / 2.0 - wind_angle_rad.abs()).abs(),
            superstructure.drag_coefficient_beam,
            superstructure.drag_coefficient_head,
        );

        let projected_area_y = superstructure.lateral_area * cos(wind_angle_rad).abs() +
                              superstructure.frontal_area * sin(wind_angle_rad).abs();

        let fy = dynamic_pressure * cd_lateral * projected_area_y * gust_factor * sin(wind_angle_rad);

        // Yaw moment
        let moment_arm = superstructure.center_of_effort_height;
        let mz = fy * moment_arm;

        Ok((fx, fy, mz))
    }

    /// Interpolate drag coefficient based on wind angle
    fn interpolate_drag_coefficient(&self, angle_rad: f64, cd_head: f64, cd_beam: f64) -> f64 {
        let angle_norm = angle_rad / (std::f64::consts::PI / 2.0); // Normalize to 0-1
        cd_head + (cd_beam - cd_head) * angle_norm
    }

    /// Assess model confidence for wind resistance calculation
    pub fn assess_model_confidence(&self, vessel: &VesselParameters) -> Result<f64> {
        let superstructure = &vessel.superstructure;
        let mut confidence_factors = Vec::new();

        // Check if drag coefficients are reasonable
        let cd_head_valid = superstructure.drag_coefficient_head > 0.0 && 
                           superstructure.drag_coefficient_head < 2.0;
        confidence_factors.push(if cd_head_valid { 1.0 } else { 0.5 });

        let cd_beam_valid = superstructure.drag_coefficient_beam > 0.0 && 
                           superstructure.drag_coefficient_beam < 2.5;
        confidence_factors.push(if cd_beam_valid { 1.0 } else { 0.5 });

        // Check if areas are reasonable
        let area_ratio = superstructure.lateral_area / superstructure.frontal_area;
        let area_ratio_valid = area_ratio > 1.0 && area_ratio < 10.0; // Typical range
        confidence_factors.push(if area_ratio_valid { 1.0 } else { 0.7 });

        // Check center of effort height
        let coe_valid = superstructure.center_of_effort_height > 0.0 && 
                       superstructure.center_of_effort_height < superstructure.height_above_waterline;
        confidence_factors.push(if coe_valid { 1.0 } else { 0.8 });

        // Calculate overall confidence
        let total_confidence: f64 = confidence_factors.iter().sum();
        let confidence = total_confidence / confidence_factors.len() as f64;

        debug!("Wind model confidence: {:.3}", confidence);
        Ok(confidence)
    }
}

/// Relative wind calculation result
#[derive(Debug, Clone)]
struct RelativeWind {
    speed: f64,      // m/s
    direction: f64,  // degrees from bow
}

impl Default for WindageCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_windage_calculator_creation() {
        let calculator = WindageCalculator::new();
        assert!(calculator.is_initialized());
    }

    #[test]
    fn test_wind_resistance_calculation() {
        let calculator = WindageCalculator::new();
        let vessel = VesselParameters::default_container_ship();
        let wind_conditions = WindConditions {
            wind_speed: 15.0,      // 15 m/s
            wind_direction: 45.0,   // 45 degrees from bow
            air_density: 1.225,    // Standard air density
            gust_factor: 1.2,
        };

        let result = calculator.calculate_wind_resistance(&vessel, &wind_conditions);
        assert!(result.is_ok());

        let wind_resistance = result.unwrap();
        assert!(wind_resistance.longitudinal_force.abs() > 0.0);
        assert!(wind_resistance.lateral_force.abs() > 0.0);
        assert!(wind_resistance.yaw_moment.abs() > 0.0);
    }

    #[test]
    fn test_relative_wind_calculation() {
        let calculator = WindageCalculator::new();
        let wind_conditions = WindConditions {
            wind_speed: 20.0,
            wind_direction: 0.0,   // Head wind
            air_density: 1.225,
            gust_factor: 1.0,
        };

        let relative_wind = calculator.calculate_relative_wind(&wind_conditions, 10.0);
        assert!(relative_wind.is_ok());

        let rel_wind = relative_wind.unwrap();
        // Head wind minus vessel speed should be 10 m/s
        assert_relative_eq!(rel_wind.speed, 10.0, epsilon = 0.1);
    }

    #[test]
    fn test_model_confidence_assessment() {
        let calculator = WindageCalculator::new();
        let vessel = VesselParameters::default_container_ship();

        let confidence = calculator.assess_model_confidence(&vessel);
        assert!(confidence.is_ok());

        let score = confidence.unwrap();
        assert!(score >= 0.0 && score <= 1.0);
        assert!(score > 0.5); // Should be reasonable for default vessel
    }

    #[test]
    fn test_drag_coefficient_interpolation() {
        let calculator = WindageCalculator::new();
        
        // Head-on wind (0 degrees)
        let cd_head = calculator.interpolate_drag_coefficient(0.0, 0.8, 1.2);
        assert_relative_eq!(cd_head, 0.8, epsilon = 0.01);
        
        // Beam wind (90 degrees)
        let cd_beam = calculator.interpolate_drag_coefficient(std::f64::consts::PI / 2.0, 0.8, 1.2);
        assert_relative_eq!(cd_beam, 1.2, epsilon = 0.01);
        
        // 45 degree wind
        let cd_45 = calculator.interpolate_drag_coefficient(std::f64::consts::PI / 4.0, 0.8, 1.2);
        assert_relative_eq!(cd_45, 1.0, epsilon = 0.01); // Should be average
    }
} 