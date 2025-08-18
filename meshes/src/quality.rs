use crate::mesh::{Mesh, Panel};
use crate::Point;
use nalgebra::{Vector3, Point3 as NalgebraPoint3};
use std::collections::HashMap;

/// Comprehensive quality metrics for mesh assessment
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub aspect_ratio: f64,
    pub skewness: f64,
    pub orthogonality: f64,
    pub volume_ratio: f64,
    pub min_angle: f64,
    pub max_angle: f64,
    pub warping: f64,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            aspect_ratio: 3.0,    // Maximum acceptable aspect ratio
            skewness: 0.8,        // Maximum acceptable skewness (0-1)
            orthogonality: 0.1,   // Minimum acceptable orthogonality
            volume_ratio: 0.1,    // Minimum acceptable volume ratio
            min_angle: 20.0,      // Minimum angle in degrees
            max_angle: 160.0,     // Maximum angle in degrees
            warping: 0.1,         // Maximum warping factor
        }
    }
}

/// Quality assessment report for entire mesh
#[derive(Debug, Clone)]
pub struct QualityReport {
    pub overall_score: f64,
    pub element_count: usize,
    pub poor_elements: Vec<usize>,
    pub excellent_elements: Vec<usize>,
    pub metrics: HashMap<usize, ElementQuality>,
    pub statistics: QualityStatistics,
    pub recommendations: Vec<String>,
}

/// Statistical summary of mesh quality
#[derive(Debug, Clone)]
pub struct QualityStatistics {
    pub aspect_ratio: StatisticalSummary,
    pub skewness: StatisticalSummary,
    pub orthogonality: StatisticalSummary,
    pub angles: AngleStatistics,
    pub quality_distribution: QualityDistribution,
}

/// Statistical summary for a metric
#[derive(Debug, Clone)]
pub struct StatisticalSummary {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub percentile_95: f64,
}

/// Angle statistics for mesh elements
#[derive(Debug, Clone)]
pub struct AngleStatistics {
    pub min_angle: f64,
    pub max_angle: f64,
    pub mean_angle: f64,
    pub acute_count: usize,
    pub obtuse_count: usize,
    pub right_count: usize,
}

/// Quality distribution breakdown
#[derive(Debug, Clone)]
pub struct QualityDistribution {
    pub excellent: usize,  // Quality > 0.8
    pub good: usize,       // Quality 0.6-0.8
    pub fair: usize,       // Quality 0.4-0.6
    pub poor: usize,       // Quality 0.2-0.4
    pub very_poor: usize,  // Quality < 0.2
}

/// Quality metrics for individual elements
#[derive(Debug, Clone)]
pub struct ElementQuality {
    pub aspect_ratio: f64,
    pub skewness: f64,
    pub orthogonality: f64,
    pub warping: f64,
    pub min_angle: f64,
    pub max_angle: f64,
    pub quality_score: f64,
    pub quality_grade: QualityGrade,
}

/// Quality grade classification
#[derive(Debug, Clone, PartialEq)]
pub enum QualityGrade {
    Excellent,  // > 0.8
    Good,       // 0.6-0.8
    Fair,       // 0.4-0.6
    Poor,       // 0.2-0.4
    VeryPoor,   // < 0.2
}

impl QualityGrade {
    pub fn from_score(score: f64) -> Self {
        if score > 0.8 {
            QualityGrade::Excellent
        } else if score > 0.6 {
            QualityGrade::Good
        } else if score > 0.4 {
            QualityGrade::Fair
        } else if score > 0.2 {
            QualityGrade::Poor
        } else {
            QualityGrade::VeryPoor
        }
    }
}

impl QualityMetrics {
    /// Calculate mesh quality for each element
    pub fn calculate_element_quality(&self, panel: &Panel) -> Result<ElementQuality, Box<dyn std::error::Error>> {
        let aspect_ratio = self.calculate_aspect_ratio(panel)?;
        let skewness = self.calculate_skewness(panel)?;
        let orthogonality = self.calculate_orthogonality(panel)?;
        let warping = self.calculate_warping(panel)?;
        let (min_angle, max_angle) = self.calculate_angles(panel)?;
        
        // Calculate overall quality score
        let quality_score = self.calculate_quality_score(
            aspect_ratio, skewness, orthogonality, warping, min_angle, max_angle
        );
        
        let quality_grade = QualityGrade::from_score(quality_score);
        
        Ok(ElementQuality {
            aspect_ratio,
            skewness,
            orthogonality,
            warping,
            min_angle,
            max_angle,
            quality_score,
            quality_grade,
        })
    }

