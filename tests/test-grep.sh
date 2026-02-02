#!/bin/bash

# Test script for grep tool functionality
# This script demonstrates the new grep tool with various test cases

set -e

echo "🔍 Testing Grep Tool Functionality"
echo "=================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Test 1: Search for 'grep' in tools.rs${NC}"
echo "Command: cargo run -- ask 'Search for the word grep in src/tools.rs'"
echo ""
cargo run -- ask "Search for the word 'grep' in src/tools.rs"
echo ""
echo "---"
echo ""

echo -e "${BLUE}Test 2: Search for function definitions in src directory${NC}"
echo "Command: cargo run -- ask 'Find all function definitions (pattern: ^pub fn) in the src directory'"
echo ""
cargo run -- ask "Find all function definitions using pattern '^pub fn' in the src directory"
echo ""
echo "---"
echo ""

echo -e "${BLUE}Test 3: Case-sensitive search${NC}"
echo "Command: cargo run -- ask 'Search for API_URL (case-sensitive) in the project root .'"
echo ""
cargo run -- ask "Search for 'API_URL' case-sensitive in the current directory ."
echo ""
echo "---"
echo ""

echo -e "${BLUE}Test 4: Search in README.md${NC}"
echo "Command: cargo run -- ask 'Search for streaming in README.md'"
echo ""
cargo run -- ask "Search for the word 'streaming' in README.md"
echo ""
echo "---"
echo ""

echo -e "${BLUE}Test 5: Search for TODO comments${NC}"
echo "Command: cargo run -- ask 'Find all TODO comments in src directory'"
echo ""
cargo run -- ask "Find all TODO or FIXME comments in the src directory"
echo ""
echo "---"
echo ""

echo -e "${GREEN}✓ Grep tool tests completed!${NC}"
echo ""
echo -e "${YELLOW}Note: You should have been prompted to approve each grep operation.${NC}"
echo -e "${YELLOW}The grep tool supports:${NC}"
echo "  - Pattern: regex pattern to search for"
echo "  - Path: file or directory to search in"
echo "  - Case sensitivity: optional (default: false)"
echo "  - Max results: optional (default: 50)"
