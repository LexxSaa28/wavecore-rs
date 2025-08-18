# WaveCore Test Configuration Guide

Panduan lengkap untuk mengkonfigurasi `config.yml` untuk berbagai skenario test WaveCore, termasuk configuration switching dan test results management.

## 📋 Overview

File `config.yml` mengontrol parameter test untuk berbagai skenario WaveCore, termasuk mesh size, frequency ranges, dan test configurations. Sistem ini dilengkapi dengan configuration switcher untuk memudahkan perpindahan antara berbagai preset configurations.

## 🚀 Quick Configuration Switching

### Using Switch Config Script
```bash
# Switch antara konfigurasi yang tersedia
./switch-config.sh

# Pilihan konfigurasi:
# 1. quick-dev     - Testing cepat (30-60 detik)
# 2. standard      - Testing standar (5-15 menit)
# 3. stress        - Stress testing
# 4. high-accuracy - Akurasi tinggi (lambat)
# 5. production    - Monitoring production
# 6. custom        - Konfigurasi kustom
```

### Available Configuration Files
```
scripts/
├── config.yml                    # Active configuration (current)
├── config-quick-dev.yml         # Quick development testing
├── config-standard.yml          # Standard performance testing
├── config-stress.yml            # Stress testing
├── config-high-accuracy.yml     # High accuracy testing
├── config-production.yml        # Production monitoring
└── config.yml.backup           # Backup configuration
```

## 📊 Test Results Location & Format

### Results Directory Structure
```
scripts/test_results/
├── YYYYMMDD_HHMMSS/             # Timestamped session folder (local time)
│   ├── test_summary.md          # Comprehensive summary for all tests
│   ├── hydrostatics_summary.md  # Individual test summary
│   ├── hydrostatics_detailed.json # Detailed JSON data
│   ├── radiation_summary.md
│   ├── radiation_detailed.json
│   ├── diffraction_summary.md
│   ├── diffraction_detailed.json
│   ├── rao_summary.md
│   └── rao_detailed.json
├── .session_timestamp           # Shared session timestamp
└── .session_lock               # Session lock file
```

### Viewing Test Results
```bash
# List all test sessions
ls -la test_results/

# Get latest session
latest_dir=$(ls -t test_results/ | head -1)
echo "Latest session: $latest_dir"

# View comprehensive summary
cat test_results/$latest_dir/test_summary.md

# View individual test results
cat test_results/$latest_dir/hydrostatics_summary.md

# View detailed JSON data
cat test_results/$latest_dir/hydrostatics_detailed.json | jq .

# View all files in latest session
ls -la test_results/$latest_dir/
```

## 🔧 Configuration Details

### 1. Quick Development Configuration (`config-quick-dev.yml`)
**Use Case:** Development dan testing cepat
```yaml
# Very small mesh for quick feedback
mesh_sizes:
  small:
    panels: 50
    max_edge_length: 1.0
    description: "Very small mesh for quick development"

# Only essential tests enabled
categories:
  hydrostatics:
    enabled: true
    timeout_seconds: 30
    mesh_sizes: ["small"]
  
  radiation:
    enabled: false
  
  diffraction:
    enabled: false
  
  rao:
    enabled: false

# Fast execution settings
global:
  timeout_seconds: 60
  retry_count: 1
  enable_metrics: true
```

**Expected Duration:** 30-60 seconds
**Accuracy:** Low (quick feedback)
**Best For:** Development, debugging, quick verification

### 2. Standard Performance Configuration (`config-standard.yml`)
**Use Case:** Testing standar dengan balanced performance
```yaml
# Balanced mesh sizes
mesh_sizes:
  small:
    panels: 1000
    max_edge_length: 0.5
  medium:
    panels: 5000
    max_edge_length: 0.25

# All test categories enabled
categories:
  hydrostatics:
    enabled: true
    timeout_seconds: 60
    mesh_sizes: ["small", "medium"]
  
  radiation:
    enabled: true
    timeout_seconds: 180
    mesh_sizes: ["small", "medium"]
  
  diffraction:
    enabled: true
    timeout_seconds: 120
    mesh_sizes: ["small", "medium"]
  
  rao:
    enabled: true
    timeout_seconds: 90
    mesh_sizes: ["small", "medium"]

# Standard settings
global:
  timeout_seconds: 300
  retry_count: 2
  enable_metrics: true
```