    /// Overall mesh quality assessment
    pub fn assess_mesh_quality(&self, mesh: &mut Mesh) -> Result<QualityReport, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();
        let mut poor_elements = Vec::new();
        let mut excellent_elements = Vec::new();
        let mut quality_scores = Vec::new();
        
        // Calculate quality for each element
        for (i, panel) in mesh.panels()?.iter().enumerate() {
            let quality = self.calculate_element_quality(panel)?;
            
            match quality.quality_grade {
                QualityGrade::Excellent => excellent_elements.push(i),
                QualityGrade::Poor | QualityGrade::VeryPoor => poor_elements.push(i),
                _ => {}
            }
            
            quality_scores.push(quality.quality_score);
            metrics.insert(i, quality);
        }
        
        // Calculate overall statistics
        let overall_score = if !quality_scores.is_empty() {
            quality_scores.iter().sum::<f64>() / quality_scores.len() as f64
        } else {
            0.0
        };
        
        let statistics = self.calculate_statistics(mesh, &metrics)?;
        
        Ok(QualityReport {
            overall_score,
            element_count: mesh.panels()?.len(),
            poor_elements: poor_elements.clone(),
            excellent_elements: Vec::new(), // TODO: Calculate excellent elements
            metrics,
            statistics: statistics.clone(),
            recommendations: self.generate_recommendations(&statistics, &poor_elements),
        })
    }

    /// Identify poor-quality elements for refinement
    pub fn identify_refinement_candidates(&self, mesh: &Mesh) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        let mut mesh_mut = mesh.clone();
        let quality_report = self.assess_mesh_quality(&mut mesh_mut)?;
        Ok(quality_report.poor_elements)
    }

    /// Calculate aspect ratio of a panel
    fn calculate_aspect_ratio(&self, panel: &Panel) -> Result<f64, Box<dyn std::error::Error>> {
        if panel.vertices.len() < 3 {
            return Err("Panel must have at least 3 vertices".into());
        }
        
        let mut edge_lengths = Vec::new();
        
        for i in 0..panel.vertices.len() {
            let j = (i + 1) % panel.vertices.len();
            let edge_length = (panel.vertices[i] - panel.vertices[j]).norm();
            edge_lengths.push(edge_length);
        }
        
        let min_edge = edge_lengths.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_edge = edge_lengths.iter().cloned().fold(0.0, f64::max);
        
        if min_edge > 1e-12 {
            Ok(max_edge / min_edge)
        } else {
            Ok(f64::INFINITY)
        }
    }

    /// Calculate skewness of a panel
    fn calculate_skewness(&self, panel: &Panel) -> Result<f64, Box<dyn std::error::Error>> {
        if panel.vertices.len() == 3 {
            // For triangles, calculate deviation from equilateral
            return self.calculate_triangle_skewness(panel);
        } else if panel.vertices.len() == 4 {
            // For quads, calculate deviation from rectangle
            return self.calculate_quad_skewness(panel);
        }
        
        Ok(0.0) // For other shapes, return 0
    }

    /// Calculate triangle skewness
    fn calculate_triangle_skewness(&self, panel: &Panel) -> Result<f64, Box<dyn std::error::Error>> {
        if panel.vertices.len() != 3 {
            return Err("Expected triangle".into());
        }
        
        let v0 = panel.vertices[0];
        let v1 = panel.vertices[1];
        let v2 = panel.vertices[2];
        
        // Calculate edge lengths
        let a = (v1 - v2).norm();
        let b = (v2 - v0).norm();
        let c = (v0 - v1).norm();
        
        // Ideal equilateral triangle would have all edges equal
        let mean_edge = (a + b + c) / 3.0;
        
        if mean_edge > 1e-12 {
            let deviation = ((a - mean_edge).abs() + (b - mean_edge).abs() + (c - mean_edge).abs()) / (3.0 * mean_edge);
            Ok(deviation)
        } else {
            Ok(1.0)
        }
    }

    /// Calculate quadrilateral skewness
    fn calculate_quad_skewness(&self, panel: &Panel) -> Result<f64, Box<dyn std::error::Error>> {
        // Panel struct only supports triangular panels
        // For now, return 0 (no skewness defined for triangles)
        Ok(0.0)
    }

    /// Calculate orthogonality of a panel
    fn calculate_orthogonality(&self, panel: &Panel) -> Result<f64, Box<dyn std::error::Error>> {
        let vertices = panel.vertices();
        if vertices.len() < 3 {
            return Err("Panel must have at least 3 vertices".into());
        }
        
        // Calculate normal vector
        let v1 = vertices[1] - vertices[0];
        let v2 = vertices[2] - vertices[0];
        let normal = v1.cross(&v2);
        
        if normal.norm() < 1e-12 {
            return Ok(0.0);
        }
        
        let normal = normal.normalize();
        
        // Check how planar the panel is
        let mut max_deviation: f64 = 0.0;
        let center = panel.center();
        
        for vertex in vertices {
            let to_vertex = vertex - center;
            let deviation = to_vertex.dot(&normal).abs();
            max_deviation = max_deviation.max(deviation);
        }
        
        // Orthogonality is inverse of deviation
        let diagonal_length = self.calculate_diagonal_length(panel)?;
        if diagonal_length > 1e-12 {
            Ok(1.0 - (max_deviation / diagonal_length).min(1.0))
        } else {
            Ok(1.0)
        }
    }

    /// Calculate warping factor for a panel
    fn calculate_warping(&self, panel: &Panel) -> Result<f64, Box<dyn std::error::Error>> {
        let vertices = panel.vertices();
        if vertices.len() <= 3 {
            return Ok(0.0); // Triangles cannot warp
        }
        
        // For quadrilaterals, calculate warping
        if vertices.len() == 4 {
            return self.calculate_quad_warping(panel);
        }
        
        Ok(0.0)
    }

    /// Calculate warping for quadrilateral
    fn calculate_quad_warping(&self, panel: &Panel) -> Result<f64, Box<dyn std::error::Error>> {
        // Panel struct only supports triangular panels
        // Triangles cannot warp, so return 0
        Ok(0.0)
    }

    /// Calculate minimum and maximum angles in a panel
    fn calculate_angles(&self, panel: &Panel) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        let vertices = panel.vertices();
        if vertices.len() < 3 {
            return Err("Panel must have at least 3 vertices".into());
        }
        
        let mut angles = Vec::new();
        
        for i in 0..vertices.len() {
            let prev = (i + vertices.len() - 1) % vertices.len();
            let next = (i + 1) % vertices.len();
            
            let v1 = (vertices[prev] - vertices[i]).normalize();
            let v2 = (vertices[next] - vertices[i]).normalize();
            
            let dot_product = v1.dot(&v2).clamp(-1.0, 1.0);
            let angle = dot_product.acos() * 180.0 / std::f64::consts::PI;
            angles.push(angle);
        }
        
        let min_angle = angles.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_angle = angles.iter().cloned().fold(0.0, f64::max);
        
        Ok((min_angle, max_angle))
    }

    /// Calculate overall quality score
    fn calculate_quality_score(&self, aspect_ratio: f64, skewness: f64, orthogonality: f64, 
                              warping: f64, min_angle: f64, max_angle: f64) -> f64 {
        // Aspect ratio score (1.0 is perfect, decreases with higher ratios)
        let aspect_score = (1.0 / (1.0 + (aspect_ratio - 1.0) / self.aspect_ratio)).min(1.0);
        
        // Skewness score (0 is perfect, 1 is worst)
        let skewness_score = (1.0 - skewness.min(1.0)).max(0.0);
        
        // Orthogonality score (1.0 is perfect)
        let orthogonality_score = orthogonality;
        
        // Warping score (0 is perfect, 1 is worst)
        let warping_score = (1.0 - warping.min(1.0)).max(0.0);
        
        // Angle score
        let angle_score = if min_angle > self.min_angle && max_angle < self.max_angle {
            1.0
        } else {
            let min_penalty = if min_angle < self.min_angle { 
                (self.min_angle - min_angle) / self.min_angle 
            } else { 0.0 };
            let max_penalty = if max_angle > self.max_angle { 
                (max_angle - self.max_angle) / (180.0 - self.max_angle) 
            } else { 0.0 };
            (1.0 - min_penalty - max_penalty).max(0.0)
        };
        
        // Weighted average of all scores
        let weights = [0.25, 0.2, 0.2, 0.15, 0.2]; // Aspect, skewness, orthogonality, warping, angles
        let scores = [aspect_score, skewness_score, orthogonality_score, warping_score, angle_score];
        
        weights.iter().zip(scores.iter()).map(|(w, s)| w * s).sum()
    }

    /// Calculate diagonal length for reference scaling
    fn calculate_diagonal_length(&self, panel: &Panel) -> Result<f64, Box<dyn std::error::Error>> {
        let vertices = panel.vertices();
        if vertices.is_empty() {
            return Ok(0.0);
        }
        
        let mut max_distance: f64 = 0.0;
        
        for i in 0..vertices.len() {
            for j in (i + 1)..vertices.len() {
                let distance = (vertices[i] - vertices[j]).norm();
                max_distance = max_distance.max(distance);
            }
        }
        
        Ok(max_distance)
    }

    /// Calculate comprehensive statistics
    fn calculate_statistics(&self, _mesh: &Mesh, metrics: &HashMap<usize, ElementQuality>) 
                           -> Result<QualityStatistics, Box<dyn std::error::Error>> {
        if metrics.is_empty() {
            return Err("No metrics available".into());
        }
        
        let mut aspect_ratios = Vec::new();
        let mut skewness_values = Vec::new();
        let mut orthogonality_values = Vec::new();
        let mut min_angles = Vec::new();
        let mut max_angles = Vec::new();
        let mut quality_scores = Vec::new();
        
        for quality in metrics.values() {
            aspect_ratios.push(quality.aspect_ratio);
            skewness_values.push(quality.skewness);
            orthogonality_values.push(quality.orthogonality);
            min_angles.push(quality.min_angle);
            max_angles.push(quality.max_angle);
            quality_scores.push(quality.quality_score);
        }
        
        let aspect_ratio_stats = self.calculate_statistical_summary(&aspect_ratios);
        let skewness_stats = self.calculate_statistical_summary(&skewness_values);
        let orthogonality_stats = self.calculate_statistical_summary(&orthogonality_values);
        
        let angles = self.calculate_angle_statistics(&min_angles, &max_angles);
        let quality_distribution = self.calculate_quality_distribution(&quality_scores);
        
        Ok(QualityStatistics {
            aspect_ratio: aspect_ratio_stats,
            skewness: skewness_stats,
            orthogonality: orthogonality_stats,
            angles,
            quality_distribution,
        })
    }

    /// Calculate statistical summary for a dataset
    fn calculate_statistical_summary(&self, data: &[f64]) -> StatisticalSummary {
        if data.is_empty() {
            return StatisticalSummary {
                min: 0.0, max: 0.0, mean: 0.0, median: 0.0, std_dev: 0.0, percentile_95: 0.0
            };
        }
        
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min = sorted_data[0];
        let max = sorted_data[sorted_data.len() - 1];
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        
        let median_idx = sorted_data.len() / 2;
        let median = if sorted_data.len() % 2 == 0 {
            (sorted_data[median_idx - 1] + sorted_data[median_idx]) / 2.0
        } else {
            sorted_data[median_idx]
        };
        
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();
        
        let percentile_95_idx = ((sorted_data.len() as f64) * 0.95) as usize;
        let percentile_95 = sorted_data[percentile_95_idx.min(sorted_data.len() - 1)];
        
        StatisticalSummary {
            min, max, mean, median, std_dev, percentile_95
        }
    }

    /// Calculate angle statistics
    fn calculate_angle_statistics(&self, min_angles: &[f64], max_angles: &[f64]) -> AngleStatistics {
        if min_angles.is_empty() || max_angles.is_empty() {
            return AngleStatistics {
                min_angle: 0.0, max_angle: 0.0, mean_angle: 0.0,
                acute_count: 0, obtuse_count: 0, right_count: 0
            };
        }
        
        let min_angle = min_angles.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_angle = max_angles.iter().cloned().fold(0.0, f64::max);
        
        let all_angles: Vec<f64> = min_angles.iter().chain(max_angles.iter()).cloned().collect();
        let mean_angle = all_angles.iter().sum::<f64>() / all_angles.len() as f64;
        
        let mut acute_count = 0;
        let mut obtuse_count = 0;
        let mut right_count = 0;
        
        for &angle in &all_angles {
            if (angle - 90.0).abs() < 5.0 {
                right_count += 1;
            } else if angle < 90.0 {
                acute_count += 1;
            } else {
                obtuse_count += 1;
            }
        }
        
        AngleStatistics {
            min_angle, max_angle, mean_angle,
            acute_count, obtuse_count, right_count
        }
    }

    /// Calculate quality distribution
    fn calculate_quality_distribution(&self, quality_scores: &[f64]) -> QualityDistribution {
        let mut excellent = 0;
        let mut good = 0;
        let mut fair = 0;
        let mut poor = 0;
        let mut very_poor = 0;
        
        for &score in quality_scores {
            match QualityGrade::from_score(score) {
                QualityGrade::Excellent => excellent += 1,
                QualityGrade::Good => good += 1,
                QualityGrade::Fair => fair += 1,
                QualityGrade::Poor => poor += 1,
                QualityGrade::VeryPoor => very_poor += 1,
            }
        }
        
        QualityDistribution {
            excellent, good, fair, poor, very_poor
        }
    }

    /// Generate improvement recommendations
    fn generate_recommendations(&self, statistics: &QualityStatistics, poor_elements: &[usize]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Overall recommendations
        if !poor_elements.is_empty() {
            recommendations.push(format!("{} elements have poor quality and need attention", poor_elements.len()));
        }
        
        // Specific recommendations based on statistics
        if statistics.aspect_ratio.mean > self.aspect_ratio {
            recommendations.push("High aspect ratios detected. Consider mesh refinement or element reshaping".to_string());
        }
        
        if statistics.skewness.mean > self.skewness {
            recommendations.push("High skewness detected. Apply mesh smoothing or regeneration".to_string());
        }
        
        if statistics.orthogonality.mean < self.orthogonality {
            recommendations.push("Poor orthogonality detected. Improve mesh alignment".to_string());
        }
        
        if statistics.angles.min_angle < self.min_angle {
            recommendations.push(format!("Minimum angle ({:.1}°) is too small. Minimum should be >{:.1}°", 
                                        statistics.angles.min_angle, self.min_angle));
        }
        
        if statistics.angles.max_angle > self.max_angle {
            recommendations.push(format!("Maximum angle ({:.1}°) is too large. Maximum should be <{:.1}°", 
                                        statistics.angles.max_angle, self.max_angle));
        }
        
        // Quality distribution recommendations
        let total_elements = statistics.quality_distribution.excellent + 
                           statistics.quality_distribution.good + 
                           statistics.quality_distribution.fair + 
                           statistics.quality_distribution.poor + 
                           statistics.quality_distribution.very_poor;
        
        if total_elements > 0 {
            let poor_percentage = ((statistics.quality_distribution.poor + 
                                   statistics.quality_distribution.very_poor) as f64 / total_elements as f64) * 100.0;
            
            if poor_percentage > 20.0 {
                recommendations.push("More than 20% of elements have poor quality. Consider global mesh regeneration".to_string());
            } else if poor_percentage > 10.0 {
                recommendations.push("More than 10% of elements have poor quality. Apply targeted refinement".to_string());
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("Mesh quality is acceptable. Consider minor optimizations for better performance".to_string());
        }
        
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Point3;

    #[test]
    fn test_quality_metrics_creation() {
        let metrics = QualityMetrics::default();
        assert!(metrics.aspect_ratio > 0.0);
        assert!(metrics.skewness > 0.0);
    }

    #[test]
    fn test_triangle_quality_calculation() {
        let metrics = QualityMetrics::default();
        
        // Create an equilateral triangle
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, (3.0_f64).sqrt() / 2.0, 0.0),
        ];
        let panel = Panel::new(vertices);
        
        let quality = metrics.calculate_element_quality(&panel).unwrap();
        assert!(quality.quality_score > 0.7); // Should be high quality
        assert_eq!(quality.quality_grade, QualityGrade::Good);
    }

    #[test]
    fn test_degenerate_triangle() {
        let metrics = QualityMetrics::default();
        
        // Create a very thin triangle (poor quality)
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(5.0, 0.01, 0.0),
        ];
        let panel = Panel::new(vertices);
        
        let quality = metrics.calculate_element_quality(&panel).unwrap();
        assert!(quality.aspect_ratio > 10.0);
        assert!(quality.quality_score < 0.5);
    }

    #[test]
    fn test_quality_grade_classification() {
        assert_eq!(QualityGrade::from_score(0.9), QualityGrade::Excellent);
        assert_eq!(QualityGrade::from_score(0.7), QualityGrade::Good);
        assert_eq!(QualityGrade::from_score(0.5), QualityGrade::Fair);
        assert_eq!(QualityGrade::from_score(0.3), QualityGrade::Poor);
        assert_eq!(QualityGrade::from_score(0.1), QualityGrade::VeryPoor);
    }

    #[test]
    fn test_statistical_summary() {
        let metrics = QualityMetrics::default();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let summary = metrics.calculate_statistical_summary(&data);
        
        assert_eq!(summary.min, 1.0);
        assert_eq!(summary.max, 5.0);
        assert_eq!(summary.mean, 3.0);
        assert_eq!(summary.median, 3.0);
    }
} 