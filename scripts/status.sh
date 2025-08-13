#!/bin/bash

# WaveCore Monitoring Status Script
# This script shows the status of monitoring services and metrics

echo "🌊 WaveCore Monitoring Status"
echo "============================"

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
echo "📈 StatsD Metrics:"
echo "-----------------"

# Check if StatsD is running
if curl -s http://localhost:8080 > /dev/null 2>&1; then
    echo "✅ StatsD is running"
    
    # Get wavecore metrics
    echo ""
    echo "📊 WaveCore Metrics:"
    
    # Counters
    echo "🔢 Counters:"
    counter_metrics=$(curl -s "http://localhost:8080/metrics/find?query=stats.wavecore.test.*" 2>/dev/null | jq -r '.[].text' 2>/dev/null)
    if [ -n "$counter_metrics" ]; then
        echo "$counter_metrics" | while read metric; do
            echo "  - $metric"
        done
    else
        echo "  No counter metrics found"
    fi
    
    # Gauges
    echo ""
    echo "📊 Gauges:"
    gauge_metrics=$(curl -s "http://localhost:8080/metrics/find?query=stats.gauges.wavecore.test.*" 2>/dev/null | jq -r '.[].text' 2>/dev/null)
    if [ -n "$gauge_metrics" ]; then
        echo "$gauge_metrics" | while read metric; do
            echo "  - $metric"
        done
    else
        echo "  No gauge metrics found"
    fi
    
    # Timers
    echo ""
    echo "⏱️  Timers:"
    timer_metrics=$(curl -s "http://localhost:8080/metrics/find?query=stats.timers.wavecore.test.*" 2>/dev/null | jq -r '.[].text' 2>/dev/null)
    if [ -n "$timer_metrics" ]; then
        echo "$timer_metrics" | while read metric; do
            echo "  - $metric"
        done
    else
        echo "  No timer metrics found"
    fi
    
    # Show latest values
    echo ""
    echo "📈 Latest Values (last 5 minutes):"
    echo "---------------------------------"
    
    # Test requests
    requests_value=$(curl -s "http://localhost:8080/render?target=stats.wavecore.test.requests&format=json&from=-5min" 2>/dev/null | jq -r '.[0].datapoints[-1][0]' 2>/dev/null)
    if [ "$requests_value" != "null" ] && [ -n "$requests_value" ]; then
        echo "  Test Requests: $requests_value"
    fi
    
    # P50 latency
    p50_value=$(curl -s "http://localhost:8080/render?target=stats.gauges.wavecore.test.p50_latency_ms&format=json&from=-5min" 2>/dev/null | jq -r '.[0].datapoints[-1][0]' 2>/dev/null)
    if [ "$p50_value" != "null" ] && [ -n "$p50_value" ]; then
        echo "  P50 Latency: ${p50_value}ms"
    fi
    
    # P95 latency
    p95_value=$(curl -s "http://localhost:8080/render?target=stats.gauges.wavecore.test.p95_latency_ms&format=json&from=-5min" 2>/dev/null | jq -r '.[0].datapoints[-1][0]' 2>/dev/null)
    if [ "$p95_value" != "null" ] && [ -n "$p95_value" ]; then
        echo "  P95 Latency: ${p95_value}ms"
    fi
    
    # P99 latency
    p99_value=$(curl -s "http://localhost:8080/render?target=stats.gauges.wavecore.test.p99_latency_ms&format=json&from=-5min" 2>/dev/null | jq -r '.[0].datapoints[-1][0]' 2>/dev/null)
    if [ "$p99_value" != "null" ] && [ -n "$p99_value" ]; then
        echo "  P99 Latency: ${p99_value}ms"
    fi
    
    # Throughput
    throughput_value=$(curl -s "http://localhost:8080/render?target=stats.gauges.wavecore.test.throughput_ops_per_sec&format=json&from=-5min" 2>/dev/null | jq -r '.[0].datapoints[-1][0]' 2>/dev/null)
    if [ "$throughput_value" != "null" ] && [ -n "$throughput_value" ]; then
        echo "  Throughput: ${throughput_value} ops/sec"
    fi
    
    # Test duration
    duration_value=$(curl -s "http://localhost:8080/render?target=stats.timers.wavecore.test.duration&format=json&from=-5min" 2>/dev/null | jq -r '.[0].datapoints[-1][0]' 2>/dev/null)
    if [ "$duration_value" != "null" ] && [ -n "$duration_value" ]; then
        echo "  Test Duration: ${duration_value}ms"
    fi
    
else
    echo "❌ StatsD is not running"
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
echo "🔧 Quick Commands:"
echo "-----------------"
echo "  Run tests: ./run-test.sh"
echo "  Start monitoring: ./start-monitoring.sh"
echo "  Stop monitoring: ./stop-monitoring.sh"
echo "  Restart monitoring: ./restart-monitoring.sh"
echo "  Debug monitoring: ./debug-monitoring.sh" 