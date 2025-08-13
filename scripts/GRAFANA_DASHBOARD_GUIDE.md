# WaveCore Grafana Dashboard Guide

## 📊 Dashboard Overview

Dashboard ini telah dikonfigurasi dengan semua metrics yang tersedia dari WaveCore test suite. Berikut adalah konfigurasi lengkap untuk setiap panel:

## 🔧 Metrics Configuration

### 1. Test Request Counter
- **Metric**: `stats_counts.wavecore.test.requests`
- **Type**: Counter
- **Description**: Jumlah total request test yang diproses
- **Unit**: Count

### 2. Latency Percentiles (P50, P95, P99)
- **Metrics**: 
  - `stats.gauges.wavecore.test.p50_latency_ms`
  - `stats.gauges.wavecore.test.p95_latency_ms`
  - `stats.gauges.wavecore.test.p99_latency_ms`
- **Type**: Gauge
- **Description**: Latency percentiles untuk monitoring performa
- **Unit**: Milliseconds (ms)

### 3. Throughput (Operations/Second)
- **Metric**: `stats.gauges.wavecore.test.throughput_ops_per_sec`
- **Type**: Gauge
- **Description**: Throughput dalam operasi per detik
- **Unit**: Operations per second (ops)

### 4. Test Duration
- **Metric**: `stats.timers.wavecore.test.duration`
- **Type**: Timer
- **Description**: Durasi eksekusi test
- **Unit**: Milliseconds (ms)

### 5. Total Metrics Processed
- **Metric**: `stats_counts.wavecore.test.metric`
- **Type**: Counter
- **Description**: Total metrics yang diproses
- **Unit**: Count

### 6. Current P50 Latency
- **Metric**: `stats.gauges.wavecore.test.p50_latency_ms`
- **Type**: Gauge
- **Description**: Latency P50 saat ini
- **Unit**: Milliseconds (ms)

### 7. Current P95 Latency
- **Metric**: `stats.gauges.wavecore.test.p95_latency_ms`
- **Type**: Gauge
- **Description**: Latency P95 saat ini
- **Unit**: Milliseconds (ms)

### 8. Current P99 Latency
- **Metric**: `stats.gauges.wavecore.test.p99_latency_ms`
- **Type**: Gauge
- **Description**: Latency P99 saat ini
- **Unit**: Milliseconds (ms)

### 9. Current Throughput
- **Metric**: `stats.gauges.wavecore.test.throughput_ops_per_sec`
- **Type**: Gauge
- **Description**: Throughput saat ini
- **Unit**: Operations per second (ops)

### 10. Panel Processing Time
- **Metric**: `stats.timers.wavecore.test.duration`
- **Type**: Timer
- **Description**: Waktu pemrosesan panel
- **Unit**: Milliseconds (ms)

## 🚀 Cara Menggunakan Dashboard

### 1. Akses Dashboard
```
URL: http://localhost:3000
Username: admin
Password: wavecore123
```

### 2. Pilih Dashboard
- **WaveCore Comprehensive Dashboard**: Dashboard lengkap dengan semua metrics
- **WaveCore Test Suite - Simple Metrics**: Dashboard sederhana untuk metrics test

### 3. Atur Time Range
- Klik tombol waktu di pojok kanan atas
- Pilih "Last 15 minutes" atau "Last 1 hour"
- Klik "Apply"

### 4. Refresh Dashboard
- Klik tombol refresh (🔄) untuk memperbarui data
- Dashboard akan auto-refresh setiap 5 detik

## 🧪 Menjalankan Test

### 🆕 Progress Monitoring & Status Information

Script test telah diperbarui dengan fitur progress monitoring yang memungkinkan Anda membedakan antara test yang sedang berjalan normal dan yang stuck:

#### **Progress Indicators:**
- **Spinner Animation**: `[|/-\]` yang berputar menunjukkan test sedang berjalan
- **Expected Duration**: Informasi durasi yang diharapkan untuk setiap test
- **Test Progress**: `[1/5]` menunjukkan progress test dalam sequence
- **Real-time Status**: Status test yang sedang berjalan

#### **Test Durations:**
- **hydrostatics**: 30-60 seconds
- **radiation**: 2-5 minutes
- **diffraction**: 1-3 minutes
- **rao**: 1-2 minutes
- **stress**: 5-10 minutes
- **debug**: 5-10 seconds

### ✅ Test Runner Scripts

