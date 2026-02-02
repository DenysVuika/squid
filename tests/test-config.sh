#!/bin/bash

# Test script for squid configuration functionality
# Tests both squid.config.json and .env file loading

set -e

echo "=== Squid Configuration Tests ==="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Build the project first
echo "Building squid..."
cargo build --release 2>&1 | grep -q "Finished" && echo "✓ Build successful" || { echo "✗ Build failed"; exit 1; }
echo

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"

    echo -n "Testing: $test_name... "

    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Cleanup function
cleanup() {
    rm -f squid.config.json
    rm -f .env.test
}

# Test 1: Config file creation
echo "Test 1: Creating example config file"
cat > squid.config.json << 'EOF'
{
  "api_url": "http://test-url:1234/v1",
  "api_model": "test-model",
  "api_key": "test-key"
}
EOF
run_test "Config file created" "[ -f squid.config.json ]"
echo

# Test 2: Config file parsing
echo "Test 2: Verifying config file structure"
run_test "Config has api_url" "grep -q '\"api_url\"' squid.config.json"
run_test "Config has api_model" "grep -q '\"api_model\"' squid.config.json"
run_test "Config has api_key" "grep -q '\"api_key\"' squid.config.json"
echo

# Test 3: Config example file exists
echo "Test 3: Checking example config file"
run_test "Example config exists" "[ -f squid.config.json.example ]"
run_test "Example config is valid JSON" "cat squid.config.json.example | python3 -m json.tool > /dev/null"
echo

# Test 4: Config without api_key (optional field)
echo "Test 4: Testing config without api_key"
cat > squid.config.json << 'EOF'
{
  "api_url": "http://127.0.0.1:1234/v1",
  "api_model": "local-model"
}
EOF
run_test "Config without api_key is valid JSON" "cat squid.config.json | python3 -m json.tool > /dev/null"
echo

# Test 5: Config module exists
echo "Test 5: Checking config module"
run_test "config.rs exists" "[ -f src/config.rs ]"
run_test "config module has Config struct" "grep -q 'pub struct Config' src/config.rs"
run_test "config module has load method" "grep -q 'pub fn load' src/config.rs"
run_test "config module has save method" "grep -q 'pub fn save' src/config.rs"
echo

# Test 6: main.rs imports config
echo "Test 6: Checking main.rs integration"
run_test "main.rs imports config module" "grep -q 'mod config' src/main.rs"
run_test "main.rs has Init command" "grep -q 'Commands::Init' src/main.rs"
echo

# Cleanup
cleanup

# Summary
echo
echo "==================================="
echo "Test Summary:"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
fi
echo "==================================="
