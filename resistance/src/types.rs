//! Type definitions for resistance calculations
//! 
//! This module contains all the data structures used throughout the resistance
//! calculation system, including vessel parameters, operating conditions, 
//! environmental data, and calculation results.

use nalgebra as na;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete vessel parameters for resistance calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VesselParameters {
    pub name: String,
    pub vessel_type: String,
    pub hull: HullParameters,
    pub propulsion: PropulsionParameters,
    pub superstructure: SuperstructureParameters,
    pub appendages: Vec<AppendageParameters>,
}

/// Hull geometry and characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HullParameters {
    pub length_overall: f64,           // LOA (m)
    pub length_between_perpendiculars: f64, // LBP (m) 
    pub length_waterline: f64,         // LWL (m)
    pub beam: f64,                     // B (m)
    pub draft: f64,                    // T (m)
    pub displacement: f64,             // Δ (m³)
    pub block_coefficient: f64,        // CB
    pub midship_coefficient: f64,      // CM
    pub waterplane_coefficient: f64,   // CWP
    pub prismatic_coefficient: f64,    // CP
    pub longitudinal_center_buoyancy: f64, // LCB from midships (% LBP)
    pub longitudinal_center_flotation: f64, // LCF from midships (% LBP)
    pub half_angle_entrance: f64,      // iE (degrees)
    pub stern_type: SternType,
    pub bulbous_bow: Option<BulbousBowParameters>,
    pub transom_area: f64,             // AT (m²)
    pub hull_efficiency: f64,          // ηH
}

/// Propulsion system parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropulsionParameters {
    pub propeller_diameter: f64,       // DP (m)
    pub propeller_pitch_ratio: f64,    // P/D
    pub number_of_propellers: u32,     // Number of propellers
    pub propeller_efficiency: f64,     // ηP
    pub shaft_efficiency: f64,         // ηS
    pub propeller_type: PropellerType,
}

/// Superstructure parameters for windage calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperstructureParameters {
    pub frontal_area: f64,             // AF (m²)
    pub lateral_area: f64,             // AL (m²)
    pub height_above_waterline: f64,   // Height (m)
    pub center_of_effort_height: f64,  // COE height (m)
    pub drag_coefficient_head: f64,    // CDX head winds
    pub drag_coefficient_beam: f64,    // CDY beam winds
}

/// Appendage parameters (rudder, brackets, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendageParameters {
    pub appendage_type: AppendageType,
    pub area: f64,                     // Appendage area (m²)
    pub drag_coefficient: f64,         // CD for appendage
}

/// Bulbous bow parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulbousBowParameters {
    pub bulb_area: f64,                // ABT (m²)
    pub bulb_center_height: f64,       // hB (m)
}

/// Operating conditions for the vessel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingConditions {
    pub speed_knots: f64,              // V (knots)
    pub draft: f64,                    // T (m) - current draft
    pub displacement: f64,             // Δ (tonnes)
    pub trim: f64,                     // Trim (m, positive by stern)
    pub heel_angle: f64,               // Heel angle (degrees)
    pub water_density: f64,            // ρ (kg/m³)
    pub kinematic_viscosity: f64,      // ν (m²/s)
}

/// Environmental conditions affecting resistance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConditions {
    pub wave_spectrum: Option<WaveSpectrum>,
    pub wind_conditions: Option<WindConditions>,
    pub current: Option<CurrentConditions>,
    pub water_temperature: f64,        // Temperature (°C)
    pub salinity: f64,                 // Salinity (ppt)
}

/// Wave spectrum for added resistance calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveSpectrum {
    pub significant_wave_height: f64,  // Hs (m)
    pub peak_period: f64,              // Tp (s)
    pub wave_direction: f64,           // Direction (degrees from bow)
    pub spectrum_type: SpectrumType,
    pub frequencies: Array1<f64>,      // ω (rad/s)
    pub spectral_densities: Array1<f64>, // S(ω) (m²s)
}

/// Wind conditions for windage calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindConditions {
    pub wind_speed: f64,               // True wind speed (m/s)
    pub wind_direction: f64,           // Direction (degrees from bow)
    pub air_density: f64,              // ρ_air (kg/m³)
    pub gust_factor: f64,              // Gust factor
}

/// Current conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentConditions {
    pub current_speed: f64,            // Current speed (m/s)
    pub current_direction: f64,        // Direction (degrees)
    pub current_profile: CurrentProfile,
}

/// Comprehensive resistance calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalResistanceResult {
    pub total_resistance: f64,         // RT (N)
    pub breakdown: ResistanceBreakdown,
    pub power_requirements: PowerRequirements,
    pub confidence: f64,               // Confidence score (0-1)
    pub calculation_metadata: CalculationMetadata,
}

/// Breakdown of resistance components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResistanceBreakdown {
    pub calm_water_resistance: HoltropMennenResult,
    pub added_resistance: AddedResistanceResult,
    pub wind_resistance: WindResistance,
}

/// Holtrop-Mennen specific results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoltropMennenResult {
    pub total_resistance: f64,         // RT (N)
    pub frictional_resistance: f64,    // RF (N)
    pub appendage_resistance: f64,     // RAPP (N)
    pub wave_resistance: f64,          // RW (N)
    pub bulbous_bow_resistance: f64,   // RB (N)
    pub transom_resistance: f64,       // RTR (N)
    pub model_ship_correlation: f64,   // RA (N)
    pub resistance_coefficient: f64,   // CT
    pub effective_power: f64,          // PE (kW)
    pub validation_flags: ValidationFlags,
}

/// Added resistance in waves result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddedResistanceResult {
    pub total_resistance: f64,         // RAW (N)
    pub mean_added_resistance: f64,    // Mean component (N)
    pub oscillatory_component: f64,    // Oscillatory component (N)
    pub rao_data: Option<RAOData>,     // RAO if available
    pub integration_method: String,    // Method used
}

