//! Post-processing analysis tools

use super::*;
use std::time::Instant;
use std::f64::consts::PI;

/// RAO analyzer
pub struct RAOAnalyzer {
    config: AnalysisConfig,
}

impl RAOAnalyzer {
    /// Create a new RAO analyzer
    pub fn new() -> Self {
        Self {
            config: AnalysisConfig::default(),
        }
    }
    
    /// Create a new RAO analyzer with configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        Self { config }
    }
    
    /// Calculate RAOs
    pub fn calculate_raos(&self, bem_results: &wavecore_bem::BEMResult) -> Result<RAOData> {
        let start_time = Instant::now();
        
        // Extract frequency and direction ranges
        let frequencies = self.generate_frequencies()?;
        let directions = self.generate_directions()?;
        
        // Initialize RAO values
        let mut rao_values = Vec::new();
        
        for freq in &frequencies {
            let mut freq_raos = Vec::new();
            
            for direction in &directions {
                let mut direction_raos = Vec::new();
                
                // Calculate RAO for each DOF
                for dof_idx in 0..6 {
                    let rao = self.calculate_single_rao(bem_results, *freq, *direction, dof_idx)?;
                    direction_raos.push(rao);
                }
                
                freq_raos.push(direction_raos);
            }
            
            rao_values.push(freq_raos);
        }
        
        let processing_time = start_time.elapsed().as_secs_f64();
        
        Ok(RAOData {
            frequencies,
            directions,
            rao_values,
            dofs: vec!["Surge".to_string(), "Sway".to_string(), "Heave".to_string(),
                      "Roll".to_string(), "Pitch".to_string(), "Yaw".to_string()],
        })
    }
    
    /// Calculate single RAO value
    fn calculate_single_rao(&self, bem_results: &wavecore_bem::BEMResult, frequency: f64, direction: f64, dof_idx: usize) -> Result<Complex64> {
        // Extract relevant BEM results
        let added_mass = self.get_added_mass(bem_results, frequency, dof_idx)?;
        let damping = self.get_damping(bem_results, frequency, dof_idx)?;
        let excitation = self.get_excitation_force(bem_results, frequency, direction, dof_idx)?;
        
        // Get body properties
        let mass = self.get_body_mass(bem_results, dof_idx)?;
        let stiffness = self.get_hydrostatic_stiffness(bem_results, dof_idx)?;
        
        // Calculate RAO using frequency domain analysis
        let omega = frequency;
        let omega_squared = omega * omega;
        
        // Complex impedance: Z = -omega^2 * (M + A) + i*omega*B + C
        let impedance = Complex64::new(
            -omega_squared * (mass + added_mass.re) + stiffness,
            omega * damping
        );
        
        // RAO = F / Z
        let rao = excitation / impedance;
        
        Ok(rao)
    }
    
    /// Generate frequency range
    fn generate_frequencies(&self) -> Result<Vec<f64>> {
        let (min_freq, max_freq) = self.config.frequency_range
            .unwrap_or((0.1, 2.0)); // Default range: 0.1 to 2.0 rad/s
        
        let frequencies: Vec<f64> = (0..self.config.num_frequencies)
            .map(|i| {
                let t = i as f64 / (self.config.num_frequencies - 1) as f64;
                min_freq + t * (max_freq - min_freq)
            })
            .collect();
        
        Ok(frequencies)
    }
    
    /// Generate direction range
    fn generate_directions(&self) -> Result<Vec<f64>> {
        let (min_dir, max_dir) = self.config.direction_range
            .unwrap_or((0.0, 2.0 * PI)); // Default range: 0 to 2π
        
        let directions: Vec<f64> = (0..self.config.num_directions)
            .map(|i| {
                let t = i as f64 / (self.config.num_directions - 1) as f64;
                min_dir + t * (max_dir - min_dir)
            })
            .collect();
        
        Ok(directions)
    }
    
    /// Get added mass from BEM results
    fn get_added_mass(&self, _bem_results: &wavecore_bem::BEMResult, _frequency: f64, _dof_idx: usize) -> Result<Complex64> {
        // Placeholder - would extract from actual BEM results
        Ok(Complex64::new(1000.0, 0.0))
    }
    
    /// Get damping from BEM results
    fn get_damping(&self, _bem_results: &wavecore_bem::BEMResult, _frequency: f64, _dof_idx: usize) -> Result<f64> {
        // Placeholder - would extract from actual BEM results
        Ok(50.0)
    }
    
    /// Get excitation force from BEM results
    fn get_excitation_force(&self, _bem_results: &wavecore_bem::BEMResult, _frequency: f64, _direction: f64, _dof_idx: usize) -> Result<Complex64> {
        // Placeholder - would extract from actual BEM results
        Ok(Complex64::new(1000.0, 500.0))
    }
    
    /// Get body mass
    fn get_body_mass(&self, _bem_results: &wavecore_bem::BEMResult, _dof_idx: usize) -> Result<f64> {
        // Placeholder - would extract from body properties
        Ok(10000.0)
    }
    
    /// Get hydrostatic stiffness
    fn get_hydrostatic_stiffness(&self, _bem_results: &wavecore_bem::BEMResult, _dof_idx: usize) -> Result<f64> {
        // Placeholder - would extract from body properties
        Ok(100000.0)
    }
}

