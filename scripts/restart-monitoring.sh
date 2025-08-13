#!/bin/bash

# WaveCore Test Suite Monitoring Stack - Restart
# This script restarts the monitoring stack (StatsD + Grafana)

echo "🌊 WaveCore Test Suite Monitoring Stack - Restart"
echo "==============================================="

# Check if Docker Compose is available
if ! command -v docker compose > /dev/null 2>&1; then
    echo "❌ Error: Docker Compose is not available."
    exit 1
fi

echo "🔄 Restarting monitoring stack..."

# Stop existing containers
echo "⏹️  Stopping existing containers..."
docker compose down

# Start containers
echo "🚀 Starting containers..."
docker compose up -d

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 5

# Check if services are running
echo "📊 Checking service status..."
if docker compose ps | grep -q "Up"; then
    echo "✅ Monitoring stack restarted successfully!"
    
    echo ""
    echo "📊 Services:"
    echo "   - StatsD:    http://localhost:8080"
    echo "   - Grafana:   http://localhost:3000 (admin/wavecore123)"
    
    echo ""
    echo "🔧 Next steps:"
    echo "   1. Run tests to see live metrics:"
    echo "      ./run-test.sh"
    echo ""
    echo "   2. Check monitoring status:"
    echo "      ./status.sh"
    
else
    echo "❌ Error: Failed to restart monitoring stack"
    echo "📋 Checking logs..."
    docker compose logs
    exit 1
fi 