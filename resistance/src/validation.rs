//! Validation and benchmarking module for resistance calculations
//! 
//! This module provides comprehensive validation capabilities including
//! standard benchmark cases and statistical validation methods.

use crate::{
    types::*,
    errors::{Result, ResistanceError},
    ResistanceCalculator,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Validation suite for resistance calculations
#[derive(Debug, Clone)]
pub struct ValidationSuite {
    benchmark_cases: Vec<BenchmarkCase>,
    validation_config: ValidationConfig,
}

/// Configuration for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub tolerance_percentage: f64,
    pub enable_statistical_analysis: bool,
    pub include_uncertainty_analysis: bool,
    pub benchmark_timeout_seconds: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            tolerance_percentage: 10.0, // 10% tolerance
            enable_statistical_analysis: true,
            include_uncertainty_analysis: false,
            benchmark_timeout_seconds: 300, // 5 minutes per benchmark
        }
    }
}

/// Benchmark test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkCase {
    pub name: String,
    pub description: String,
    pub vessel: VesselParameters,
    pub conditions: OperatingConditions,
    pub environment: EnvironmentalConditions,
    pub reference_results: BenchmarkReferenceResults,
    pub source: String,
    pub vessel_category: VesselCategory,
}

/// Reference results for benchmark validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReferenceResults {
    pub total_resistance: f64,           // N
    pub resistance_coefficient: f64,     // CT
    pub effective_power: f64,            // kW
    pub uncertainty: Option<f64>,        // ± percentage
    pub data_source: String,             // Experimental, CFD, etc.
}

/// Vessel category for classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VesselCategory {
    ContainerShip,
    BulkCarrier,
    Tanker,
    Ferry,
    IndonesianVessel,
    ResearchVessel,
    Other(String),
}

/// Comprehensive validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub overall_summary: ValidationSummary,
    pub benchmark_results: Vec<BenchmarkResult>,
    pub statistical_analysis: Option<StatisticalAnalysis>,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Overall validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub pass_rate: f64,
    pub average_error: f64,
    pub maximum_error: f64,
    pub overall_score: f64,
}

/// Individual benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub case_name: String,
    pub vessel_category: VesselCategory,
    pub calculated_results: TotalResistanceResult,
    pub reference_results: BenchmarkReferenceResults,
    pub validation_metrics: ValidationMetrics,
    pub passed: bool,
    pub execution_time_ms: u64,
}

/// Validation metrics for a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    pub relative_error_percentage: f64,
    pub absolute_error: f64,
    pub confidence_score: f64,
    pub method_applicability: f64,
}

/// Statistical analysis of validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysis {
    pub mean_error: f64,
    pub standard_deviation: f64,
    pub correlation_coefficient: f64,
    pub bias: f64,
    pub error_distribution: Vec<f64>,
}

impl ValidationSuite {
    /// Create a new validation suite
    pub fn new() -> Self {
        Self {
            benchmark_cases: Self::load_default_benchmarks(),
            validation_config: ValidationConfig::default(),
        }
    }

