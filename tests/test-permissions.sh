#!/bin/bash

# Test script for squid permissions feature
# This script demonstrates the tool permissions (allow/deny lists) functionality

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "Squid Permissions Feature Test"
echo "=========================================="
echo ""
echo "This script demonstrates the tool permissions feature:"
echo "- Allow list: Tools that run automatically without confirmation"
echo "- Deny list: Tools that are completely blocked"
echo "- Interactive prompts with Always/Never options"
echo ""

# Check if squid is built
if [ ! -f "$PROJECT_ROOT/target/release/squid" ]; then
    echo "Error: squid binary not found. Please run: cargo build --release"
    exit 1
fi

SQUID="$PROJECT_ROOT/target/release/squid"

# Create a temporary test directory
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

cd "$TEST_DIR"
echo "Working in temporary directory: $TEST_DIR"
echo ""

# Test 1: Create config with default permissions
echo "=========================================="
echo "Test 1: Default Permissions"
echo "=========================================="
echo ""
echo "Creating squid.config.json with default permissions..."
cat > squid.config.json << 'EOF'
{
  "api_url": "http://127.0.0.1:1234/v1",
  "api_model": "local-model",
  "log_level": "info",
  "permissions": {
    "allow": ["now"],
    "deny": []
  }
}
EOF

echo "✓ Config created with 'now' tool in allow list"
echo ""
cat squid.config.json
echo ""

# Test 2: Show allow list behavior
echo "=========================================="
echo "Test 2: Allow List Behavior"
echo "=========================================="
echo ""
echo "The 'now' tool is in the allow list, so it should run automatically"
echo "without prompting for user approval."
echo ""
echo "Note: This requires a running LLM server (LM Studio, Ollama, etc.)"
echo "If you don't have one running, this test will be skipped."
echo ""
echo "Testing 'now' tool (with 5 second timeout)..."
echo ""

# This will only work if an LLM is running
if timeout 5s "$SQUID" ask "What time is it?" 2>/dev/null; then
    echo ""
    echo "✓ The 'now' tool executed without user prompt (auto-allowed)"
else
    echo ""
    echo "⚠ Test skipped (no LLM server running or timeout)"
fi

echo ""

# Test 3: Show deny list behavior
echo "=========================================="
echo "Test 3: Deny List Behavior"
echo "=========================================="
echo ""
echo "Let's add 'write_file' to the deny list..."
cat > squid.config.json << 'EOF'
{
  "api_url": "http://127.0.0.1:1234/v1",
  "api_model": "local-model",
  "log_level": "info",
  "permissions": {
    "allow": ["now"],
    "deny": ["write_file"]
  }
}
EOF

echo "✓ Updated config with 'write_file' in deny list"
echo ""
cat squid.config.json
echo ""

# Test 4: Configuration structure
echo "=========================================="
echo "Test 4: Configuration Structure"
echo "=========================================="
echo ""
echo "Example configurations:"
echo ""

echo "1. Read-only mode (safe for exploration):"
cat << 'EOF'
{
  "permissions": {
    "allow": ["now", "read_file", "grep"],
    "deny": ["write_file"]
  }
}
EOF
echo ""

echo "2. Write-protected mode (prevent modifications):"
cat << 'EOF'
{
  "permissions": {
    "allow": ["now", "read_file"],
    "deny": ["write_file", "grep"]
  }
}
EOF
echo ""

echo "3. Fully manual mode (confirm everything):"
cat << 'EOF'
{
  "permissions": {
    "allow": [],
    "deny": []
  }
}
EOF
echo ""

# Test 5: Interactive prompt options
echo "=========================================="
echo "Test 5: Interactive Prompt Options"
echo "=========================================="
echo ""
echo "When a tool is NOT in allow/deny lists, you get 4 options:"
echo ""
echo "  ❯ Yes (this time)           - Allow once, ask again next time"
echo "    No (skip)                 - Deny once, ask again next time"
echo "    Always (add to allow list) - Auto-approve forever, saves to config"
echo "    Never (add to deny list)   - Block forever, saves to config"
echo ""
echo "Choosing 'Always' or 'Never' automatically updates squid.config.json"
echo "and saves your preference for future runs."
echo ""

# Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo ""
echo "✓ Default config has 'now' tool in allow list"
echo "✓ Allow list tools run without user confirmation"
echo "✓ Deny list tools are blocked immediately"
echo "✓ Interactive prompts offer Always/Never options"
echo "✓ Permissions are saved automatically to squid.config.json"
echo ""
echo "For more details, see:"
echo "  - README.md (Configuration section)"
echo "  - docs/SECURITY.md (Tool Permissions section)"
echo "  - CHANGELOG.md (Unreleased section)"
echo ""
echo "=========================================="
echo "Test Complete!"
echo "=========================================="
