# Test Scripts

This directory contains test scripts for the `squid` CLI tool.

## Directory Structure

```
squid/
├── tests/              # ← You are here
│   ├── README.md       # This file
│   ├── test-security.sh    # Security approval tests
│   └── test-reviews.sh     # Code review tests
├── examples/           # Example files with intentional issues
│   ├── example.rs      # Rust example
│   ├── example.ts      # TypeScript example
│   ├── example.js      # JavaScript example
│   ├── example.html    # HTML example
│   ├── example.css     # CSS example
│   └── example.py      # Python example
└── docs/              # Documentation
    ├── SECURITY.md    # Security features guide
    └── ...
```

## Available Tests

### 🔒 Security Approval Test (`test-security.sh`)

Interactive test script that demonstrates the security approval feature for tool calling.

**Usage:**
```bash
# From the project root
./tests/test-security.sh
```

**What it tests:**
- ✅ File read approval prompts
- ✅ File write approval with content preview
- ✅ Multiple tool call approvals in sequence
- ✅ Denying/skipping tool executions

**Note:** This is an interactive test - you'll be prompted to approve or deny each operation.

---

### 🔍 Code Review Test (`test-reviews.sh`)

Automated test script that validates the code review functionality across different file types.

**Usage:**
```bash
# From the project root
./tests/test-reviews.sh
```

**What it tests:**
- ✅ Rust code reviews (`.rs`)
- ✅ TypeScript code reviews (`.ts`)
- ✅ JavaScript code reviews (`.js`)
- ✅ HTML code reviews (`.html`)
- ✅ CSS code reviews (`.css`)
- ✅ Python code reviews (`.py` - generic prompt)
- ✅ Reviews with custom messages (`-m` flag)

**Output:** Shows pass/fail status for each test with a summary at the end.

---

## Running Tests

### Prerequisites

1. Make sure you're in the project root directory
2. Build the project: `cargo build --release`
3. Configure your `.env` file with API credentials
4. Make scripts executable:
   ```bash
   chmod +x tests/test-security.sh
   chmod +x tests/test-reviews.sh
   ```

### Run All Tests

```bash
# Security tests (interactive)
./tests/test-security.sh

# Code review tests (automated)
./tests/test-reviews.sh
```

### Run Specific Tests

Both scripts run all their tests by default. For custom testing, you can run individual commands manually:

```bash
# Test a specific review
cargo run -- review examples/example.rs

# Test a specific security scenario
cargo run -- ask "Read README.md and summarize it"
```

## Test Files

The test scripts use files from the `examples/` directory:
- `examples/example.rs` - Rust code with intentional issues
- `examples/example.ts` - TypeScript code with issues
- `examples/example.js` - JavaScript code with issues
- `examples/example.html` - HTML with accessibility issues
- `examples/example.css` - CSS with performance issues
- `examples/example.py` - Python code (generic review)

See `examples/README.md` for details about each example file.

## Troubleshooting

### "Please run this script from the squid project root directory"

Both scripts must be run from the project root, not from the `tests/` directory:

```bash
# ❌ Wrong
cd tests
./test-security.sh

# ✅ Correct
./tests/test-security.sh
```

### Tests Fail to Run

1. Check that you've built the project: `cargo build --release`
2. Verify your `.env` configuration
3. Ensure LM Studio is running (if using local model)
4. Check that example files exist in `examples/` directory

### Security Test Hangs

The security test is interactive and waits for your input. Press `Y` or `N` when prompted, or `Ctrl+C` to exit.

## Adding New Tests

To add new tests, edit the appropriate script:

1. **Security tests** - Add new test scenarios to `test-security.sh`
2. **Review tests** - Add new file types or test cases to `test-reviews.sh`

Follow the existing pattern for consistency.

## Notes

- Security tests require manual interaction (Y/N prompts)
- Review tests run automatically and show pass/fail results
- Both tests use the `--release` build for better performance
- All tests require a valid API configuration in `.env`