/// Kochin analyzer
pub struct KochinAnalyzer {
    config: AnalysisConfig,
}

impl KochinAnalyzer {
    /// Create a new Kochin analyzer
    pub fn new() -> Self {
        Self {
            config: AnalysisConfig::default(),
        }
    }
    
    /// Create a new Kochin analyzer with configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        Self { config }
    }
    
    /// Calculate Kochin functions
    pub fn calculate_kochin(&self, bem_results: &wavecore_bem::BEMResult) -> Result<KochinData> {
        let start_time = Instant::now();
        
        // Extract frequency and direction ranges
        let frequencies = self.generate_frequencies()?;
        let directions = self.generate_directions()?;
        
        // Initialize Kochin values
        let mut kochin_values = Vec::new();
        
        for freq in &frequencies {
            let mut freq_kochin = Vec::new();
            
            for direction in &directions {
                let kochin = self.calculate_single_kochin(bem_results, *freq, *direction)?;
                freq_kochin.push(kochin);
            }
            
            kochin_values.push(freq_kochin);
        }
        
        let processing_time = start_time.elapsed().as_secs_f64();
        
        Ok(KochinData {
            frequencies,
            directions,
            kochin_values,
            far_field_distance: 100.0,
        })
    }
    
    /// Calculate single Kochin function value
    fn calculate_single_kochin(&self, bem_results: &wavecore_bem::BEMResult, frequency: f64, direction: f64) -> Result<Complex64> {
        // Extract source strength from BEM results
        let source_strength = self.get_source_strength(bem_results, frequency, direction)?;
        
        // Calculate Kochin function using far-field approximation
        let k = frequency * frequency / 9.81; // Wave number
        let r = self.config.tolerance; // Far-field distance
        
        // Kochin function: H(theta) = sum(q_i * exp(-i*k*r*cos(theta-theta_i)))
        let mut kochin = Complex64::new(0.0, 0.0);
        
        for (i, &strength) in source_strength.iter().enumerate() {
            let theta_i = 2.0 * PI * i as f64 / source_strength.len() as f64;
            let phase = -k * r * (direction - theta_i).cos();
            kochin += strength * Complex64::new(phase.cos(), phase.sin());
        }
        
        Ok(kochin)
    }
    
    /// Generate frequency range for Kochin analysis
    fn generate_frequencies(&self) -> Result<Vec<f64>> {
        let (min_freq, max_freq) = self.config.frequency_range
            .unwrap_or((0.1, 2.0));
        
        let frequencies: Vec<f64> = (0..self.config.num_frequencies)
            .map(|i| {
                let t = i as f64 / (self.config.num_frequencies - 1) as f64;
                min_freq + t * (max_freq - min_freq)
            })
            .collect();
        
        Ok(frequencies)
    }
    
    /// Generate direction range for Kochin analysis
    fn generate_directions(&self) -> Result<Vec<f64>> {
        let (min_dir, max_dir) = self.config.direction_range
            .unwrap_or((0.0, 2.0 * PI));
        
        let directions: Vec<f64> = (0..self.config.num_directions)
            .map(|i| {
                let t = i as f64 / (self.config.num_directions - 1) as f64;
                min_dir + t * (max_dir - min_dir)
            })
            .collect();
        
        Ok(directions)
    }
    
    /// Get source strength from BEM results
    fn get_source_strength(&self, _bem_results: &wavecore_bem::BEMResult, _frequency: f64, _direction: f64) -> Result<Vec<Complex64>> {
        // Placeholder - would extract from actual BEM results
        let num_panels = 100;
        let mut source_strength = Vec::new();
        
        for i in 0..num_panels {
            let phase = 2.0 * PI * i as f64 / num_panels as f64;
            source_strength.push(Complex64::new(phase.cos(), phase.sin()));
        }
        
        Ok(source_strength)
    }
}

/// Free surface analyzer
pub struct FreeSurfaceAnalyzer {
    config: AnalysisConfig,
}

impl FreeSurfaceAnalyzer {
    /// Create a new free surface analyzer
    pub fn new() -> Self {
        Self {
            config: AnalysisConfig::default(),
        }
    }
    
    /// Create a new free surface analyzer with configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        Self { config }
    }
    
    /// Calculate free surface elevation
    pub fn calculate_free_surface(&self, bem_results: &wavecore_bem::BEMResult, time_points: Vec<f64>, spatial_points: Vec<Point>) -> Result<FreeSurfaceData> {
        let start_time = Instant::now();
        
        let mut elevation_values = Vec::new();
        
        for &time in &time_points {
            let mut time_elevations = Vec::new();
            
            for &point in &spatial_points {
                let elevation = self.calculate_single_elevation(bem_results, time, point)?;
                time_elevations.push(elevation);
            }
            
            elevation_values.push(time_elevations);
        }
        
        let processing_time = start_time.elapsed().as_secs_f64();
        
        Ok(FreeSurfaceData {
            time_points,
            spatial_points,
            elevation_values,
            wave_height: 1.0,
            wave_period: 10.0,
        })
    }
    
    /// Calculate single elevation point
    fn calculate_single_elevation(&self, _bem_results: &wavecore_bem::BEMResult, time: f64, point: Point) -> Result<f64> {
        // Simple harmonic wave model
        let amplitude = 0.5;
        let frequency = 0.5;
        let wave_number = 0.1;
        
        let elevation = amplitude * (frequency * time - wave_number * point.x).sin();
        Ok(elevation)
    }
}

