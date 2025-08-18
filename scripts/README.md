# WaveCore Test Suite & Monitoring System

Sistem testing dan monitoring real-time untuk WaveCore dengan StatsD dan Grafana, dilengkapi dengan progress monitoring, configuration switching, dan comprehensive test results.

## 🚀 Quick Start

### 1. Start Monitoring
```bash
./start-monitoring.sh
```

### 2. Switch Configuration (Optional)
```bash
# Pilih konfigurasi test yang sesuai
./switch-config.sh

# Pilihan konfigurasi:
# 1. quick-dev     - Testing cepat (30-60 detik)
# 2. standard      - Testing standar (5-15 menit) 
# 3. stress        - Stress testing
# 4. high-accuracy - Akurasi tinggi (lambat)
# 5. production    - Monitoring production
# 6. custom        - Konfigurasi kustom
```

### 3. Run Tests
```bash
# Main test runner dengan progress monitoring
./run-test.sh

# Quick test runner untuk test individual
./run-quick-test.sh

# Debug test untuk verifikasi cepat
../target/release/wavecore_test_suite debug
```

### 4. Check Status & Results
```bash
# Check monitoring status
./status.sh

# View test results
ls -la test_results/
cat test_results/LATEST_TIMESTAMP/test_summary.md
```

### 5. View Dashboard
Buka Grafana: http://localhost:3000
- Username: `admin`
- Password: `wavecore123`
- Dashboard: **WaveCore Comprehensive Dashboard**

## 📋 Available Tests

| Test | Description | Expected Duration | Default Panels |
|------|-------------|-------------------|----------------|
| `hydrostatics` | Buoyancy dan stability calculations | 30-60 seconds | 20,000 |
| `radiation` | Added mass dan damping | 2-5 minutes | 15,000 |
| `diffraction` | Wave exciting forces | 1-3 minutes | 12,000 |
| `rao` | Response Amplitude Operator | 1-2 minutes | 10,000 |
| `stress` | Stress testing dengan load patterns | 5-10 minutes | 25,000 |
| `debug` | Debug test (verifikasi cepat) | 5-10 seconds | 1,000 |

## 🛠️ Scripts

| Script | Description |
|--------|-------------|
| `start-monitoring.sh` | Start semua services (StatsD, Grafana) |
| `stop-monitoring.sh` | Stop semua services |
| `restart-monitoring.sh` | Restart semua services |
| `debug-monitoring.sh` | Debug monitoring stack |
| `status.sh` | Check status dan metrics |
| `switch-config.sh` | **Switch antara konfigurasi test** |
| `run-test.sh` | Main test runner dengan progress monitoring |
| `run-quick-test.sh` | Quick test runner untuk test individual |

## 🔧 Configuration Management

### Switch Configuration
```bash
./switch-config.sh
```

**Available Configurations:**

1. **Quick Development** (`config-quick-dev.yml`)
   - Very small mesh (50 panels)
   - Only hydrostatics test enabled
   - Fast execution (30-60 seconds)
   - Low accuracy for quick feedback

2. **Standard Performance** (`config-standard.yml`)
   - Small and medium mesh sizes
   - All test categories enabled
   - Balanced performance and accuracy
   - Standard execution time (5-15 minutes)

3. **Stress Testing** (`config-stress.yml`)
   - Small mesh for fast iteration
   - All test categories enabled
   - Stress testing patterns enabled
   - Load testing capabilities

4. **High Accuracy** (`config-high-accuracy.yml`)
   - Medium and large mesh sizes
   - All test categories enabled
   - High frequency resolution
   - Maximum accuracy (slow execution)

5. **Production Monitoring** (`config-production.yml`)
   - Optimized for monitoring
   - All test categories enabled
   - Balanced performance
   - Production-ready settings

6. **Custom Configuration**
   - Use your own config file
   - Specify custom path

### Configuration Files
```
scripts/
├── config.yml                    # Active configuration
├── config-quick-dev.yml         # Quick development
├── config-standard.yml          # Standard performance
├── config-stress.yml            # Stress testing
├── config-high-accuracy.yml     # High accuracy
├── config-production.yml        # Production monitoring
└── config.yml.backup           # Backup configuration
```

## 📊 Test Results & Output

### Results Location
```
scripts/test_results/
├── YYYYMMDD_HHMMSS/             # Timestamped session folder
│   ├── test_summary.md          # Comprehensive summary
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

### Results Format

#### Test Summary (`test_summary.md`)
```markdown
# WaveCore Test Suite - Comprehensive Summary

**Test Session Date:** 2025-08-13 13:45:14 +07:00

**Total Tests:** 4

**Session Duration:** 15.2s

## Test Results Overview

