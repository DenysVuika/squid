#!/bin/bash

# Test script for grep tool functionality
# This script demonstrates the new grep tool with various test cases
# Run this from the project root directory

set -e

echo "🔍 Testing Grep Tool Functionality"
echo "=================================="
echo ""

# Ensure we're in the project root
if [ ! -f "Cargo.toml" ]; then
    echo "ERROR: Please run this script from the project root directory"
    echo "Usage: ./tests/test-grep.sh"
    exit 1
fi

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Test 1: Search for 'grep' in tools.rs${NC}"
echo "Command: cargo run -- ask 'Use the grep tool to search for the word grep in src/tools.rs'"
echo ""
cargo run -- ask "Use the grep tool to search for the word 'grep' in src/tools.rs"
echo ""
echo "---"
echo ""

echo -e "${BLUE}Test 2: Search for function definitions in src directory${NC}"
echo "Command: cargo run -- ask 'Use the grep tool with pattern ^pub fn to search in src/'"
echo ""
cargo run -- ask "Use the grep tool with pattern '^pub fn' to search in src/"
echo ""
echo "---"
echo ""

echo -e "${BLUE}Test 3: Case-sensitive search${NC}"
echo "Command: cargo run -- ask 'Use the grep tool to search for Client case-sensitive in src/'"
echo ""
cargo run -- ask "Use the grep tool to search for 'Client' case-sensitive in src/"
echo ""
echo "---"
echo ""

echo -e "${BLUE}Test 4: Search in README.md${NC}"
echo "Command: cargo run -- ask 'Use the grep tool to search for streaming in README.md'"
echo ""
cargo run -- ask "Use the grep tool to search for the word 'streaming' in README.md"
echo ""
echo "---"
echo ""

echo -e "${BLUE}Test 5: Search for TODO comments${NC}"
echo "Command: cargo run -- ask 'Use the grep tool with pattern (TODO|FIXME) to search in src/'"
echo ""
cargo run -- ask "Use the grep tool with pattern '(TODO|FIXME)' to search in src/"
echo ""
echo "---"
echo ""

echo -e "${BLUE}Test 6: Search for use statements${NC}"
echo "Command: cargo run -- ask 'Use the grep tool to find all use statements in src/main.rs'"
echo ""
cargo run -- ask "Use the grep tool with pattern '^use ' to search in src/main.rs"
echo ""
echo "---"
echo ""

echo -e "${GREEN}✓ Grep tool tests completed!${NC}"
echo ""
echo -e "${YELLOW}Note: You should have been prompted to approve each grep operation.${NC}"
echo -e "${YELLOW}The grep tool supports:${NC}"
echo "  - Pattern: regex pattern to search for"
echo "  - Path: file or directory to search in (relative to project root)"
echo "  - Case sensitivity: optional (default: false)"
echo "  - Max results: optional (default: 50)"
echo ""
echo -e "${YELLOW}Tips for using the grep tool:${NC}"
echo "  - Paths are relative to where you run the command from"
echo "  - Use trailing slash for directories (e.g., 'src/' instead of 'src')"
echo "  - Regex patterns support full Rust regex syntax"
echo "  - For case-sensitive searches, explicitly mention it in your prompt"