#### **1. Main Test Runner** (`./run-test.sh`)
```bash
./run-test.sh
```
- Menjalankan semua test dengan progress monitoring
- Menampilkan spinner dan status real-time
- Informasi durasi yang diharapkan
- Progress tracking untuk test sequence

#### **2. Quick Test Runner** (`./run-quick-test.sh`)
```bash
./run-quick-test.sh
```
- Menjalankan test individual dengan progress monitoring
- Pilihan test yang tersedia:
  1. **hydrostatics** - Buoyancy and stability calculations
  2. **radiation** - Added mass and damping  
  3. **diffraction** - Wave exciting forces
  4. **rao** - Response Amplitude Operator
  5. **debug** - Debug test (paling cepat)

#### **3. Debug Test**
```bash
../target/release/wavecore_test_suite debug
```
- Test tercepat untuk verifikasi dashboard
- Durasi: 5-10 seconds

### 📊 Status Monitoring

#### **✅ Test Running Normally:**
- Progress spinner `[|/-\]` berputar
- Tidak ada error messages
- Metrics dikirim ke StatsD
- Durasi sesuai dengan expected duration

#### **⚠️ Test Might Be Stuck:**
- Tidak ada progress selama lebih dari expected duration
- Error messages atau exceptions
- Tidak ada metrics yang dikirim ke StatsD

#### **🔧 Troubleshooting Commands:**
```bash
# Stop current test
Ctrl+C

# Check if test is still running
ps aux | grep wavecore_test_suite

# Check metrics status
./status.sh

# Check raw metrics
curl http://localhost:8080/metrics/find?query=stats.wavecore.test.*
```

## 📈 Metrics yang Tersedia

### Counters
- `stats_counts.wavecore.test.requests` - Jumlah request
- `stats_counts.wavecore.test.metric` - Jumlah metrics

### Gauges
- `stats.gauges.wavecore.test.p50_latency_ms` - P50 Latency
- `stats.gauges.wavecore.test.p95_latency_ms` - P95 Latency
- `stats.gauges.wavecore.test.p99_latency_ms` - P99 Latency
- `stats.gauges.wavecore.test.throughput_ops_per_sec` - Throughput
- `stats.gauges.wavecore.test.debug` - Debug metrics
- `stats.gauges.wavecore.test.manual` - Manual metrics
- `stats.gauges.wavecore.test.rust_check` - Rust check metrics

### Timers
- `stats.timers.wavecore.test.duration` - Test duration

## 🔍 Troubleshooting

### Jika test stuck:
1. **Periksa progress spinner** - Jika berputar, test sedang berjalan
2. **Periksa expected duration** - Tunggu sesuai durasi yang diharapkan
3. **Gunakan Ctrl+C** untuk stop test jika diperlukan
4. **Cek metrics**: `./status.sh`
5. **Kill test jika perlu**: `pkill -f wavecore_test_suite`

### Jika metrics tidak muncul:
1. **Periksa StatsD**: `curl http://localhost:8080/metrics/find?query=stats.wavecore.test.*`
2. **Periksa Grafana**: `curl http://localhost:3000/api/health`
3. **Restart monitoring**: `./restart-monitoring.sh`
4. **Jalankan debug test**: `../target/release/wavecore_test_suite debug`

### Jika dashboard kosong:
1. **Atur time range** ke "Last 15 minutes"
2. **Refresh dashboard**
3. **Periksa datasource** di Grafana settings
4. **Verifikasi metrics** tersedia di Graphite

## 📊 Dashboard Features

- **Auto-refresh**: Setiap 5 detik
- **Legend**: Menampilkan mean, max, min
- **Tooltips**: Informasi detail saat hover
- **Responsive**: Layout yang responsif
- **Dark theme**: Tema gelap untuk monitoring

## 🎯 Best Practices

1. **Monitor progress spinner** untuk mengetahui status test
2. **Perhatikan expected duration** untuk setiap test
3. **Gunakan Quick Test** untuk test individual
4. **Monitor dashboard** secara real-time
5. **Setel alert** untuk metrics kritis
6. **Gunakan time range** yang sesuai
7. **Refresh dashboard** secara berkala
8. **Simpan dashboard** untuk referensi

## 🔗 Links

- **Grafana**: http://localhost:3000
- **StatsD/Graphite**: http://localhost:8080
- **Main Test Runner**: `./run-test.sh`
- **Quick Test Runner**: `./run-quick-test.sh`
- **Debug Test**: `../target/release/wavecore_test_suite debug`
- **Status**: `./status.sh` 