| Test Name | Duration | P50 Latency | P95 Latency | P99 Latency | Throughput | Total Panels |
|-----------|----------|-------------|-------------|-------------|------------|--------------|
| hydrostatics | 7.05s | 0.35ms | 0.44ms | 0.45ms | 2836.39 ops/sec | 20000 |
| radiation | 3.2s | 0.28ms | 0.35ms | 0.38ms | 4687.5 ops/sec | 15000 |
| diffraction | 2.8s | 0.32ms | 0.41ms | 0.43ms | 4285.71 ops/sec | 12000 |
| rao | 2.15s | 0.25ms | 0.31ms | 0.33ms | 4651.16 ops/sec | 10000 |
```

#### Individual Test Summary (`hydrostatics_summary.md`)
```markdown
# HYDROSTATICS Test Results

**Test Date:** 2025-08-13 13:45:14 +07:00

**Test Duration:** 7.052902298s

**Total Panels:** 20000

**P50 Latency:** 0.35 ms
**P95 Latency:** 0.44 ms
**P99 Latency:** 0.45 ms
**Throughput:** 2836.39 ops/sec

**Problem Type:** hydrostatics
**Mesh Tier:** T2

## Performance Breakdown
- **StatsD Initialization:** 115ns
- **Test Setup:** 1.596µs
- **Main Execution:** 7.051221227s
- **Metrics Calculation:** 1.634168ms
```

#### Detailed JSON Data (`hydrostatics_detailed.json`)
```json
{
  "test_name": "hydrostatics",
  "timestamp": "2025-08-13T13:45:14+07:00",
  "duration": {
    "total": 7.052902298,
    "statsd_init": 0.000000115,
    "setup": 0.000001596,
    "execution": 7.051221227,
    "metrics": 0.001634168
  },
  "performance": {
    "p50_latency_ms": 0.35,
    "p95_latency_ms": 0.44,
    "p99_latency_ms": 0.45,
    "throughput_ops_per_sec": 2836.39
  },
  "parameters": {
    "num_panels": 20000,
    "problem_type": "hydrostatics",
    "mesh_tier": "T2"
  }
}
```

### Viewing Results
```bash
# List all test sessions
ls -la test_results/

# View latest session
latest_dir=$(ls -t test_results/ | head -1)
echo "Latest session: $latest_dir"

# View comprehensive summary
cat test_results/$latest_dir/test_summary.md

# View individual test results
cat test_results/$latest_dir/hydrostatics_summary.md

# View detailed JSON data
cat test_results/$latest_dir/hydrostatics_detailed.json | jq .
```

## 🆕 Progress Monitoring Features

### Visual Progress Indicators
- **Spinner Animation**: `[|/-\]` yang berputar menunjukkan test sedang berjalan
- **Expected Duration**: Informasi durasi yang diharapkan untuk setiap test
- **Test Progress**: `[1/5]` menunjukkan progress test dalam sequence
- **Real-time Status**: Status test yang sedang berjalan

### Status Information
- **Test Running Normally**: Progress spinner berputar, tidak ada error
- **Test Might Be Stuck**: Tidak ada progress > expected duration, error messages
- **Troubleshooting Commands**: Command untuk debugging yang jelas

## 📊 Metrics

Sistem menggunakan StatsD untuk mengirim metrics secara real-time:

### Counters
- `stats_counts.wavecore.test.requests` - Jumlah test requests
- `stats_counts.wavecore.test.metric` - Jumlah metrics processed
- `stats_counts.wavecore.panel.count` - Jumlah panel processed

### Gauges
- `stats.gauges.wavecore.test.p50_latency_ms` - 50th percentile latency
- `stats.gauges.wavecore.test.p95_latency_ms` - 95th percentile latency
- `stats.gauges.wavecore.test.p99_latency_ms` - 99th percentile latency
- `stats.gauges.wavecore.test.throughput_ops_per_sec` - Throughput operations per second
- `stats.gauges.wavecore.test.total_panels` - Total panels processed

### Timers
- `stats.timers.wavecore.test.duration` - Total test execution time
- `stats.timers.wavecore.panel.processing_time` - Individual panel processing time

## 📈 Dashboard Panels

**WaveCore Comprehensive Dashboard** mencakup 10 panels:

### Time Series Charts
1. **Test Request Counter** - Jumlah test requests over time
2. **Latency Percentiles (P50, P95, P99)** - Latency trends
3. **Throughput (Operations/Second)** - Operations per second
4. **Test Duration** - Total test execution time
5. **Total Metrics Processed** - Cumulative metrics processed
6. **Current P50 Latency** - Real-time P50 latency
7. **Current P95 Latency** - Real-time P95 latency
8. **Current P99 Latency** - Real-time P99 latency
9. **Current Throughput** - Real-time throughput
10. **Panel Processing Time** - Individual panel processing time

### Features
- **Auto-refresh**: 5 seconds
- **Time range**: Last 15 minutes (adjustable)
- **Legend**: Mean, max, min calculations
- **Tooltips**: Informasi detail saat hover
- **Responsive**: Layout yang responsif
- **Dark theme**: Modern UI design

## 🔧 Configuration

### StatsD Configuration
- **Host**: `127.0.0.1`
- **Port**: `8125` (UDP)
- **Prefix**: `wavecore`
- **Batch Size**: `1` (immediate flush)
- **Flush Interval**: `100ms`

### Grafana Configuration
- **URL**: http://localhost:3000
- **Username**: `admin`
- **Password**: `wavecore123`
- **Datasource**: Graphite
- **Graphite URL**: `http://statsd:80` (internal Docker network)