**Expected Duration:** 5-15 minutes
**Accuracy:** Medium (balanced)
**Best For:** Regular testing, performance validation

### 3. Stress Testing Configuration (`config-stress.yml`)
**Use Case:** Stress testing dan load testing
```yaml
# Small mesh for fast iteration
mesh_sizes:
  small:
    panels: 500
    max_edge_length: 0.8

# All tests enabled with stress patterns
categories:
  hydrostatics:
    enabled: true
    timeout_seconds: 30
    mesh_sizes: ["small"]
    stress_patterns: true
  
  radiation:
    enabled: true
    timeout_seconds: 60
    mesh_sizes: ["small"]
    stress_patterns: true
  
  diffraction:
    enabled: true
    timeout_seconds: 45
    mesh_sizes: ["small"]
    stress_patterns: true
  
  rao:
    enabled: true
    timeout_seconds: 30
    mesh_sizes: ["small"]
    stress_patterns: true

# Stress testing settings
stress_testing:
  enabled: true
  load_patterns: ["spike", "gradual", "random"]
  duration_minutes: 10
  concurrent_tests: 2
```

**Expected Duration:** 5-10 minutes
**Accuracy:** Low (stress focus)
**Best For:** Load testing, stress validation

### 4. High Accuracy Configuration (`config-high-accuracy.yml`)
**Use Case:** Testing dengan akurasi maksimum
```yaml
# Large mesh for high accuracy
mesh_sizes:
  medium:
    panels: 10000
    max_edge_length: 0.2
  large:
    panels: 25000
    max_edge_length: 0.1

# All tests with high resolution
categories:
  hydrostatics:
    enabled: true
    timeout_seconds: 300
    mesh_sizes: ["medium", "large"]
  
  radiation:
    enabled: true
    timeout_seconds: 600
    mesh_sizes: ["medium", "large"]
    frequency_resolution: "high"
  
  diffraction:
    enabled: true
    timeout_seconds: 450
    mesh_sizes: ["medium", "large"]
    frequency_resolution: "high"
  
  rao:
    enabled: true
    timeout_seconds: 300
    mesh_sizes: ["medium", "large"]
    frequency_resolution: "high"

# High accuracy settings
global:
  timeout_seconds: 900
  retry_count: 3
  enable_metrics: true
  accuracy_mode: "high"
```

**Expected Duration:** 15-30 minutes
**Accuracy:** High (maximum)
**Best For:** Final validation, research, publication

### 5. Production Monitoring Configuration (`config-production.yml`)
**Use Case:** Monitoring production environment
```yaml
# Optimized for monitoring
mesh_sizes:
  small:
    panels: 2000
    max_edge_length: 0.4

# All tests optimized for monitoring
categories:
  hydrostatics:
    enabled: true
    timeout_seconds: 120
    mesh_sizes: ["small"]
    monitoring_mode: true
  
  radiation:
    enabled: true
    timeout_seconds: 240
    mesh_sizes: ["small"]
    monitoring_mode: true
  
  diffraction:
    enabled: true
    timeout_seconds: 180
    mesh_sizes: ["small"]
    monitoring_mode: true
  
  rao:
    enabled: true
    timeout_seconds: 120
    mesh_sizes: ["small"]
    monitoring_mode: true

# Production monitoring settings
global:
  timeout_seconds: 600
  retry_count: 2
  enable_metrics: true
  monitoring_mode: true
  alert_thresholds:
    latency_ms: 100
    error_rate: 0.01
```

