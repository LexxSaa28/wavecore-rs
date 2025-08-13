use std::time::Duration;

/// Performance metrics for WaveCore library tests
#[derive(Debug, Clone)]
pub struct WaveCoreMetrics {
    pub latencies: Vec<f64>,
    pub throughput: f64,
    pub memory_usage: u64,
    pub matrix_size: usize,
    pub num_panels: usize,
    pub num_frequencies: usize,
    pub problem_type: String,
    pub mesh_tier: String,
    pub has_field: bool,
    pub num_bodies: usize,
    pub depth_type: String,
    pub if_removal: bool,
    pub forward_speed: f64,
}

impl WaveCoreMetrics {
    pub fn new() -> Self {
        Self {
            latencies: Vec::new(),
            throughput: 0.0,
            memory_usage: 0,
            matrix_size: 0,
            num_panels: 0,
            num_frequencies: 0,
            problem_type: "unknown".to_string(),
            mesh_tier: "unknown".to_string(),
            has_field: false,
            num_bodies: 1,
            depth_type: "infinite".to_string(),
            if_removal: false,
            forward_speed: 0.0,
        }
    }

    pub fn with_problem_type(mut self, problem_type: &str) -> Self {
        self.problem_type = problem_type.to_string();
        self
    }

    pub fn with_mesh_tier(mut self, mesh_tier: &str) -> Self {
        self.mesh_tier = mesh_tier.to_string();
        self
    }

    pub fn with_field(mut self, has_field: bool) -> Self {
        self.has_field = has_field;
        self
    }

    pub fn with_bodies(mut self, num_bodies: usize) -> Self {
        self.num_bodies = num_bodies;
        self
    }

    pub fn with_depth(mut self, depth_type: &str) -> Self {
        self.depth_type = depth_type.to_string();
        self
    }

    pub fn with_if_removal(mut self, if_removal: bool) -> Self {
        self.if_removal = if_removal;
        self
    }

    pub fn with_forward_speed(mut self, speed: f64) -> Self {
        self.forward_speed = speed;
        self
    }

    pub fn add_latency(&mut self, latency_ms: f64) {
        self.latencies.push(latency_ms);
    }

    pub fn calculate_p50(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted[sorted.len() / 2]
    }

    pub fn calculate_p95(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = (sorted.len() as f64 * 0.95) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    pub fn calculate_p99(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = (sorted.len() as f64 * 0.99) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    pub fn calculate_throughput(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let total_time = self.latencies.iter().sum::<f64>();
        if total_time > 0.0 {
            (self.latencies.len() as f64 * 1000.0) / total_time
        } else {
            0.0
        }
    }

    /// Get percentile latency (P50, P95, P99, etc.)
    pub fn get_p_percentile_latency(&self, percentile: f64) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = (sorted.len() as f64 * percentile / 100.0) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    /// Get throughput in operations per second
    pub fn get_throughput_ops_per_sec(&self, execution_duration: Duration) -> f64 {
        if execution_duration.as_secs_f64() > 0.0 {
            self.latencies.len() as f64 / execution_duration.as_secs_f64()
        } else {
            0.0
        }
    }
} 