### Graphite Datasource Settings
```yaml
name: Graphite
type: graphite
access: proxy
url: http://statsd:80
isDefault: true
jsonData:
  timeInterval: "5s"
  queryTimeout: "60s"
```

## 📁 File Structure

```
scripts/
├── README.md                           # This file
├── GRAFANA_DASHBOARD_GUIDE.md         # Comprehensive dashboard guide
├── TEST_CONFIGURATION_GUIDE.md        # Test configuration guide
├── docker-compose.yml                  # Docker services
├── config.yml                         # Active test configuration
├── config-quick-dev.yml              # Quick development config
├── config-standard.yml               # Standard performance config
├── config-stress.yml                 # Stress testing config
├── config-high-accuracy.yml          # High accuracy config
├── config-production.yml             # Production monitoring config
├── switch-config.sh                  # Configuration switcher
├── start-monitoring.sh               # Start monitoring
├── stop-monitoring.sh                # Stop monitoring
├── restart-monitoring.sh             # Restart monitoring
├── debug-monitoring.sh               # Debug monitoring
├── status.sh                        # Check status
├── run-test.sh                      # Main test runner
├── run-quick-test.sh                # Quick test runner
├── test_results/                    # Test results directory
│   ├── YYYYMMDD_HHMMSS/             # Timestamped sessions
│   ├── .session_timestamp           # Shared timestamp
│   └── .session_lock               # Session lock
└── monitoring/
    ├── grafana/
    │   ├── dashboards/              # Grafana dashboards
    │   │   ├── wavecore-comprehensive-dashboard.json  # Main dashboard
    │   │   ├── wavecore-simple-dashboard.json
    │   │   └── wavecore-testing-dashboard.json
    │   └── provisioning/            # Grafana configuration
    │       ├── datasources/         # Datasource configs
    │       └── dashboards/          # Dashboard configs
    └── graphite/
        └── statsd.conf             # StatsD configuration
```

## 📚 Documentation

### 📖 Guides
- **[Grafana Dashboard Guide](GRAFANA_DASHBOARD_GUIDE.md)** - Panduan lengkap penggunaan dashboard
- **[Test Configuration Guide](TEST_CONFIGURATION_GUIDE.md)** - Setup config.yml untuk berbagai skenario test

### 🔍 Troubleshooting
- **Progress Monitoring**: Cara membedakan test normal vs stuck
- **Metrics Verification**: Cara memverifikasi metrics terkirim
- **Dashboard Issues**: Solusi masalah dashboard kosong
- **Configuration Issues**: Solusi masalah konfigurasi

## 🔄 Complete Workflow

### 1. Initial Setup
```bash
# Start monitoring services
./start-monitoring.sh

# Switch to desired configuration
./switch-config.sh
```

### 2. Run Tests
```bash
# Option 1: Run all tests with progress monitoring
./run-test.sh

# Option 2: Run individual tests
./run-quick-test.sh

# Option 3: Debug test
../target/release/wavecore_test_suite debug
```

### 3. Monitor Progress
- Watch progress spinner and expected duration
- Check for any error messages
- Monitor real-time metrics in Grafana

### 4. View Results
```bash
# Check test results
latest_dir=$(ls -t test_results/ | head -1)
echo "Latest session: $latest_dir"

# View comprehensive summary
cat test_results/$latest_dir/test_summary.md

# View individual test results
ls -la test_results/$latest_dir/
```

### 5. Analyze Performance
- Open Grafana: http://localhost:3000
- View real-time metrics and trends
- Analyze performance patterns
- Export data for further analysis

### 6. Switch Configuration (if needed)
```bash
# Switch to different configuration
./switch-config.sh

# Run tests with new configuration
./run-test.sh
```

## ✅ Status

**SISTEM TESTING & MONITORING BERFUNGSI DENGAN SEMPURNA!**

- ✅ StatsD client mengirim metrics secara real-time
- ✅ Grafana dashboard menampilkan metrics dengan 10 panels
- ✅ Progress monitoring dengan spinner animation
- ✅ Expected duration information untuk setiap test
- ✅ Test status information yang jelas
- ✅ Configuration switching dengan 6 preset configurations
- ✅ Comprehensive test results dengan timestamped folders
- ✅ Individual test summaries dan detailed JSON data
- ✅ Shared session timestamp untuk multiple tests
- ✅ Quick test runner untuk test individual
- ✅ Debug test untuk verifikasi cepat
- ✅ Comprehensive documentation
- ✅ Troubleshooting guides
- ✅ Real-time performance monitoring
- ✅ Local time timestamps untuk semua results 