/// Wind resistance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindResistance {
    pub longitudinal_force: f64,       // FX (N)
    pub lateral_force: f64,            // FY (N)
    pub yaw_moment: f64,               // MZ (N⋅m)
    pub relative_wind_speed: f64,      // Vr (m/s)
    pub relative_wind_angle: f64,      // βr (degrees)
}

/// RAO (Response Amplitude Operator) data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAOData {
    pub frequencies: Array1<f64>,      // ω (rad/s)
    pub surge_rao: Array1<f64>,        // |RAO_1| (m/m)
    pub heave_rao: Array1<f64>,        // |RAO_3| (m/m)
    pub pitch_rao: Array1<f64>,        // |RAO_5| (rad/m)
    pub added_resistance_rao: Array1<f64>, // RAO_AW (N⋅s²/m²)
}

/// Power requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerRequirements {
    pub effective_power: f64,          // PE (W)
    pub brake_power: f64,              // PB (W)
    pub propeller_efficiency: f64,     // ηP
}

/// Calculation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationMetadata {
    pub method: String,
    pub vessel_type: String,
    pub conditions: OperatingConditions,
    pub environment: EnvironmentalConditions,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Validation flags for checking applicability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFlags {
    pub length_range_valid: bool,      // LBP within valid range
    pub speed_range_valid: bool,       // Froude number within range
    pub cb_range_valid: bool,          // Block coefficient within range
    pub cp_range_valid: bool,          // Prismatic coefficient within range
    pub overall_validity: f64,         // Overall validity score (0-1)
}

// Enumerations

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SternType {
    V,                                 // V-shaped stern
    Normal,                            // Normal stern
    UType,                             // U-shaped stern
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropellerType {
    Conventional,                      // Conventional propeller
    Ducted,                           // Ducted propeller
    Podded,                           // Podded propulsion
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppendageType {
    Rudder,
    Skeg,
    Bracket,
    Shaft,
    BossArms,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpectrumType {
    JONSWAP,
    PiersonMoskowitz,
    BMKG,                             // Indonesian-specific spectrum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurrentProfile {
    Uniform,
    Linear,
    Exponential,
    Measured(Array1<f64>),            // Depth-dependent current
}

// Default implementations

impl Default for OperatingConditions {
    fn default() -> Self {
        Self {
            speed_knots: 15.0,
            draft: 8.0,
            displacement: 10000.0,
            trim: 0.0,
            heel_angle: 0.0,
            water_density: 1025.0,        // Seawater density
            kinematic_viscosity: 1.188e-6, // Seawater at 15°C
        }
    }
}

impl EnvironmentalConditions {
    /// Create calm sea conditions (no waves, wind, or current)
    pub fn calm_sea() -> Self {
        Self {
            wave_spectrum: None,
            wind_conditions: None,
            current: None,
            water_temperature: 15.0,
            salinity: 35.0,
        }
    }

    /// Check if waves are present
    pub fn has_waves(&self) -> bool {
        self.wave_spectrum.is_some()
    }

    /// Check if wind is present
    pub fn has_wind(&self) -> bool {
        self.wind_conditions.is_some()
    }

    /// Check if current is present
    pub fn has_current(&self) -> bool {
        self.current.is_some()
    }
}

impl AddedResistanceResult {
    /// Create zero added resistance result
    pub fn zero() -> Self {
        Self {
            total_resistance: 0.0,
            mean_added_resistance: 0.0,
            oscillatory_component: 0.0,
            rao_data: None,
            integration_method: "None".to_string(),
        }
    }
}

impl WindResistance {
    /// Create zero wind resistance result
    pub fn zero() -> Self {
        Self {
            longitudinal_force: 0.0,
            lateral_force: 0.0,
            yaw_moment: 0.0,
            relative_wind_speed: 0.0,
            relative_wind_angle: 0.0,
        }
    }
}

impl VesselParameters {
    /// Create default container ship parameters
    pub fn default_container_ship() -> Self {
        Self {
            name: "Generic Container Ship".to_string(),
            vessel_type: "Container Ship".to_string(),
            hull: HullParameters {
                length_overall: 300.0,
                length_between_perpendiculars: 280.0,
                length_waterline: 285.0,
                beam: 40.0,
                draft: 12.0,
                displacement: 52000.0,
                block_coefficient: 0.65,
                midship_coefficient: 0.99,
                waterplane_coefficient: 0.85,
                prismatic_coefficient: 0.66,
                longitudinal_center_buoyancy: 2.0,
                longitudinal_center_flotation: 0.0,
                half_angle_entrance: 20.0,
                stern_type: SternType::Normal,
                bulbous_bow: Some(BulbousBowParameters {
                    bulb_area: 25.0,
                    bulb_center_height: 4.0,
                }),
                transom_area: 0.0,
                hull_efficiency: 1.05,
            },
            propulsion: PropulsionParameters {
                propeller_diameter: 8.5,
                propeller_pitch_ratio: 0.7,
                number_of_propellers: 1,
                propeller_efficiency: 0.65,
                shaft_efficiency: 0.98,
                propeller_type: PropellerType::Conventional,
            },
            superstructure: SuperstructureParameters {
                frontal_area: 600.0,
                lateral_area: 2400.0,
                height_above_waterline: 25.0,
                center_of_effort_height: 15.0,
                drag_coefficient_head: 0.8,
                drag_coefficient_beam: 1.2,
            },
            appendages: vec![
                AppendageParameters {
                    appendage_type: AppendageType::Rudder,
                    area: 80.0,
                    drag_coefficient: 0.03,
                },
            ],
        }
    }
} 