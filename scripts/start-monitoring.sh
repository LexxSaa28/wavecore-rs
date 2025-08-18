#!/bin/bash

# WaveCore Test Suite Monitoring Stack - Start
# This script starts the monitoring stack (StatsD + Grafana)

echo "🌊 WaveCore Test Suite Monitoring Stack - Start"
echo "=============================================="

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Error: Docker is not running. Please start Docker first."
    exit 1
fi

# Check if Docker Compose is available
if ! command -v docker compose > /dev/null 2>&1; then
    echo "❌ Error: Docker Compose is not available. Please install Docker Compose."
    exit 1
fi

# Check if containers are already running
if docker compose ps | grep -q "Up"; then
    echo "⚠️  Warning: Monitoring stack is already running!"
    echo "   To restart, use: ./restart-monitoring.sh"
    echo "   To stop, use: ./stop-monitoring.sh"
    exit 1
fi

echo "🚀 Starting monitoring stack..."

# Start containers
docker compose up -d

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 5

# Check if services are running
echo "📊 Checking service status..."
if docker compose ps | grep -q "Up"; then
    echo "✅ Monitoring stack started successfully!"
    
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
    echo ""
    echo "   3. View metrics in Grafana:"
    echo "      http://localhost:3000"
    
    echo ""
    echo "📈 To view logs:"
    echo "   docker compose logs -f"
    
else
    echo "❌ Error: Failed to start monitoring stack"
    echo "📋 Checking logs..."
    docker compose logs
    exit 1
fi 