    /// Create validation suite with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            benchmark_cases: Self::load_default_benchmarks(),
            validation_config: config,
        }
    }

    /// Run all validation benchmarks
    pub async fn run_all_benchmarks(&self) -> Result<ValidationReport> {
        info!("Starting comprehensive validation suite with {} benchmarks", self.benchmark_cases.len());

        let mut benchmark_results = Vec::new();
        let calculator = ResistanceCalculator::new();

        for (i, case) in self.benchmark_cases.iter().enumerate() {
            info!("Running benchmark {}/{}: {}", i + 1, self.benchmark_cases.len(), case.name);
            
            let start_time = std::time::Instant::now();
            let result = self.run_single_benchmark(&calculator, case).await?;
            let execution_time = start_time.elapsed().as_millis() as u64;

            let mut benchmark_result = result;
            benchmark_result.execution_time_ms = execution_time;
            
            benchmark_results.push(benchmark_result);
        }

        // Generate comprehensive report
        let validation_summary = self.calculate_validation_summary(&benchmark_results);
        let statistical_analysis = if self.validation_config.enable_statistical_analysis {
            Some(self.perform_statistical_analysis(&benchmark_results)?)
        } else {
            None
        };

        let recommendations = self.generate_recommendations(&benchmark_results, &validation_summary);

        info!("Validation completed: {}/{} tests passed ({:.1}% pass rate)", 
              validation_summary.passed_tests, 
              validation_summary.total_tests,
              validation_summary.pass_rate);

        Ok(ValidationReport {
            overall_summary: validation_summary,
            benchmark_results,
            statistical_analysis,
            recommendations,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Run a single benchmark case
    async fn run_single_benchmark(
        &self,
        calculator: &ResistanceCalculator,
        case: &BenchmarkCase,
    ) -> Result<BenchmarkResult> {
        debug!("Executing benchmark: {}", case.name);

        // Calculate resistance using our implementation
        let calculated_results = calculator.calculate_total_resistance(
            &case.vessel,
            &case.conditions,
            &case.environment,
        )?;

        // Calculate validation metrics
        let validation_metrics = self.calculate_validation_metrics(
            &calculated_results,
            &case.reference_results,
        )?;

        // Determine if test passed
        let passed = validation_metrics.relative_error_percentage.abs() <= self.validation_config.tolerance_percentage;

        debug!("Benchmark {} completed: error={:.1}%, passed={}", 
               case.name, validation_metrics.relative_error_percentage, passed);

        Ok(BenchmarkResult {
            case_name: case.name.clone(),
            vessel_category: case.vessel_category.clone(),
            calculated_results,
            reference_results: case.reference_results.clone(),
            validation_metrics,
            passed,
            execution_time_ms: 0, // Will be set by caller
        })
    }

    /// Calculate validation metrics for a result
    fn calculate_validation_metrics(
        &self,
        calculated: &TotalResistanceResult,
        reference: &BenchmarkReferenceResults,
    ) -> Result<ValidationMetrics> {
        let relative_error_percentage = ((calculated.total_resistance - reference.total_resistance) / reference.total_resistance) * 100.0;
        let absolute_error = (calculated.total_resistance - reference.total_resistance).abs();

        Ok(ValidationMetrics {
            relative_error_percentage,
            absolute_error,
            confidence_score: calculated.confidence,
            method_applicability: calculated.confidence, // Simplified
        })
    }

    /// Calculate overall validation summary
    fn calculate_validation_summary(&self, results: &[BenchmarkResult]) -> ValidationSummary {
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        let pass_rate = if total_tests > 0 {
            (passed_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        let errors: Vec<f64> = results.iter()
            .map(|r| r.validation_metrics.relative_error_percentage.abs())
            .collect();

        let average_error = if !errors.is_empty() {
            errors.iter().sum::<f64>() / errors.len() as f64
        } else {
            0.0
        };

        let maximum_error = errors.iter().fold(0.0f64, |acc, &x| acc.max(x));

        // Overall score based on pass rate and average error
        let overall_score = (pass_rate / 100.0) * (1.0 - average_error / 100.0).max(0.0);

        ValidationSummary {
            total_tests,
            passed_tests,
            failed_tests,
            pass_rate,
            average_error,
            maximum_error,
            overall_score,
        }
    }

    /// Perform statistical analysis on validation results
    fn perform_statistical_analysis(&self, results: &[BenchmarkResult]) -> Result<StatisticalAnalysis> {
        let errors: Vec<f64> = results.iter()
            .map(|r| r.validation_metrics.relative_error_percentage)
            .collect();

        if errors.is_empty() {
            return Err(ResistanceError::calculation_error("No data for statistical analysis"));
        }

        let mean_error = errors.iter().sum::<f64>() / errors.len() as f64;
        
        let variance = errors.iter()
            .map(|e| (e - mean_error).powi(2))
            .sum::<f64>() / errors.len() as f64;
        let standard_deviation = variance.sqrt();

        // Simple correlation calculation (calculated vs reference)
        let calculated_values: Vec<f64> = results.iter()
            .map(|r| r.calculated_results.total_resistance)
            .collect();
        let reference_values: Vec<f64> = results.iter()
            .map(|r| r.reference_results.total_resistance)
            .collect();

        let correlation_coefficient = self.calculate_correlation(&calculated_values, &reference_values)?;
        let bias = mean_error; // Simple bias estimate

        Ok(StatisticalAnalysis {
            mean_error,
            standard_deviation,
            correlation_coefficient,
            bias,
            error_distribution: errors,
        })
    }

    /// Calculate correlation coefficient
    fn calculate_correlation(&self, x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() || x.is_empty() {
            return Err(ResistanceError::calculation_error("Invalid data for correlation"));
        }

        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        let numerator: f64 = x.iter().zip(y.iter())
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();

        let sum_sq_x: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();
        let sum_sq_y: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();

        let denominator = (sum_sq_x * sum_sq_y).sqrt();

        if denominator == 0.0 {
            Ok(0.0)
        } else {
            Ok(numerator / denominator)
        }
    }

    /// Generate recommendations based on validation results
    fn generate_recommendations(
        &self,
        results: &[BenchmarkResult],
        summary: &ValidationSummary,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if summary.pass_rate < 80.0 {
            recommendations.push("Overall pass rate is below 80%. Consider reviewing calculation methods.".to_string());
        }

        if summary.average_error > 15.0 {
            recommendations.push("Average error exceeds 15%. Check vessel parameter validation and method applicability.".to_string());
        }

        // Check for systematic errors by vessel category
        let mut category_errors: HashMap<String, Vec<f64>> = HashMap::new();
        for result in results {
            let category = format!("{:?}", result.vessel_category);
            category_errors.entry(category)
                .or_insert_with(Vec::new)
                .push(result.validation_metrics.relative_error_percentage.abs());
        }

        for (category, errors) in category_errors {
            let avg_error = errors.iter().sum::<f64>() / errors.len() as f64;
            if avg_error > 20.0 {
                recommendations.push(format!("High average error ({:.1}%) for {} vessels. Consider specialized calibration.", avg_error, category));
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Validation results are satisfactory. No specific recommendations.".to_string());
        }

        recommendations
    }

    /// Load default benchmark cases
    fn load_default_benchmarks() -> Vec<BenchmarkCase> {
        vec![
            // DTMB 5415 Model
            BenchmarkCase {
                name: "DTMB 5415".to_string(),
                description: "US Navy combatant hull form - standard resistance benchmark".to_string(),
                vessel: Self::create_dtmb5415_vessel(),
                conditions: OperatingConditions {
                    speed_knots: 20.0,
                    draft: 1.158,
                    displacement: 8.424,
                    trim: 0.0,
                    heel_angle: 0.0,
                    water_density: 1025.0,
                    kinematic_viscosity: 1.188e-6,
                },
                environment: EnvironmentalConditions::calm_sea(),
                reference_results: BenchmarkReferenceResults {
                    total_resistance: 850.0, // N (example)
                    resistance_coefficient: 0.004,
                    effective_power: 8.5, // kW
                    uncertainty: Some(3.0), // ±3%
                    data_source: "Experimental - DTMB Tank".to_string(),
                },
                source: "DTMB Model Basin".to_string(),
                vessel_category: VesselCategory::ResearchVessel,
            },
            
            // KCS Container Ship
            BenchmarkCase {
                name: "KCS Container Ship".to_string(),
                description: "KRISO Container Ship - international benchmark".to_string(),
                vessel: Self::create_kcs_vessel(),
                conditions: OperatingConditions {
                    speed_knots: 24.0,
                    draft: 10.8,
                    displacement: 52030.0,
                    trim: 0.0,
                    heel_angle: 0.0,
                    water_density: 1025.0,
                    kinematic_viscosity: 1.188e-6,
                },
                environment: EnvironmentalConditions::calm_sea(),
                reference_results: BenchmarkReferenceResults {
                    total_resistance: 180000.0, // N (example)
                    resistance_coefficient: 0.0035,
                    effective_power: 1800.0, // kW
                    uncertainty: Some(5.0), // ±5%
                    data_source: "CFD/Experimental - KRISO".to_string(),
                },
                source: "KRISO/SIMMAN".to_string(),
                vessel_category: VesselCategory::ContainerShip,
            },

            // Indonesian Ferry
            BenchmarkCase {
                name: "Indonesian Ferry".to_string(),
                description: "Typical Indonesian inter-island ferry".to_string(),
                vessel: Self::create_indonesian_ferry_vessel(),
                conditions: OperatingConditions {
                    speed_knots: 16.0,
                    draft: 3.5,
                    displacement: 1200.0,
                    trim: 0.0,
                    heel_angle: 0.0,
                    water_density: 1025.0,
                    kinematic_viscosity: 1.188e-6,
                },
                environment: EnvironmentalConditions::calm_sea(),
                reference_results: BenchmarkReferenceResults {
                    total_resistance: 25000.0, // N (estimated)
                    resistance_coefficient: 0.0045,
                    effective_power: 200.0, // kW
                    uncertainty: Some(8.0), // ±8%
                    data_source: "Estimated - Indonesian Shipyard Data".to_string(),
                },
                source: "Indonesian Maritime Data".to_string(),
                vessel_category: VesselCategory::IndonesianVessel,
            },
        ]
    }

    /// Create DTMB 5415 vessel parameters
    fn create_dtmb5415_vessel() -> VesselParameters {
        VesselParameters {
            name: "DTMB 5415".to_string(),
            vessel_type: "Research Vessel".to_string(),
            hull: HullParameters {
                length_overall: 5.72,
                length_between_perpendiculars: 5.379,
                length_waterline: 5.379,
                beam: 0.728,
                draft: 1.158,
                displacement: 8.424,
                block_coefficient: 0.506,
                midship_coefficient: 0.974,
                waterplane_coefficient: 0.758,
                prismatic_coefficient: 0.519,
                longitudinal_center_buoyancy: 2.48,
                longitudinal_center_flotation: 2.38,
                half_angle_entrance: 19.3,
                stern_type: SternType::Normal,
                bulbous_bow: Some(BulbousBowParameters {
                    bulb_area: 0.012,
                    bulb_center_height: 0.8,
                }),
                transom_area: 0.0,
                hull_efficiency: 1.0,
            },
            propulsion: PropulsionParameters {
                propeller_diameter: 0.25,
                propeller_pitch_ratio: 1.1,
                number_of_propellers: 1,
                propeller_efficiency: 0.7,
                shaft_efficiency: 0.98,
                propeller_type: PropellerType::Conventional,
            },
            superstructure: SuperstructureParameters {
                frontal_area: 2.0,
                lateral_area: 8.0,
                height_above_waterline: 2.0,
                center_of_effort_height: 1.5,
                drag_coefficient_head: 0.8,
                drag_coefficient_beam: 1.2,
            },
            appendages: vec![],
        }
    }

    /// Create KCS vessel parameters
    fn create_kcs_vessel() -> VesselParameters {
        VesselParameters {
            name: "KCS Container Ship".to_string(),
            vessel_type: "Container Ship".to_string(),
            hull: HullParameters {
                length_overall: 232.5,
                length_between_perpendiculars: 230.0,
                length_waterline: 230.0,
                beam: 32.2,
                draft: 10.8,
                displacement: 52030.0,
                block_coefficient: 0.651,
                midship_coefficient: 0.985,
                waterplane_coefficient: 0.820,
                prismatic_coefficient: 0.663,
                longitudinal_center_buoyancy: 1.48,
                longitudinal_center_flotation: 0.0,
                half_angle_entrance: 15.0,
                stern_type: SternType::Normal,
                bulbous_bow: Some(BulbousBowParameters {
                    bulb_area: 6.0,
                    bulb_center_height: 2.0,
                }),
                transom_area: 0.0,
                hull_efficiency: 1.0,
            },
            propulsion: PropulsionParameters {
                propeller_diameter: 7.9,
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

    /// Create Indonesian ferry vessel parameters
    fn create_indonesian_ferry_vessel() -> VesselParameters {
        VesselParameters {
            name: "Indonesian Ferry".to_string(),
            vessel_type: "Ferry".to_string(),
            hull: HullParameters {
                length_overall: 65.0,
                length_between_perpendiculars: 60.0,
                length_waterline: 62.0,
                beam: 12.0,
                draft: 3.5,
                displacement: 1200.0,
                block_coefficient: 0.55,
                midship_coefficient: 0.95,
                waterplane_coefficient: 0.85,
                prismatic_coefficient: 0.58,
                longitudinal_center_buoyancy: 1.0,
                longitudinal_center_flotation: 0.0,
                half_angle_entrance: 20.0,
                stern_type: SternType::Normal,
                bulbous_bow: None,
                transom_area: 15.0,
                hull_efficiency: 1.0,
            },
            propulsion: PropulsionParameters {
                propeller_diameter: 2.2,
                propeller_pitch_ratio: 0.8,
                number_of_propellers: 2,
                propeller_efficiency: 0.6,
                shaft_efficiency: 0.95,
                propeller_type: PropellerType::Conventional,
            },
            superstructure: SuperstructureParameters {
                frontal_area: 120.0,
                lateral_area: 480.0,
                height_above_waterline: 8.0,
                center_of_effort_height: 5.0,
                drag_coefficient_head: 0.9,
                drag_coefficient_beam: 1.3,
            },
            appendages: vec![
                AppendageParameters {
                    appendage_type: AppendageType::Rudder,
                    area: 8.0,
                    drag_coefficient: 0.03,
                },
            ],
        }
    }
}

impl Default for ValidationSuite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_suite_creation() {
        let suite = ValidationSuite::new();
        assert!(suite.benchmark_cases.len() > 0);
        assert_eq!(suite.validation_config.tolerance_percentage, 10.0);
    }

    #[test]
    fn test_benchmark_case_creation() {
        let benchmarks = ValidationSuite::load_default_benchmarks();
        assert!(benchmarks.len() >= 3);
        
        let dtmb_case = benchmarks.iter().find(|c| c.name == "DTMB 5415");
        assert!(dtmb_case.is_some());
        
        let dtmb = dtmb_case.unwrap();
        assert_eq!(dtmb.vessel.name, "DTMB 5415");
        assert!(dtmb.reference_results.total_resistance > 0.0);
    }

    #[test]
    fn test_validation_metrics_calculation() {
        let suite = ValidationSuite::new();
        
        let calculated = TotalResistanceResult {
            total_resistance: 1000.0,
            breakdown: ResistanceBreakdown {
                calm_water_resistance: HoltropMennenResult {
                    total_resistance: 900.0,
                    frictional_resistance: 500.0,
                    appendage_resistance: 50.0,
                    wave_resistance: 300.0,
                    bulbous_bow_resistance: 30.0,
                    transom_resistance: 20.0,
                    model_ship_correlation: 100.0,
                    resistance_coefficient: 0.004,
                    effective_power: 18.0,
                    validation_flags: ValidationFlags {
                        length_range_valid: true,
                        speed_range_valid: true,
                        cb_range_valid: true,
                        cp_range_valid: true,
                        overall_validity: 0.9,
                    },
                },
                added_resistance: AddedResistanceResult::zero(),
                wind_resistance: WindResistance::zero(),
            },
            power_requirements: PowerRequirements {
                effective_power: 18000.0,
                brake_power: 20000.0,
                propeller_efficiency: 0.65,
            },
            confidence: 0.85,
            calculation_metadata: CalculationMetadata {
                method: "Test".to_string(),
                vessel_type: "Test".to_string(),
                conditions: OperatingConditions::default(),
                environment: EnvironmentalConditions::calm_sea(),
                timestamp: chrono::Utc::now(),
            },
        };
        
        let reference = BenchmarkReferenceResults {
            total_resistance: 900.0,
            resistance_coefficient: 0.004,
            effective_power: 18.0,
            uncertainty: Some(5.0),
            data_source: "Test".to_string(),
        };
        
        let metrics = suite.calculate_validation_metrics(&calculated, &reference);
        assert!(metrics.is_ok());
        
        let validation_metrics = metrics.unwrap();
        // Should be about 11.1% error (1000-900)/900*100
        assert!((validation_metrics.relative_error_percentage - 11.111).abs() < 0.1);
    }

    #[test]
    fn test_correlation_calculation() {
        let suite = ValidationSuite::new();
        
        // Perfect correlation
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![2.0, 4.0, 6.0, 8.0];
        let corr = suite.calculate_correlation(&x, &y);
        assert!(corr.is_ok());
        assert!((corr.unwrap() - 1.0).abs() < 1e-10);
        
        // No correlation
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![1.0, 1.0, 1.0, 1.0];
        let corr = suite.calculate_correlation(&x, &y);
        assert!(corr.is_ok());
        assert_eq!(corr.unwrap(), 0.0);
    }
} 