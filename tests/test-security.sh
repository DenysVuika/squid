#!/bin/bash

# Test script for demonstrating security approval feature
# This script shows how the user approval works for tool calling
# Run this from the squid project root directory: ./tests/test-security.sh

set -e

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Please run this script from the squid project root directory"
    echo "Usage: ./tests/test-security.sh"
    exit 1
fi

echo "🔒 Security Approval Feature Test"
echo "=================================="
echo ""
echo "This script will demonstrate the security approval feature."
echo "You'll be prompted to approve or deny each tool execution."
echo ""
echo "Press Ctrl+C to exit at any time."
echo ""

# Test 1: File read approval
echo "📖 Test 1: File Read Approval"
echo "-------------------------------"
echo "Command: squid ask \"Read the README.md file and tell me what this project does\""
echo ""
echo "Expected behavior:"
echo "  1. LLM will request to read README.md"
echo "  2. You'll see: 'Allow reading file: README.md? (Y/n)'"
echo "  3. Press Y to approve or N to skip"
echo ""
read -p "Press Enter to run this test..."
cargo run --release -- ask "Read the README.md file and tell me what this project does"
echo ""
echo "✅ Test 1 complete"
echo ""

# Test 2: File write approval
echo "✍️  Test 2: File Write Approval"
echo "-------------------------------"
echo "Command: squid ask \"Create a test-output.txt file with 'This is a security test'\""
echo ""
echo "Expected behavior:"
echo "  1. LLM will request to write to test-output.txt"
echo "  2. You'll see a content preview"
echo "  3. Press Y to approve or N to skip"
echo ""
read -p "Press Enter to run this test..."
cargo run --release -- ask "Create a test-output.txt file with 'This is a security test'"
echo ""
echo "✅ Test 2 complete"
echo ""

# Test 3: Multiple approvals
echo "🔄 Test 3: Multiple Tool Calls"
echo "-------------------------------"
echo "Command: squid ask \"Read Cargo.toml and create a summary.txt with the project name and version\""
echo ""
echo "Expected behavior:"
echo "  1. First approval: Read Cargo.toml (Y/n)"
echo "  2. Second approval: Write summary.txt with preview (Y/n)"
echo ""
read -p "Press Enter to run this test..."
cargo run --release -- ask "Read Cargo.toml and create a summary.txt with the project name and version"
echo ""
echo "✅ Test 3 complete"
echo ""

# Test 4: Denial test
echo "🚫 Test 4: Denying Tool Execution"
echo "----------------------------------"
echo "Command: squid ask \"Read the .env file\""
echo ""
echo "Expected behavior:"
echo "  1. You'll be prompted: 'Allow reading file: .env? (Y/n)'"
echo "  2. Press N to deny (to demonstrate security)"
echo "  3. LLM should receive an error message"
echo ""
read -p "Press Enter to run this test..."
cargo run --release -- ask "Read the .env file"
echo ""
echo "✅ Test 4 complete"
echo ""

echo "🎉 All tests complete!"
echo ""
echo "Security features verified:"
echo "  ✓ File read operations require approval"
echo "  ✓ File write operations show content preview"
echo "  ✓ Multiple tool calls each require approval"
echo "  ✓ Users can deny suspicious operations"
echo ""
echo "Generated files (if approved):"
echo "  - test-output.txt"
echo "  - summary.txt"
echo ""
echo "You can clean up with: rm -f test-output.txt summary.txt"
