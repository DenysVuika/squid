#!/bin/bash

# Test script for custom prompt feature
# Tests the -p/--prompt flag for the ask command

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "================================="
echo "Custom Prompt Feature Test"
echo "================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create temporary test prompts
echo -e "${BLUE}Creating test prompts...${NC}"

# Test 1: Pirate prompt
cat > /tmp/pirate-prompt.md << 'EOF'
You are a pirate assistant. Always respond in pirate speak using terms like "ahoy", "matey", "arr", and "savvy". Be brief and stay in character.
EOF

# Test 2: Formal prompt
cat > /tmp/formal-prompt.md << 'EOF'
You are a formal academic assistant. Respond in a very formal, scholarly tone. Use phrases like "Furthermore", "Indeed", "One must consider", etc. Be brief but formal.
EOF

# Test 3: Emoji prompt
cat > /tmp/emoji-prompt.md << 'EOF'
You are a fun assistant who uses emojis in every response. Start each sentence with a relevant emoji. Be brief and enthusiastic.
EOF

echo ""
echo "================================="
echo "Test 1: Permanent Test Prompt File"
echo "================================="
echo -e "${YELLOW}Command: squid ask -p tests/test-prompt.md \"What is Rust?\" --no-stream${NC}"
echo ""
./target/release/squid ask -p tests/test-prompt.md "What is Rust?" --no-stream
echo ""

echo "================================="
echo "Test 2: Pirate Prompt (Temporary)"
echo "================================="
echo -e "${YELLOW}Command: squid ask -p /tmp/pirate-prompt.md \"What is Rust?\" --no-stream${NC}"
echo ""
./target/release/squid ask -p /tmp/pirate-prompt.md "What is Rust?" --no-stream
echo ""

echo "================================="
echo "Test 3: Formal Academic Prompt"
echo "================================="
echo -e "${YELLOW}Command: squid ask -p /tmp/formal-prompt.md \"What is Rust?\" --no-stream${NC}"
echo ""
./target/release/squid ask -p /tmp/formal-prompt.md "What is Rust?" --no-stream
echo ""

echo "================================="
echo "Test 4: Emoji Prompt"
echo "================================="
echo -e "${YELLOW}Command: squid ask -p /tmp/emoji-prompt.md \"Hello\" --no-stream${NC}"
echo ""
./target/release/squid ask -p /tmp/emoji-prompt.md "Hello" --no-stream
echo ""

echo "================================="
echo "Test 5: Custom Prompt with File Context"
echo "================================="
echo -e "${YELLOW}Command: squid ask -f sample-files/sample.txt -p tests/test-prompt.md \"What is this about?\" --no-stream${NC}"
echo ""
./target/release/squid ask -f sample-files/sample.txt -p tests/test-prompt.md "What is this about?" --no-stream
echo ""

echo "================================="
echo "Test 6: Invalid Prompt File (Error Handling)"
echo "================================="
echo -e "${YELLOW}Command: squid ask -p /tmp/nonexistent-prompt.md \"Hello\" --no-stream${NC}"
echo ""
./target/release/squid ask -p /tmp/nonexistent-prompt.md "Hello" --no-stream 2>&1 || echo -e "${GREEN}✓ Error handled correctly${NC}"
echo ""

# Cleanup
echo -e "${BLUE}Cleaning up temporary files...${NC}"
rm -f /tmp/pirate-prompt.md /tmp/formal-prompt.md /tmp/emoji-prompt.md

echo ""
echo "================================="
echo -e "${GREEN}Custom Prompt Tests Complete!${NC}"
echo "================================="
echo ""
echo "Summary:"
echo "- Test 1: Permanent test-prompt.md file (pirate speak)"
echo "- Test 2: Temporary pirate prompt"
echo "- Test 3: Formal prompt should respond in academic tone"
echo "- Test 4: Emoji prompt should use emojis"
echo "- Test 5: Custom prompt with file context"
echo "- Test 6: Error handling for missing prompt file"
echo ""
echo "If all responses matched their expected styles, the feature works correctly!"
