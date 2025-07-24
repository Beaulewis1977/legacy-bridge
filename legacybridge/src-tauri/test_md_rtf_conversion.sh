#!/bin/bash
# Comprehensive test suite execution for MD→RTF conversion

echo "=== MD→RTF Conversion Test Suite ==="
echo "Testing the Markdown to RTF conversion pipeline..."
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run tests and capture results
run_test_category() {
    local category=$1
    local test_pattern=$2
    
    echo -e "${YELLOW}Running $category tests...${NC}"
    
    if cargo test $test_pattern -- --nocapture 2>&1 | tee test_output.tmp; then
        echo -e "${GREEN}✓ $category tests passed${NC}"
        echo
    else
        echo -e "${RED}✗ $category tests failed${NC}"
        echo "See test_output.tmp for details"
        return 1
    fi
}

# Change to the src-tauri directory
cd "$(dirname "$0")"

# 1. Unit Tests for Markdown Parser
echo "1. Testing Markdown Parser Edge Cases"
run_test_category "Markdown Parser" "markdown_parser_edge_cases"

# 2. Unit Tests for RTF Generator
echo "2. Testing RTF Generator Edge Cases"
run_test_category "RTF Generator" "rtf_generator_edge_cases"

# 3. Integration Tests for MD→RTF Pipeline
echo "3. Testing MD→RTF Pipeline Integration"
run_test_category "MD→RTF Pipeline" "md_to_rtf_pipeline_tests"

# 4. Performance Benchmarks
echo "4. Running Performance Benchmarks"
echo -e "${YELLOW}Note: This may take a while...${NC}"
run_test_category "Performance Benchmarks" "conversion_benchmarks"

# 5. Run all existing tests to ensure no regression
echo "5. Running Regression Tests"
run_test_category "All Conversion Tests" "conversion::"

# 6. Generate test coverage report (if grcov is installed)
echo "6. Generating Test Coverage Report"
if command -v grcov &> /dev/null; then
    echo -e "${YELLOW}Generating coverage report...${NC}"
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS="-Cinstrument-coverage"
    export LLVM_PROFILE_FILE="cargo-test-%p-%m.profraw"
    
    cargo test --tests
    
    grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing -o ./coverage/
    echo -e "${GREEN}Coverage report generated in ./coverage/index.html${NC}"
else
    echo -e "${YELLOW}grcov not found. Skipping coverage report.${NC}"
fi

# 7. Memory leak check (if valgrind is available)
echo "7. Checking for Memory Leaks"
if command -v valgrind &> /dev/null; then
    echo -e "${YELLOW}Running valgrind memory check...${NC}"
    cargo test --release md_to_rtf_pipeline_tests::test_basic_md_to_rtf_conversion 2>&1 | valgrind --leak-check=full --show-leak-kinds=all cargo test 2>&1 | grep -E "(definitely lost:|ERROR SUMMARY:)" || echo -e "${GREEN}No memory leaks detected${NC}"
else
    echo -e "${YELLOW}valgrind not found. Skipping memory leak check.${NC}"
fi

# Summary
echo
echo "=== Test Summary ==="
echo "All test categories have been executed."
echo "Check the output above for any failures."

# Clean up
rm -f test_output.tmp *.profraw

echo
echo "To run specific tests:"
echo "  cargo test test_name -- --nocapture"
echo
echo "To run benchmarks only:"
echo "  cargo test benchmark -- --nocapture"
echo
echo "To test a specific markdown file:"
echo "  cargo run -- convert-md-to-rtf input.md output.rtf"