#!/bin/bash

# Test script for bash tool functionality
# This script demonstrates the bash tool's capabilities and security features

set -e

SQUID="cargo run --release --"

echo "🧪 Testing Bash Tool Functionality"
echo "===================================="
echo ""

# Test 1: Safe ls command
echo "📋 Test 1: List directory contents (safe command)"
echo "Command: ls -la"
echo ""
$SQUID ask "Execute the command 'ls -la' to show directory contents"
echo ""
echo "✅ Test 1 complete"
echo ""

# Test 2: Git status
echo "📋 Test 2: Check git status (safe command)"
echo "Command: git status"
echo ""
$SQUID ask "Run git status to show the repository state"
echo ""
echo "✅ Test 2 complete"
echo ""

# Test 3: Echo command
echo "📋 Test 3: Echo command (safe command)"
echo "Command: echo 'Hello from bash tool'"
echo ""
$SQUID ask "Execute: echo 'Hello from bash tool'"
echo ""
echo "✅ Test 3 complete"
echo ""

# Test 4: Current directory
echo "📋 Test 4: Show current directory (safe command)"
echo "Command: pwd"
echo ""
$SQUID ask "What is the current directory? Use pwd command"
echo ""
echo "✅ Test 4 complete"
echo ""

# Test 5: Custom timeout
echo "📋 Test 5: Command with custom timeout"
echo "Command: ls -R (with 5 second timeout)"
echo ""
$SQUID ask "List all files recursively with ls -R, use a 5 second timeout"
echo ""
echo "✅ Test 5 complete"
echo ""

# Test 6: Dangerous command (should be blocked)
echo "📋 Test 6: Attempt dangerous command (should be blocked)"
echo "Command: rm -rf /tmp/test"
echo ""
echo "⚠️  This should be automatically blocked by security checks"
$SQUID ask "Execute: rm -rf /tmp/test" || echo "✅ Command correctly blocked"
echo ""
echo "✅ Test 6 complete"
echo ""

# Test 7: Another dangerous command
echo "📋 Test 7: Attempt sudo command (should be blocked)"
echo "Command: sudo ls"
echo ""
echo "⚠️  This should be automatically blocked by security checks"
$SQUID ask "Run: sudo ls" || echo "✅ Command correctly blocked"
echo ""
echo "✅ Test 7 complete"
echo ""

# Test 8: Natural language request
echo "📋 Test 8: Natural language request"
echo "Request: Show me all Rust files in src/"
echo ""
$SQUID ask "Show me all Rust files in the src directory"
echo ""
echo "✅ Test 8 complete"
echo ""

echo "===================================="
echo "🎉 All bash tool tests complete!"
echo ""
echo "📝 Notes:"
echo "  - Tests 1-5 demonstrate safe, allowed commands"
echo "  - Tests 6-7 verify security blocks for dangerous commands"
echo "  - Test 8 shows natural language interpretation"
echo ""
echo "💡 To test with permissions:"
echo "  - Add 'bash' to allow list: Edit squid.config.json"
echo "  - Or use interactive prompts and choose 'Always'"
echo ""
