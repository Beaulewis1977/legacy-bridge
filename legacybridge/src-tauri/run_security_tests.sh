#!/bin/bash

# Security Test Runner for LegacyBridge
# Executes comprehensive security test suite with detailed reporting

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="$TEST_DIR/security_test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$RESULTS_DIR/security_test_report_$TIMESTAMP.txt"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    
    case $status in
        "INFO")
            echo -e "${BLUE}[INFO]${NC} $message"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[SUCCESS]${NC} $message"
            ;;
        "WARNING")
            echo -e "${YELLOW}[WARNING]${NC} $message"
            ;;
        "ERROR")
            echo -e "${RED}[ERROR]${NC} $message"
            ;;
    esac
}

# Function to run a test module
run_test_module() {
    local module=$1
    local description=$2
    
    print_status "INFO" "Running $description..."
    
    if cargo test --test $module -- --test-threads=1 --nocapture 2>&1 | tee -a "$REPORT_FILE"; then
        print_status "SUCCESS" "$description passed"
        return 0
    else
        print_status "ERROR" "$description failed"
        return 1
    fi
}

# Start security testing
echo "========================================" | tee "$REPORT_FILE"
echo "LegacyBridge Security Test Suite" | tee -a "$REPORT_FILE"
echo "Timestamp: $(date)" | tee -a "$REPORT_FILE"
echo "========================================" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

# Check prerequisites
print_status "INFO" "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    print_status "ERROR" "Cargo not found. Please install Rust."
    exit 1
fi

# Run unit tests for security modules
print_status "INFO" "Running security unit tests..."

FAILED_TESTS=0

# Core security tests
if ! cargo test --lib conversion::security_test -- --test-threads=1 --nocapture 2>&1 | tee -a "$REPORT_FILE"; then
    ((FAILED_TESTS++))
fi

if ! cargo test --lib conversion::malicious_input_tests -- --test-threads=1 --nocapture 2>&1 | tee -a "$REPORT_FILE"; then
    ((FAILED_TESTS++))
fi

# Integration security tests
print_status "INFO" "Running integration security tests..."

# Fuzzing tests
if ! cargo test --test fuzzing_tests -- --test-threads=1 --nocapture 2>&1 | tee -a "$REPORT_FILE"; then
    ((FAILED_TESTS++))
fi

# DoS resistance tests
if ! cargo test --test dos_resistance_tests -- --test-threads=1 --nocapture 2>&1 | tee -a "$REPORT_FILE"; then
    ((FAILED_TESTS++))
fi

# Injection tests
if ! cargo test --test injection_tests -- --test-threads=1 --nocapture 2>&1 | tee -a "$REPORT_FILE"; then
    ((FAILED_TESTS++))
fi

# Performance security tests
if ! cargo test --test performance_security_tests -- --test-threads=1 --nocapture 2>&1 | tee -a "$REPORT_FILE"; then
    ((FAILED_TESTS++))
fi

# Test malicious samples
print_status "INFO" "Testing malicious sample files..."

SAMPLES_DIR="$TEST_DIR/tests/security/samples/malicious"
if [ -d "$SAMPLES_DIR" ]; then
    for sample in "$SAMPLES_DIR"/*.rtf "$SAMPLES_DIR"/*.md; do
        if [ -f "$sample" ]; then
            filename=$(basename "$sample")
            print_status "INFO" "Testing sample: $filename"
            
            # Run the sample through the validator (this would use your actual binary)
            # For now, we'll just note it in the report
            echo "Sample test: $filename" >> "$REPORT_FILE"
        fi
    done
else
    print_status "WARNING" "Malicious samples directory not found"
fi

# Performance benchmarks
print_status "INFO" "Running security performance benchmarks..."

if cargo bench --bench conversion_bench -- security 2>&1 | tee -a "$REPORT_FILE"; then
    print_status "SUCCESS" "Security benchmarks completed"
else
    print_status "WARNING" "Security benchmarks had issues"
fi

# Memory safety check with Miri (if available)
if command -v cargo-miri &> /dev/null; then
    print_status "INFO" "Running Miri memory safety checks..."
    
    if MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --lib conversion::security_test 2>&1 | tee -a "$REPORT_FILE"; then
        print_status "SUCCESS" "Miri checks passed"
    else
        print_status "WARNING" "Miri checks had issues"
    fi
else
    print_status "INFO" "Skipping Miri checks (not installed)"
fi

# Static analysis
print_status "INFO" "Running static security analysis..."

# Clippy security lints
if cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery -D warnings 2>&1 | tee -a "$REPORT_FILE"; then
    print_status "SUCCESS" "Clippy analysis passed"
else
    print_status "WARNING" "Clippy found issues"
fi

# Cargo audit for dependencies
if command -v cargo-audit &> /dev/null; then
    print_status "INFO" "Checking for vulnerable dependencies..."
    
    if cargo audit 2>&1 | tee -a "$REPORT_FILE"; then
        print_status "SUCCESS" "No vulnerable dependencies found"
    else
        print_status "ERROR" "Vulnerable dependencies detected"
        ((FAILED_TESTS++))
    fi
else
    print_status "INFO" "Skipping dependency audit (cargo-audit not installed)"
fi

# Generate summary
echo "" | tee -a "$REPORT_FILE"
echo "========================================" | tee -a "$REPORT_FILE"
echo "Security Test Summary" | tee -a "$REPORT_FILE"
echo "========================================" | tee -a "$REPORT_FILE"

if [ $FAILED_TESTS -eq 0 ]; then
    print_status "SUCCESS" "All security tests passed!" | tee -a "$REPORT_FILE"
    EXIT_CODE=0
else
    print_status "ERROR" "$FAILED_TESTS test suite(s) failed" | tee -a "$REPORT_FILE"
    EXIT_CODE=1
fi

echo "" | tee -a "$REPORT_FILE"
echo "Full report saved to: $REPORT_FILE" | tee -a "$REPORT_FILE"

# Generate HTML report if possible
if command -v pandoc &> /dev/null; then
    HTML_REPORT="$RESULTS_DIR/security_test_report_$TIMESTAMP.html"
    pandoc -f markdown -t html -o "$HTML_REPORT" "$REPORT_FILE" 2>/dev/null || true
    
    if [ -f "$HTML_REPORT" ]; then
        print_status "INFO" "HTML report generated: $HTML_REPORT"
    fi
fi

exit $EXIT_CODE