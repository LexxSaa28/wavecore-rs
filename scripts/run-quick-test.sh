#!/bin/bash

# WaveCore Quick Test Runner Script
# This script runs individual tests with progress monitoring

echo "🌊 WaveCore Quick Test Runner"
echo "============================"

# Check if we're in the right directory
if [ ! -f "config.yml" ]; then
    echo "❌ Error: config.yml not found. Please run this script from the scripts directory."
    exit 1
fi

# Check if binary exists
if [ ! -f "../target/release/wavecore_test_suite" ]; then
    echo "❌ Error: wavecore_test_suite binary not found. Please run 'cargo build --release' first."
    exit 1
fi

# Function to show progress spinner
show_spinner() {
    local pid=$1
    local delay=0.1
    local spinstr='|/-\'
    local i=0
    
    while kill -0 $pid 2>/dev/null; do
        local temp=${spinstr#?}
        printf " [%c]  " "$spinstr"
        local spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        printf "\b\b\b\b\b\b"
        i=$((i+1))
        if [ $i -eq 40 ]; then
            printf "     "
            printf "\b\b\b\b\b"
            i=0
        fi
    done
    printf "    \b\b\b\b"
}

# Function to get expected duration for each test
get_expected_duration() {
    case $1 in
        "hydrostatics")
            echo "30-60 seconds"
            ;;
        "radiation")
            echo "2-5 minutes"
            ;;
        "diffraction")
            echo "1-3 minutes"
            ;;
        "rao")
            echo "1-2 minutes"
            ;;
        "debug")
            echo "5-10 seconds"
            ;;
        *)
            echo "unknown duration"
            ;;
    esac
}

# Function to run a test with progress monitoring
run_quick_test() {
    local test_name=$1
    local test_desc=$2
    
    echo ""
    echo "🧪 Starting $test_name test..."
    echo "📝 Description: $test_desc"
    echo "⏱️  Expected duration: $(get_expected_duration "$test_name")"
    echo "📊 Sending metrics to StatsD..."
    echo ""
    
    if [ "$test_name" = "debug" ]; then
        # Debug test runs directly
        ../target/release/wavecore_test_suite debug
        local exit_code=$?
    else
        # Other tests run with progress monitoring
        ../target/release/wavecore_test_suite -c config.yml test "$test_name" &
        local test_pid=$!
        
        echo -n "🔄 Test in progress: "
        show_spinner $test_pid
        
        wait $test_pid
        local exit_code=$?
    fi
    
    if [ $exit_code -eq 0 ]; then
        echo "✅ $test_name test completed successfully!"
    else
        echo "❌ $test_name test failed with exit code $exit_code"
    fi
    
    echo "📈 Metrics sent to StatsD for $test_name"
    echo ""
}

# Function to show test status
show_test_status() {
    echo ""
    echo "📊 Test Status Information:"
    echo "=========================="
    echo "✅ Test is running normally if you see:"
    echo "   - Progress spinner [|/-\] rotating"
    echo "   - No error messages"
    echo "   - Metrics being sent to StatsD"
    echo ""
    echo "⚠️  Test might be stuck if you see:"
    echo "   - No progress for more than expected duration"
    echo "   - Error messages or exceptions"
    echo "   - No metrics being sent"
    echo ""
    echo "🔧 To check if test is still running:"
    echo "   - Press Ctrl+C to stop current test"
    echo "   - Run: ps aux | grep wavecore_test_suite"
    echo "   - Run: ./status.sh to check metrics"
    echo ""
}

# Main menu
echo "Available quick tests:"
echo "1. hydrostatics - Buoyancy and stability calculations"
echo "2. radiation - Added mass and damping"
echo "3. diffraction - Wave exciting forces"
echo "4. rao - Response Amplitude Operator"
echo "5. debug - Debug test (fastest)"
echo ""

show_test_status

read -p "Enter test number (1-5): " test_choice

case $test_choice in
    1)
        run_quick_test "hydrostatics" "Buoyancy and stability calculations"
        ;;
    2)
        run_quick_test "radiation" "Added mass and damping"
        ;;
    3)
        run_quick_test "diffraction" "Wave exciting forces"
        ;;
    4)
        run_quick_test "rao" "Response Amplitude Operator"
        ;;
    5)
        run_quick_test "debug" "Debug test (fastest)"
        ;;
    *)
        echo "❌ Invalid choice. Please run the script again."
        exit 1
        ;;
esac

echo ""
echo "📊 Final Status:"
echo "==============="
echo "✅ Test execution completed!"
echo "📈 Metrics have been sent to StatsD!"
echo ""
echo "🌐 View metrics in Grafana: http://localhost:3000"
echo "   Username: admin"
echo "   Password: wavecore123"
echo ""
echo "📈 View raw metrics: http://localhost:8080"
echo ""
echo "🔍 Check current metrics: ./status.sh" 