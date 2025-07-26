#!/bin/bash

# Quick Security Check for LegacyBridge
# Runs essential security tests for rapid validation

set -euo pipefail

echo "==================================="
echo "LegacyBridge Quick Security Check"
echo "==================================="
echo ""

# Run core security tests
echo "Running core security tests..."
cargo test --lib conversion::security_test::test_billion_laughs_protection -- --nocapture

echo ""
echo "Testing integer overflow protection..."
cargo test --lib conversion::security_test::test_integer_overflow_protection -- --nocapture

echo ""
echo "Testing control word injection..."
cargo test --lib conversion::security_test::test_control_word_injection -- --nocapture

echo ""
echo "Testing deep nesting protection..."
cargo test --lib conversion::security_test::test_deep_nesting_attack -- --nocapture

echo ""
echo "Running malicious input tests..."
cargo test --lib conversion::malicious_input_tests::test_billion_laughs_attack -- --nocapture
cargo test --lib conversion::malicious_input_tests::test_embedded_object_injection -- --nocapture
cargo test --lib conversion::malicious_input_tests::test_markdown_xss_injection -- --nocapture

echo ""
echo "==================================="
echo "Quick Security Check Complete"
echo "==================================="
echo ""
echo "For comprehensive testing, run: ./run_security_tests.sh"