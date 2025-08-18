#!/bin/bash

# WaveCore Configuration Switcher
# Script untuk mudah beralih antara konfigurasi test yang berbeda

echo "🌊 WaveCore Configuration Switcher"
echo "=================================="
echo ""

# Check if we're in the right directory
if [ ! -f "config.yml" ]; then
    echo "❌ Error: config.yml not found. Please run this script from the scripts directory."
    exit 1
fi

echo "Available configurations:"
echo "1. quick-dev    - Quick development testing (fastest)"
echo "2. standard     - Standard performance testing"
echo "3. stress       - Stress testing and load testing"
echo "4. high-accuracy - High accuracy testing (slowest)"
echo "5. production   - Production monitoring"
echo "6. custom       - Use custom config file"
echo ""

read -p "Enter configuration number (1-6): " choice

case $choice in
    1)
        echo "🔄 Switching to Quick Development configuration..."
        if [ -f "config-quick-dev.yml" ]; then
            cp config-quick-dev.yml config.yml
            echo "✅ Switched to Quick Development configuration"
            echo "📋 Features:"
            echo "   - Very small mesh (50 panels)"
            echo "   - Only hydrostatics test enabled"
            echo "   - Fast execution (30-60 seconds)"
            echo "   - Low accuracy for quick feedback"
        else
            echo "❌ Error: config-quick-dev.yml not found"
            exit 1
        fi
        ;;
    2)
        echo "🔄 Switching to Standard Performance configuration..."
        if [ -f "config-standard.yml" ]; then
            cp config-standard.yml config.yml
            echo "✅ Switched to Standard Performance configuration"
            echo "📋 Features:"
            echo "   - Small and medium mesh sizes"
            echo "   - All test categories enabled"
            echo "   - Balanced performance and accuracy"
            echo "   - Standard execution time (5-15 minutes)"
        else
            echo "❌ Error: config-standard.yml not found"
            exit 1
        fi
        ;;
    3)
        echo "🔄 Switching to Stress Testing configuration..."
        if [ -f "config-stress.yml" ]; then
            cp config-stress.yml config.yml
            echo "✅ Switched to Stress Testing configuration"
            echo "📋 Features:"
            echo "   - Small mesh for fast iteration"
            echo "   - All test categories enabled"
            echo "   - Stress testing patterns enabled"
            echo "   - Load testing capabilities"
        else
            echo "❌ Error: config-stress.yml not found"
            exit 1
        fi
        ;;
    4)
        echo "🔄 Switching to High Accuracy configuration..."
        if [ -f "config-high-accuracy.yml" ]; then
            cp config-high-accuracy.yml config.yml
            echo "✅ Switched to High Accuracy configuration"
            echo "📋 Features:"
            echo "   - Medium and large mesh sizes"
            echo "   - All test categories enabled"
            echo "   - High frequency resolution"
            echo "   - Maximum accuracy (slow execution)"
        else
            echo "❌ Error: config-high-accuracy.yml not found"
            exit 1
        fi
        ;;
    5)
        echo "🔄 Switching to Production Monitoring configuration..."
        if [ -f "config-production.yml" ]; then
            cp config-production.yml config.yml
            echo "✅ Switched to Production Monitoring configuration"
            echo "📋 Features:"
            echo "   - Optimized for monitoring"
            echo "   - All test categories enabled"
            echo "   - Balanced performance"
            echo "   - Production-ready settings"
        else
            echo "❌ Error: config-production.yml not found"
            exit 1
        fi
        ;;
    6)
        echo "🔄 Custom configuration..."
        read -p "Enter custom config file path: " custom_config
        if [ -f "$custom_config" ]; then
            cp "$custom_config" config.yml
            echo "✅ Switched to custom configuration: $custom_config"
        else
            echo "❌ Error: Custom config file not found: $custom_config"
            exit 1
        fi
        ;;
    *)
        echo "❌ Invalid choice. Please run the script again."
        exit 1
        ;;
esac

echo ""
echo "📊 Current Configuration:"
echo "========================"
echo "File: config.yml"
echo "Size: $(du -h config.yml | cut -f1)"
echo "Last modified: $(stat -c %y config.yml | cut -d' ' -f1,2)"
echo ""

echo "🚀 Next Steps:"
echo "=============="
echo "1. Run tests with new configuration:"
echo "   ./run-test.sh"
echo "   ./run-quick-test.sh"
echo ""
echo "2. Check monitoring status:"
echo "   ./status.sh"
echo ""
echo "3. View dashboard:"
echo "   http://localhost:3000"
echo ""
echo "4. View configuration guide:"
echo "   cat TEST_CONFIGURATION_GUIDE.md" 