# Sample Files

This directory contains example files for testing the `squid` CLI functionality, including code review prompts and file context features.

## Files

### Test File (`sample.txt`)
A general-purpose test file for trying out the file context feature.

Contains information about:
- The squid project
- Key features
- Example usage
- Fun facts about squids

**Test command:**
```bash
cargo run -- ask -f sample-files/sample.txt "What is this document about?"
cargo run -- ask -f sample-files/sample.txt "How many hearts does a squid have?"
```

## Code Review Examples

The following files contain intentional code issues designed to test the `squid review` command and its file-type-specific prompts. These files are useful for:

- Testing the code review functionality
- Evaluating the quality of review suggestions
- Demonstrating what kinds of issues each prompt catches
- Onboarding new developers to the project

### Rust (`example.rs`)
Tests the Rust-specific review prompt with issues like:
- Using `unwrap()` that can panic
- Unnecessary `clone()` operations
- Non-idiomatic code (missing iterator usage)
- Poor error handling
- Overly nested code
- Magic numbers
- Using `String` where `&str` would be better

**Test command:**
```bash
cargo run -- review sample-files/example.rs
cargo run -- review sample-files/example.rs --stream
cargo run -- review sample-files/example.rs -m "Focus on error handling and panics"
```

### TypeScript (`example.ts`)
Tests the TypeScript/JavaScript review prompt with issues like:
- Using `any` type
- Missing async/await error handling
- Using `var` instead of `const`/`let`
- Potential XSS vulnerabilities
- Memory leaks (event listeners not cleaned up)
- Inefficient array operations
- Not using destructuring
- Magic numbers

**Test command:**
```bash
cargo run -- review sample-files/example.ts
cargo run -- review sample-files/example.ts -m "Check for security issues"
```

### JavaScript (`example.js`)
Tests the JavaScript-specific review with issues like:
- Security risks (eval, innerHTML)
- Callback hell
- No error handling
- Inefficient loops
- Memory leaks
- Using `==` instead of `===`
- Not using modern ES6+ features
- Mutating parameters

**Test command:**
```bash
cargo run -- review sample-files/example.js
cargo run -- review sample-files/example.js --stream
```

### HTML (`example.html`)
Tests the HTML review prompt with issues like:
- Missing semantic elements
- Accessibility problems (missing alt text, labels)
- Improper heading hierarchy
- Missing ARIA attributes
- Non-interactive elements with click handlers
- Inline scripts and styles
- Missing meta tags

**Test command:**
```bash
cargo run -- review sample-files/example.html
cargo run -- review sample-files/example.html -m "Focus on accessibility"
```

### CSS (`example.css`)
Tests the CSS review prompt with issues like:
- Performance problems (universal selector, expensive animations)
- Overly specific selectors
- Magic numbers
- Not using CSS variables
- Fixed pixel units instead of responsive units
- Using `!important`
- Poor naming conventions
- No focus styles
- Hardcoded z-index values

**Test command:**
```bash
cargo run -- review sample-files/example.css
cargo run -- review sample-files/example.css -m "Check for performance issues"
```

### Python (`example.py`)
Tests the generic code review prompt with issues like:
- Mutable default arguments
- Using bare `except`
- Security risks (eval, hardcoded credentials)
- Not using context managers
- Poor exception handling
- Violating Single Responsibility Principle
- Magic numbers
- Using deprecated methods

**Test command:**
```bash
cargo run -- review sample-files/example.py
cargo run -- review sample-files/example.py --stream
```

## Automated Testing

Run all code review examples automatically:

```bash
# From the project root
./tests/test-reviews.sh
```

This will test all example files and show pass/fail results. See **[tests/README.md](../tests/README.md)** for more details.

## Usage Tips

1. **Stream mode**: Use the `-s` or `--stream` flag to see responses in real-time
2. **Specific questions**: Use the `-m` or `--message` flag to focus the review on specific aspects
3. **Compare prompts**: Try reviewing the same file type with different prompts to see the difference

## Example Commands

```bash
# Basic review
cargo run -- review sample-files/example.rs

# Streaming review
cargo run -- review sample-files/example.ts --stream

# Review with specific focus
cargo run -- review sample-files/example.js -m "Focus on security vulnerabilities"

# Review with specific question
cargo run -- review sample-files/example.html -m "Are there any accessibility issues?"

# Performance-focused review
cargo run -- review sample-files/example.css -m "What can I do to improve performance?"
```

## Contributing

When adding new example files:

1. Include a variety of issues that the review prompt should catch
2. Add comments indicating what the issues are (for reference)
3. Keep examples realistic and representative of common problems
4. Update this README with the new file and test commands
5. Consider adding examples for new file types with specialized prompts

## Notes

- These files intentionally contain bad practices - **do not use them as references for good code!**
- The AI's responses may vary depending on the model and configuration
- Use these examples to refine and improve the review prompts over time