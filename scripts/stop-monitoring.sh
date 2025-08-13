#!/bin/bash

# WaveCore Test Suite Monitoring Stack - Stop
# This script stops the monitoring stack (StatsD + Grafana)

echo "🌊 WaveCore Test Suite Monitoring Stack - Stop"
echo "============================================="

# Check if Docker Compose is available
if ! command -v docker compose > /dev/null 2>&1; then
    echo "❌ Error: Docker Compose is not available."
    exit 1
fi

echo "⏹️  Stopping monitoring stack..."

# Stop and remove containers
docker compose down

echo "✅ Monitoring stack stopped successfully!"

echo ""
echo "📊 Services stopped:"
echo "   - StatsD"
echo "   - Grafana"

echo ""
echo "🚀 To start again:"
echo "   ./start-monitoring.sh" 