/// Statistical analyzer
pub struct StatisticsAnalyzer;

impl StatisticsAnalyzer {
    /// Create a new statistics analyzer
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate statistics for data
    pub fn calculate_statistics(&self, data: &[f64], variable_name: &str) -> Result<StatisticsData> {
        if data.is_empty() {
            return Err(PostProError::InvalidParameters {
                message: "Data array is empty".to_string(),
            });
        }
        
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();
        
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        
        // Calculate percentiles
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p95_idx = (0.95 * data.len() as f64) as usize;
        let p99_idx = (0.99 * data.len() as f64) as usize;
        
        let p95 = if p95_idx < data.len() { sorted_data[p95_idx] } else { max };
        let p99 = if p99_idx < data.len() { sorted_data[p99_idx] } else { max };
        
        Ok(StatisticsData {
            mean: vec![mean],
            std_dev: vec![std_dev],
            max: vec![max],
            min: vec![min],
            percentiles: vec![vec![p95, p99]],
            variable_names: vec![variable_name.to_string()],
        })
    }
}

/// Main analysis engine
pub struct AnalysisEngine {
    config: AnalysisConfig,
}

impl AnalysisEngine {
    /// Create a new analysis engine
    pub fn new() -> Self {
        Self {
            config: AnalysisConfig::default(),
        }
    }
    
    /// Create a new analysis engine with configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        Self { config }
    }
    
    /// Run analysis
    pub fn run_analysis(&self, bem_results: &wavecore_bem::BEMResult) -> Result<AnalysisResult> {
        let start_time = Instant::now();
        
        let mut result = AnalysisResult::default();
        result.analysis_type = self.config.analysis_type;
        
        match self.config.analysis_type {
            AnalysisType::RAO => {
                let rao_analyzer = RAOAnalyzer::with_config(self.config.clone());
                let rao_data = rao_analyzer.calculate_raos(bem_results)?;
                result.rao_data = Some(rao_data);
            }
            AnalysisType::Kochin => {
                let kochin_analyzer = KochinAnalyzer::with_config(self.config.clone());
                let kochin_data = kochin_analyzer.calculate_kochin(bem_results)?;
                result.kochin_data = Some(kochin_data);
            }
            AnalysisType::FreeSurface => {
                // Generate time and spatial points
                let time_points: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
                let spatial_points: Vec<Point> = (0..50).map(|i| Point::new(i as f64, 0.0, 0.0)).collect();
                
                let free_surface_analyzer = FreeSurfaceAnalyzer::with_config(self.config.clone());
                let free_surface_data = free_surface_analyzer.calculate_free_surface(bem_results, time_points, spatial_points)?;
                result.free_surface_data = Some(free_surface_data);
            }
            AnalysisType::Statistics => {
                // Generate sample data for statistics
                let sample_data: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.01).sin()).collect();
                
                let stats_analyzer = StatisticsAnalyzer::new();
                let stats_data = stats_analyzer.calculate_statistics(&sample_data, "sample")?;
                result.statistics_data = Some(stats_data);
            }
            _ => {
                return Err(PostProError::InvalidParameters {
                    message: format!("Analysis type {:?} not implemented", self.config.analysis_type),
                });
            }
        }
        
        result.processing_time = start_time.elapsed().as_secs_f64();
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rao_analyzer_creation() {
        let analyzer = RAOAnalyzer::new();
        assert_eq!(analyzer.config.analysis_type, AnalysisType::RAO);
    }
    
    #[test]
    fn test_kochin_analyzer_creation() {
        let analyzer = KochinAnalyzer::new();
        assert_eq!(analyzer.config.analysis_type, AnalysisType::RAO); // Default config
    }
    
    #[test]
    fn test_free_surface_analyzer_creation() {
        let analyzer = FreeSurfaceAnalyzer::new();
        assert_eq!(analyzer.config.analysis_type, AnalysisType::RAO); // Default config
    }
    
    #[test]
    fn test_statistics_analyzer() {
        let analyzer = StatisticsAnalyzer::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = analyzer.calculate_statistics(&data, "test").unwrap();
        
        assert_eq!(stats.mean[0], 3.0);
        assert_eq!(stats.max[0], 5.0);
        assert_eq!(stats.min[0], 1.0);
        assert_eq!(stats.variable_names[0], "test");
    }
    
    #[test]
    fn test_analysis_engine() {
        let engine = AnalysisEngine::new();
        assert_eq!(engine.config.analysis_type, AnalysisType::RAO);
    }
} 