**Expected Duration:** 10-20 minutes
**Accuracy:** Medium (monitoring optimized)
**Best For:** Production monitoring, alerting

## 🔧 Basic Configuration Structure

```yaml
# WaveCore Test Configuration
global:
  timeout_seconds: 300
  retry_count: 3
  enable_metrics: true
  metrics_prefix: "wavecore"

# Test Categories
categories:
  hydrostatics:
    enabled: true
    timeout_seconds: 60
    retry_count: 2
    mesh_sizes: ["small", "medium", "large"]
    tests:
      - buoyancy_force
      - stability_analysis
      - center_of_gravity

  radiation:
    enabled: true
    timeout_seconds: 300
    retry_count: 2
    mesh_sizes: ["small", "medium"]
    frequency_ranges: ["low", "medium", "high"]
    tests:
      - added_mass
      - damping_coefficients

  diffraction:
    enabled: true
    timeout_seconds: 180
    retry_count: 2
    mesh_sizes: ["small", "medium"]
    frequency_ranges: ["low", "medium", "high"]
    tests:
      - wave_exciting_forces
      - pressure_distribution

  rao:
    enabled: true
    timeout_seconds: 120
    retry_count: 2
    mesh_sizes: ["small", "medium"]
    frequency_ranges: ["low", "medium", "high"]
    tests:
      - response_amplitude_operator
      - motion_transfer_functions

# Mesh Size Configurations
mesh_sizes:
  small:
    panels: 100
    max_edge_length: 0.5
    description: "Small mesh for quick tests"
    
  medium:
    panels: 500
    max_edge_length: 0.25
    description: "Medium mesh for standard tests"
    
  large:
    panels: 2000
    max_edge_length: 0.1
    description: "Large mesh for high accuracy"

# Frequency Range Configurations
frequency_ranges:
  low:
    min_freq: 0.1
    max_freq: 0.5
    num_points: 10
    description: "Low frequency range (0.1-0.5 Hz)"
    
  medium:
    min_freq: 0.5
    max_freq: 2.0
    num_points: 20
    description: "Medium frequency range (0.5-2.0 Hz)"
    
  high:
    min_freq: 2.0
    max_freq: 5.0
    num_points: 30
    description: "High frequency range (2.0-5.0 Hz)"

# Stress Testing Configuration
stress_testing:
  enabled: true
  load_patterns:
    - spike
    - gradual
    - random
  duration_minutes: 10
  concurrent_tests: 2
  alert_thresholds:
    latency_ms: 100
    error_rate: 0.01
```

## 📈 Performance Parameters

### Mesh Size Impact
| Mesh Size | Panels | Accuracy | Speed | Use Case |
|-----------|--------|----------|-------|----------|
| Very Small | 50-100 | Low | Very Fast | Development, Debug |
| Small | 500-1000 | Low-Medium | Fast | Quick Testing |
| Medium | 2000-5000 | Medium | Moderate | Standard Testing |
| Large | 10000-25000 | High | Slow | High Accuracy |
| Very Large | 50000+ | Very High | Very Slow | Research |

### Frequency Range Impact
| Range | Frequency | Points | Accuracy | Speed |
|-------|-----------|--------|----------|-------|
| Low | 0.1-0.5 Hz | 10 | Low | Fast |
| Medium | 0.5-2.0 Hz | 20 | Medium | Moderate |
| High | 2.0-5.0 Hz | 30 | High | Slow |
| Very High | 5.0-10.0 Hz | 50 | Very High | Very Slow |

## 🔄 Configuration Workflow

### 1. Choose Configuration
```bash
# Interactive configuration selection
./switch-config.sh

# Or manually copy configuration
cp config-standard.yml config.yml
```

### 2. Customize (Optional)
```bash
# Edit active configuration
nano config.yml

# Or create custom configuration
cp config-standard.yml config-custom.yml
nano config-custom.yml
```

