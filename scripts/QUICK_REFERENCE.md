# WaveCore Test Suite - Quick Reference

## 🚀 Quick Start Commands

### 1. Start Monitoring
```bash
./start-monitoring.sh
```

### 2. Switch Configuration
```bash
./switch-config.sh
# 1. quick-dev (30-60s)    2. standard (5-15min)
# 3. stress (5-10min)      4. high-accuracy (15-30min)
# 5. production (10-20min)  6. custom
```

### 3. Run Tests
```bash
# Run all tests with progress monitoring
./run-test.sh

# Run individual tests
./run-quick-test.sh

# Debug test
../target/release/wavecore_test_suite debug
```

### 4. View Results
```bash
# Check latest results
latest_dir=$(ls -t test_results/ | head -1)
cat test_results/$latest_dir/test_summary.md

# View individual test
cat test_results/$latest_dir/hydrostatics_summary.md
```

### 5. Monitor Dashboard
```bash
# Open Grafana
http://localhost:3000
# Username: admin
# Password: wavecore123
```

## 📋 Available Tests

| Test | Duration | Panels | Description |
|------|----------|--------|-------------|
| `hydrostatics` | 30-60s | 20,000 | Buoyancy & stability |
| `radiation` | 2-5min | 15,000 | Added mass & damping |
| `diffraction` | 1-3min | 12,000 | Wave exciting forces |
| `rao` | 1-2min | 10,000 | Response amplitude |
| `stress` | 5-10min | 25,000 | Load testing |
| `debug` | 5-10s | 1,000 | Quick verification |

## 🔧 Configuration Presets

| Config | Duration | Accuracy | Use Case |
|--------|----------|----------|----------|
| `quick-dev` | 30-60s | Low | Development |
| `standard` | 5-15min | Medium | Regular testing |
| `stress` | 5-10min | Low | Load testing |
| `high-accuracy` | 15-30min | High | Final validation |
| `production` | 10-20min | Medium | Monitoring |

## 📊 Test Results Location

```
scripts/test_results/
├── YYYYMMDD_HHMMSS/          # Session folder (local time)
│   ├── test_summary.md       # Comprehensive summary
│   ├── hydrostatics_summary.md
│   ├── hydrostatics_detailed.json
│   ├── radiation_summary.md
│   ├── radiation_detailed.json
│   ├── diffraction_summary.md
│   ├── diffraction_detailed.json
│   ├── rao_summary.md
│   └── rao_detailed.json
```

## 📈 Key Metrics

| Metric | Good | Warning | Critical |
|--------|------|---------|----------|
| P50 Latency | < 1ms | 1-5ms | > 5ms |
| P95 Latency | < 2ms | 2-10ms | > 10ms |
| P99 Latency | < 5ms | 5-20ms | > 20ms |
| Throughput | > 1000 ops/sec | 500-1000 ops/sec | < 500 ops/sec |

## 🛠️ Useful Commands

### Configuration Management
```bash
# Switch configuration
./switch-config.sh

# View current config
cat config.yml

# Backup config
cp config.yml config.yml.backup
```

### Results Analysis
```bash
# List all sessions
ls -la test_results/

# Get latest session
latest_dir=$(ls -t test_results/ | head -1)

# View comprehensive summary
cat test_results/$latest_dir/test_summary.md

# View JSON data
cat test_results/$latest_dir/hydrostatics_detailed.json | jq .

# Compare sessions
diff test_results/SESSION1/test_summary.md test_results/SESSION2/test_summary.md
```

### Monitoring
```bash
# Check status
./status.sh

# View dashboard
http://localhost:3000

# Stop monitoring
./stop-monitoring.sh

# Restart monitoring
./restart-monitoring.sh
```

### Troubleshooting
```bash
# Debug monitoring
./debug-monitoring.sh

# Check logs
docker-compose logs

# Reset results
rm -rf test_results/
```

## 🔄 Common Workflows

### Development Workflow
```bash
./start-monitoring.sh
./switch-config.sh  # Choose 1 (quick-dev)
./run-quick-test.sh
cat test_results/$(ls -t test_results/ | head -1)/test_summary.md
```

### Testing Workflow
```bash
./start-monitoring.sh
./switch-config.sh  # Choose 2 (standard)
./run-test.sh
cat test_results/$(ls -t test_results/ | head -1)/test_summary.md
http://localhost:3000
```

### Validation Workflow
```bash
./start-monitoring.sh
./switch-config.sh  # Choose 4 (high-accuracy)
./run-test.sh
cat test_results/$(ls -t test_results/ | head -1)/test_summary.md
./status.sh
```

## 📚 Documentation Files

- **[README.md](README.md)** - Complete system overview
- **[TEST_CONFIGURATION_GUIDE.md](TEST_CONFIGURATION_GUIDE.md)** - Configuration details
- **[GRAFANA_DASHBOARD_GUIDE.md](GRAFANA_DASHBOARD_GUIDE.md)** - Dashboard usage

## ✅ Status Check

**System Status:** ✅ Fully Operational
- ✅ Monitoring services running
- ✅ Test suite functional
- ✅ Results saving working
- ✅ Local time timestamps
- ✅ Configuration switching
- ✅ Progress monitoring
- ✅ Real-time metrics

---

**Last Updated:** 2025-08-13
**Version:** 2.0 