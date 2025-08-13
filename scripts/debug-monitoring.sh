#!/bin/bash

# WaveCore Test Suite Monitoring Stack - Debug
# This script provides debugging information for the monitoring stack

echo "🌊 WaveCore Test Suite Monitoring Stack - Debug"
echo "=============================================="

# Check if we're in the right directory
if [ ! -f "config.yml" ]; then
    echo "❌ Error: config.yml not found. Please run this script from the scripts directory."
    exit 1
fi

echo ""
echo "📊 Docker Services Status:"
echo "-------------------------"
docker compose ps

echo ""
echo "🔍 Container Details:"
echo "-------------------"
docker compose ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"

echo ""
echo "📋 Service Logs (last 10 lines):"
echo "-------------------------------"

echo ""
echo "📊 StatsD Logs:"
echo "--------------"
docker compose logs --tail=10 statsd

echo ""
echo "📈 Grafana Logs:"
echo "---------------"
docker compose logs --tail=10 grafana

echo ""
echo "🌐 Network Connectivity:"
echo "----------------------"

# Check StatsD connectivity
echo "📊 StatsD (UDP 8125):"
if nc -z localhost 8125 2>/dev/null; then
    echo "  ✅ UDP port 8125 is open"
else
    echo "  ❌ UDP port 8125 is not accessible"
fi

# Check StatsD HTTP interface
echo "📊 StatsD HTTP (8080):"
if curl -s http://localhost:8080 > /dev/null 2>&1; then
    echo "  ✅ HTTP port 8080 is accessible"
    
    # Check if StatsD has metrics
    echo "  📈 Checking for metrics..."
    metrics_count=$(curl -s "http://localhost:8080/metrics/find?query=stats.*" 2>/dev/null | jq length 2>/dev/null || echo "0")
    echo "  📊 Found $metrics_count metric categories"
    
    # Check wavecore metrics specifically
    wavecore_metrics=$(curl -s "http://localhost:8080/metrics/find?query=stats.wavecore.*" 2>/dev/null | jq length 2>/dev/null || echo "0")
    echo "  🌊 Found $wavecore_metrics wavecore metric categories"
    
else
    echo "  ❌ HTTP port 8080 is not accessible"
fi

# Check Grafana connectivity
echo "📊 Grafana (3000):"
if curl -s http://localhost:3000 > /dev/null 2>&1; then
    echo "  ✅ HTTP port 3000 is accessible"
else
    echo "  ❌ HTTP port 3000 is not accessible"
fi

echo ""
echo "🔧 Configuration Check:"
echo "---------------------"

# Check if binary exists
if [ -f "../target/release/wavecore_test_suite" ]; then
    echo "✅ wavecore_test_suite binary found"
else
    echo "❌ wavecore_test_suite binary not found"
    echo "   Run: cargo build --release"
fi

# Check config file
if [ -f "config.yml" ]; then
    echo "✅ config.yml found"
else
    echo "❌ config.yml not found"
fi

echo ""
echo "📊 Recent Metrics (if available):"
echo "-------------------------------"

# Try to get recent metrics
if curl -s http://localhost:8080 > /dev/null 2>&1; then
    echo "📈 Last 5 minutes of wavecore metrics:"
    
    # Test requests
    requests_value=$(curl -s "http://localhost:8080/render?target=stats.wavecore.test.requests&format=json&from=-5min" 2>/dev/null | jq -r '.[0].datapoints[-1][0]' 2>/dev/null)
    if [ "$requests_value" != "null" ] && [ -n "$requests_value" ]; then
        echo "  Test Requests: $requests_value"
    else
        echo "  Test Requests: No data"
    fi
    
    # P50 latency
    p50_value=$(curl -s "http://localhost:8080/render?target=stats.gauges.wavecore.test.p50_latency_ms&format=json&from=-5min" 2>/dev/null | jq -r '.[0].datapoints[-1][0]' 2>/dev/null)
    if [ "$p50_value" != "null" ] && [ -n "$p50_value" ]; then
        echo "  P50 Latency: ${p50_value}ms"
    else
        echo "  P50 Latency: No data"
    fi
    
else
    echo "❌ Cannot access StatsD to check metrics"
fi

echo ""
echo "🌐 Access URLs:"
echo "--------------"
echo "  Grafana Dashboard: http://localhost:3000"
echo "    Username: admin"
echo "    Password: wavecore123"
echo ""
echo "  StatsD Raw Metrics: http://localhost:8080"

echo ""
echo "🔧 Quick Fixes:"
echo "--------------"
echo "  1. If services are not running: ./start-monitoring.sh"
echo "  2. If services are stuck: ./restart-monitoring.sh"
echo "  3. If binary is missing: cargo build --release"
echo "  4. To run tests: ./run-test.sh"
echo "  5. To check status: ./status.sh" 