### 3. Run Tests
```bash
# Run with current configuration
./run-test.sh

# Or run individual tests
./run-quick-test.sh
```

### 4. View Results
```bash
# Check results location
ls -la test_results/

# View latest results
latest_dir=$(ls -t test_results/ | head -1)
cat test_results/$latest_dir/test_summary.md
```

### 5. Switch Configuration (if needed)
```bash
# Switch to different configuration
./switch-config.sh

# Run tests with new configuration
./run-test.sh
```

## 📊 Results Analysis

### Understanding Test Results
```bash
# View comprehensive summary
cat test_results/LATEST_TIMESTAMP/test_summary.md

# View individual test performance
cat test_results/LATEST_TIMESTAMP/hydrostatics_summary.md

# Analyze detailed metrics
cat test_results/LATEST_TIMESTAMP/hydrostatics_detailed.json | jq '.performance'
```

### Key Metrics to Monitor
- **P50 Latency**: Median response time
- **P95 Latency**: 95th percentile response time
- **P99 Latency**: 99th percentile response time
- **Throughput**: Operations per second
- **Total Panels**: Number of panels processed
- **Test Duration**: Total execution time

### Performance Thresholds
| Metric | Good | Warning | Critical |
|--------|------|---------|----------|
| P50 Latency | < 1ms | 1-5ms | > 5ms |
| P95 Latency | < 2ms | 2-10ms | > 10ms |
| P99 Latency | < 5ms | 5-20ms | > 20ms |
| Throughput | > 1000 ops/sec | 500-1000 ops/sec | < 500 ops/sec |

## 🔍 Troubleshooting

### Common Configuration Issues

#### 1. Tests Taking Too Long
```bash
# Switch to faster configuration
./switch-config.sh
# Choose option 1 (quick-dev)
```

#### 2. Tests Failing with Timeout
```yaml
# Increase timeout in config.yml
global:
  timeout_seconds: 600  # Increase from 300
```

#### 3. Low Accuracy Results
```bash
# Switch to high accuracy configuration
./switch-config.sh
# Choose option 4 (high-accuracy)
```

#### 4. Memory Issues
```yaml
# Reduce mesh size in config.yml
mesh_sizes:
  small:
    panels: 500  # Reduce from 1000
```

### Results Location Issues

#### 1. No Results Directory
```bash
# Create results directory
mkdir -p test_results
```

#### 2. Multiple Timestamp Folders
```bash
# This is normal - each test session creates new folder
ls -la test_results/
# Use latest folder
latest_dir=$(ls -t test_results/ | head -1)
```

#### 3. Missing Test Results
```bash
# Check if tests completed successfully
cat test_results/LATEST_TIMESTAMP/test_summary.md

# Check for error messages in test output
```

## ✅ Best Practices

### 1. Configuration Selection
- **Development**: Use `quick-dev` for fast feedback
- **Testing**: Use `standard` for balanced performance
- **Validation**: Use `high-accuracy` for final validation
- **Production**: Use `production` for monitoring

### 2. Results Management
- Always check `test_summary.md` for overview
- Use individual test summaries for detailed analysis
- Keep historical results for trend analysis
- Use JSON data for programmatic analysis

### 3. Performance Monitoring
- Monitor P95 and P99 latencies
- Track throughput trends
- Set up alerts for performance degradation
- Regular configuration reviews

### 4. Configuration Management
- Use `switch-config.sh` for preset configurations
- Create custom configurations for specific needs
- Document configuration changes
- Version control configuration files

## 📚 Additional Resources

- **[Main README](README.md)** - Complete system overview
- **[Grafana Dashboard Guide](GRAFANA_DASHBOARD_GUIDE.md)** - Dashboard usage
- **Configuration Files** - All preset configurations available
- **Test Results** - Historical performance data

---

**Last Updated:** 2025-08-13
**Version:** 2.0
**Status:** ✅ Complete with configuration switching and results management 