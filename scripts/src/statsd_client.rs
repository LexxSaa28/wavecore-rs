use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lazy_static::lazy_static;
use statsd::client::Client;

/// StatsD client configuration
#[derive(Debug, Clone)]
pub struct StatsDConfig {
    pub host: String,
    pub port: u16,
    pub prefix: String,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

impl Default for StatsDConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8125,
            prefix: "wavecore".to_string(),
            batch_size: 1,
            flush_interval_ms: 100,
        }
    }
}

// Global StatsD client using official statsd crate v0.10.0
lazy_static! {
    static ref STATSD_CLIENT: Arc<Mutex<Option<Client>>> = Arc::new(Mutex::new(None));
}

/// Initialize global StatsD client using official statsd crate v0.10.0
pub fn init_statsd(config: StatsDConfig) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(&format!("{}:{}", config.host, config.port), &config.prefix)?;
    
    let mut global_client = STATSD_CLIENT.lock().unwrap();
    *global_client = Some(client);
    
    println!("DEBUG: StatsD client initialized successfully with host: {}:{}", config.host, config.port);
    Ok(())
}

/// Get global StatsD client
pub fn get_statsd() -> Option<Arc<Mutex<Option<Client>>>> {
    Some(STATSD_CLIENT.clone())
}

/// Send counter metric using official statsd crate v0.10.0
pub fn counter(name: &str, value: f64, tags: Option<HashMap<String, String>>) {
    if let Ok(mut client_guard) = STATSD_CLIENT.lock() {
        if let Some(client) = client_guard.as_mut() {
            // Note: statsd v0.10.0 doesn't support tags, so we ignore them
            client.count(name, value);
            // eprintln!("DEBUG: Sent counter metric: {}:{}", name, value);
        } else {
            eprintln!("DEBUG: StatsD client not initialized for counter: {}", name);
        }
    } else {
        eprintln!("DEBUG: Failed to lock StatsD client for counter: {}", name);
    }
}

/// Send gauge metric using official statsd crate v0.10.0
pub fn gauge(name: &str, value: f64, tags: Option<HashMap<String, String>>) {
    if let Ok(mut client_guard) = STATSD_CLIENT.lock() {
        if let Some(client) = client_guard.as_mut() {
            // Note: statsd v0.10.0 doesn't support tags, so we ignore them
            client.gauge(name, value);
            // eprintln!("DEBUG: Sent gauge metric: {}:{}", name, value);
        } else {
            eprintln!("DEBUG: StatsD client not initialized for gauge: {}", name);
        }
    } else {
        eprintln!("DEBUG: Failed to lock StatsD client for gauge: {}", name);
    }
}

/// Send timer metric using official statsd crate v0.10.0
pub fn timer(name: &str, value_ms: f64, tags: Option<HashMap<String, String>>) {
    if let Ok(mut client_guard) = STATSD_CLIENT.lock() {
        if let Some(client) = client_guard.as_mut() {
            // Note: statsd v0.10.0 doesn't support tags, so we ignore them
            client.timer(name, value_ms);
            // eprintln!("DEBUG: Sent timer metric: {}:{}ms", name, value_ms);
        } else {
            eprintln!("DEBUG: StatsD client not initialized for timer: {}", name);
        }
    } else {
        eprintln!("DEBUG: Failed to lock StatsD client for timer: {}", name);
    }
}

/// Send histogram metric using official statsd crate v0.10.0
pub fn histogram(name: &str, value: f64, tags: Option<HashMap<String, String>>) {
    if let Ok(mut client_guard) = STATSD_CLIENT.lock() {
        if let Some(client) = client_guard.as_mut() {
            // Note: statsd v0.10.0 doesn't have histogram method, use timer instead
            client.timer(name, value);
            // eprintln!("DEBUG: Sent histogram metric (as timer): {}:{}", name, value);
        } else {
            eprintln!("DEBUG: StatsD client not initialized for histogram: {}", name);
        }
    } else {
        eprintln!("DEBUG: Failed to lock StatsD client for histogram: {}", name);
    }
}

/// Time a function execution using official statsd crate v0.10.0
pub fn time_function<F, R>(name: &str, f: F) -> R 
where
    F: FnOnce() -> R,
{
    use std::time::Instant;
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    
    timer(name, duration.as_millis() as f64, None);
    result
} 