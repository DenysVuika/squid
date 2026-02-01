#!/bin/bash

# Test script for code review examples
# This script runs the review command on all example files to verify functionality
# Run this from the squid project root directory: ./tests/test-reviews.sh

set -e  # Exit on error

echo "🦑 Squid Code Review Testing Script"
echo "===================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counter for tests
TOTAL=0
PASSED=0
FAILED=0

# Function to run a test
run_test() {
    local file=$1
    local description=$2
    local extra_args=$3

    TOTAL=$((TOTAL + 1))
    echo -e "${YELLOW}Test $TOTAL:${NC} $description"
    echo "Command: cargo run -- review $file $extra_args"
    echo "----------------------------------------"

    if cargo run -- review "$file" $extra_args > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗ FAILED${NC}"
        FAILED=$((FAILED + 1))
    fi
    echo ""
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Please run this script from the squid project root directory${NC}"
    exit 1
fi

# Check if example files exist
if [ ! -d "sample-files" ]; then
    echo -e "${RED}Error: sample-files directory not found${NC}"
    exit 1
fi

echo "Starting tests..."
echo ""

# Test Rust example
run_test "sample-files/example.rs" "Review Rust file (basic)"
run_test "sample-files/example.rs" "Review Rust file (with message)" "-m 'Focus on error handling'"

# Test TypeScript example
run_test "sample-files/example.ts" "Review TypeScript file (basic)"
run_test "sample-files/example.ts" "Review TypeScript file (security focus)" "-m 'Check for security issues'"

# Test JavaScript example
run_test "sample-files/example.js" "Review JavaScript file (basic)"
run_test "sample-files/example.js" "Review JavaScript file (performance focus)" "-m 'Focus on performance'"

# Test HTML example
run_test "sample-files/example.html" "Review HTML file (basic)"
run_test "sample-files/example.html" "Review HTML file (accessibility focus)" "-m 'Focus on accessibility'"

# Test CSS example
run_test "sample-files/example.css" "Review CSS file (basic)"
run_test "sample-files/example.css" "Review CSS file (performance focus)" "-m 'Check for performance issues'"

# Test Python example (uses generic prompt)
run_test "sample-files/example.py" "Review Python file (generic prompt)"
run_test "sample-files/example.py" "Review Python file (with message)" "-m 'Focus on best practices'"

# Summary
echo "===================================="
echo "Test Summary"
echo "===================================="
echo -e "Total tests: $TOTAL"
echo -e "${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED${NC}"
else
    echo -e "Failed: $FAILED"
fi
echo ""

# Exit with appropriate code
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! 🎉${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed${NC}"
    exit 1
fi
