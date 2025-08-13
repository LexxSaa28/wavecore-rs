#!/bin/bash

# WaveCore Test Runner Script
# This script runs tests and generates real-time metrics with progress indicators

echo "🌊 WaveCore Test Runner"
echo "======================"

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

# Function to run a test with progress monitoring
run_test() {
    local test_name=$1
    local test_number=$2
    local total_tests=$3
    
    echo ""
    echo "🧪 [${test_number}/${total_tests}] Starting $test_name test..."
    echo "⏱️  Expected duration: $(get_expected_duration "$test_name")"
    echo "📊 Sending metrics to StatsD..."
    echo ""
    
    # Start the test in background
    ../target/release/wavecore_test_suite -c config.yml test "$test_name" &
    local test_pid=$!
    
    # Show progress with spinner
    echo -n "🔄 Test in progress: "
    show_spinner $test_pid
    
    # Wait for the test to complete
    wait $test_pid
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        echo "✅ $test_name test completed successfully!"
    else
        echo "❌ $test_name test failed with exit code $exit_code"
    fi
    
    echo "📈 Metrics sent to StatsD for $test_name"
    echo ""
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
        "stress")
            echo "5-10 minutes"
            ;;
        *)
            echo "unknown duration"
            ;;
    esac
}

# Function to run stress test with progress
run_stress() {
    echo ""
    echo "🔥 Starting stress tests..."
    echo "⏱️  Expected duration: $(get_expected_duration "stress")"
    echo "📊 Sending metrics to StatsD..."
    echo ""
    
    ../target/release/wavecore_test_suite -c config.yml stress &
    local stress_pid=$!
    
    echo -n "🔄 Stress test in progress: "
    show_spinner $stress_pid
    
    wait $stress_pid
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        echo "✅ Stress tests completed successfully!"
    else
        echo "❌ Stress tests failed with exit code $exit_code"
    fi
    
    echo "📈 Metrics sent to StatsD for stress tests"
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
    echo "   - No progress for more than 10 minutes"
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
echo "Available tests:"
echo "1. hydrostatics - Buoyancy and stability calculations"
echo "2. radiation - Added mass and damping"
echo "3. diffraction - Wave exciting forces"
echo "4. rao - Response Amplitude Operator"
echo "5. stress - Stress testing with load patterns"
echo "6. all - Run all tests"
echo ""

show_test_status

read -p "Enter test number (1-6): " choice

case $choice in
    1)
        run_test "hydrostatics" 1 1
        ;;
    2)
        run_test "radiation" 1 1
        ;;
    3)
        run_test "diffraction" 1 1
        ;;
    4)
        run_test "rao" 1 1
        ;;
    5)
        run_stress
        ;;
    6)
        echo "🔄 Running all tests..."
        echo "📋 Test sequence: hydrostatics → radiation → diffraction → rao → stress"
        echo "⏱️  Total expected time: 10-20 minutes"
        
        # Run all tests in a single process to share timestamp
        echo "🧪 Running all tests in single process..."
        if ../target/release/wavecore_test_suite -c config.yml all; then
            echo "✅ All tests completed successfully!"
            echo "📈 All metrics have been sent to StatsD!"
        else
            echo "❌ Some tests failed!"
            exit 1
        fi
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
echo "📊 Test results saved to: test_results/"

# Show actual test duration if available
if [ -d "test_results" ]; then
    echo ""
    echo "📁 Test Results Summary:"
    echo "======================="
    latest_dir=$(ls -t test_results/ | head -1)
    if [ -n "$latest_dir" ]; then
        echo "📂 Latest test run: $latest_dir"
        echo ""
        echo "📋 Individual Test Results:"
        for test_file in test_results/$latest_dir/*_summary.md; do
            if [ -f "$test_file" ]; then
                test_name=$(basename "$test_file" _summary.md | tr '[:lower:]' '[:upper:]')
                duration=$(grep "Test Duration:" "$test_file" | cut -d':' -f2 | xargs)
                echo "   • $test_name: $duration"
            fi
        done
    fi
fi

echo ""
echo "🌐 View metrics in Grafana: http://localhost:3000"
echo "   Username: admin"
echo "   Password: wavecore123"
echo ""
echo "📈 View raw metrics: http://localhost:8080"
echo ""
echo "🔍 Check current metrics: ./status.sh" 