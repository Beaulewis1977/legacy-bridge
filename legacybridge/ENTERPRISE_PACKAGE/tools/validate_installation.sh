#!/bin/bash
# LegacyBridge Installation Validator
# Version 1.0.0

echo "============================================"
echo "LegacyBridge Installation Validator"
echo "Version 1.0.0"
echo "============================================"
echo

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    printf "Testing: %-40s" "$test_name"
    
    if eval "$test_command" >/dev/null 2>&1; then
        echo -e "${GREEN}[PASS]${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}[FAIL]${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Platform detection
PLATFORM="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macos"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    PLATFORM="windows"
fi

echo "Detected platform: $PLATFORM"
echo

# Linux-specific tests
if [ "$PLATFORM" == "linux" ]; then
    echo "Running Linux installation tests..."
    echo
    
    run_test "Library installed" "[ -f /usr/local/lib/liblegacybridge.so ]"
    run_test "Header installed" "[ -f /usr/local/include/legacybridge.h ]"
    run_test "Library in ldconfig" "ldconfig -p | grep -q liblegacybridge"
    run_test "Symbolic link exists" "[ -L /usr/local/lib/liblegacybridge.so.1 ]"
    run_test "pkg-config available" "pkg-config --exists legacybridge"
    run_test "Installation directory" "[ -d /opt/legacybridge ]"
    run_test "Examples directory" "[ -d /opt/legacybridge/examples ]"
    run_test "Documentation directory" "[ -d /opt/legacybridge/docs ]"
    
    # Test compilation
    if command -v gcc >/dev/null 2>&1; then
        echo
        echo "Testing compilation..."
        cat > /tmp/legacybridge_test.c << 'EOF'
#include <legacybridge.h>
#include <stdio.h>
int main() {
    const char* version = get_version();
    printf("LegacyBridge version: %s\n", version);
    return 0;
}
EOF
        
        if gcc -o /tmp/legacybridge_test /tmp/legacybridge_test.c -llegacybridge 2>/dev/null; then
            run_test "Compilation test" "[ -f /tmp/legacybridge_test ]"
            run_test "Execution test" "/tmp/legacybridge_test"
        else
            echo -e "Compilation test: ${RED}[FAIL]${NC}"
            ((TESTS_FAILED++))
        fi
        
        rm -f /tmp/legacybridge_test /tmp/legacybridge_test.c
    fi
fi

# Windows-specific tests
if [ "$PLATFORM" == "windows" ]; then
    echo "Running Windows installation tests..."
    echo
    
    run_test "DLL in System32" "[ -f /c/Windows/System32/legacybridge.dll ]"
    run_test "Program Files directory" "[ -d '/c/Program Files/LegacyBridge' ]"
    run_test "Header installed" "[ -f '/c/Program Files/LegacyBridge/include/legacybridge.h' ]"
    run_test "Examples installed" "[ -d '/c/Program Files/LegacyBridge/examples' ]"
    run_test "Registry entry" "reg query 'HKLM\SOFTWARE\LegacyBridge' 2>/dev/null | grep -q Version"
fi

# Common tests
echo
echo "Running common tests..."
echo

# Check for performance test tool
if [ -f "../tools/perf_test" ] || [ -f "/opt/legacybridge/tools/perf_test" ]; then
    run_test "Performance test tool" "true"
else
    run_test "Performance test tool" "false"
fi

# Summary
echo
echo "============================================"
echo "VALIDATION SUMMARY"
echo "============================================"
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"
echo

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! LegacyBridge is properly installed.${NC}"
    exit 0
else
    echo -e "${YELLOW}Some tests failed. Please check the installation.${NC}"
    echo "Refer to INSTALL_GUIDE.txt for troubleshooting."
    exit 1
fi