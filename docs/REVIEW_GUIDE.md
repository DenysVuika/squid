# Code Review Guide

## Overview

The `squid review` command provides AI-powered code reviews with file-type-specific prompts that focus on language-specific best practices, common pitfalls, and optimization opportunities.

## Quick Start

```bash
# Review a file
squid review path/to/file.rs

# Stream the response (real-time output)
squid review path/to/file.ts --stream

# Add a specific question or focus area
squid review path/to/file.js -m "Focus on security vulnerabilities"
```

## File Type Support

### Specialized Prompts

The review command automatically selects the most appropriate prompt based on file extension:

| File Types | Focus Areas |
|------------|-------------|
| `.rs` | Rust idioms, ownership, safety, error handling, performance |
| `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs` | TypeScript/JavaScript best practices, modern features, security |
| `.html`, `.htm` | Semantic HTML, accessibility, SEO |
| `.css`, `.scss`, `.sass`, `.less` | Modern CSS, performance, responsive design, maintainability |
| All others | Generic code quality, security, performance, best practices |

### Rust Reviews

**What it checks:**
- Ownership, borrowing, and lifetime usage
- Idiomatic patterns (iterators, pattern matching)
- Error handling (`Result`, `Option`, avoiding panics)
- Performance (unnecessary clones, allocations)
- Safety considerations

**Example:**
```bash
squid review src/main.rs
squid review lib.rs -m "Are there any potential panics?"
```

### TypeScript/JavaScript Reviews

**What it checks:**
- Type safety (avoiding `any`, proper interfaces)
- Modern ES6+ features
- Async/await patterns
- Security issues (XSS, injection)
- Performance (memory leaks, inefficient operations)

**Example:**
```bash
squid review src/app.ts
squid review utils.js -m "Check for memory leaks"
```

### HTML Reviews

**What it checks:**
- Semantic HTML elements
- Accessibility (ARIA, alt text, labels)
- Proper heading hierarchy
- Form best practices
- SEO considerations

**Example:**
```bash
squid review index.html
squid review components/form.html -m "Focus on accessibility"
```

### CSS Reviews

**What it checks:**
- Modern CSS features (Grid, Flexbox, Custom Properties)
- Performance optimizations
- Responsive design patterns
- Maintainability (naming conventions, organization)
- Accessibility (contrast, focus states)

**Example:**
```bash
squid review styles/main.css
squid review app.scss -m "How can I improve performance?"
```

## Command Options

### Basic Syntax
```bash
squid review <FILE> [OPTIONS]
```

### Options

- `<FILE>` - Path to the file to review (required)
- `-m, --message <MESSAGE>` - Optional message to guide the review focus
- `-s, --stream` - Stream the response in real-time

## Usage Examples

### Basic Review
```bash
squid review src/main.rs
```

### Streaming Output
```bash
squid review app.ts --stream
```

### Focused Review
```bash
# Security focus
squid review auth.js -m "Are there any security vulnerabilities?"

# Performance focus
squid review styles.css -m "What can I optimize for better performance?"

# Accessibility focus
squid review form.html -m "Check for accessibility issues"

# Error handling focus
squid review handler.rs -m "Review error handling patterns"
```

### Specific Questions
```bash
squid review utils.ts -m "Is this function testable?"
squid review layout.css -m "Is this responsive design approach good?"
squid review main.rs -m "Are there any potential memory leaks?"
```

## Best Practices

### 1. Use Specific Messages
Instead of generic requests, ask specific questions:
```bash
# ❌ Generic
squid review file.js -m "Review this"

# ✅ Specific
squid review file.js -m "Focus on error handling and edge cases"
```

### 2. Review Before Committing
Integrate code review into your workflow:
```bash
# Review changed files before commit
git diff --name-only | xargs -I {} squid review {}
```

### 3. Stream for Long Reviews
Use streaming for larger files to see progress:
```bash
squid review large-component.tsx --stream
```

### 4. Focus on One Aspect at a Time
For comprehensive reviews, run multiple focused passes:
```bash
squid review app.ts -m "Check for security issues"
squid review app.ts -m "Review performance optimizations"
squid review app.ts -m "Assess testability"
```

## Testing the Review Command

Try the example files in the `sample-files/` directory:

```bash
# Test Rust review
cargo run -- review sample-files/example.rs

# Test TypeScript review
cargo run -- review sample-files/example.ts

# Test HTML review with accessibility focus
cargo run -- review sample-files/example.html -m "Focus on accessibility"

# Test CSS review with streaming
cargo run -- review sample-files/example.css --stream
```

See `sample-files/README.md` for detailed information about each example file.

## Configuration

The review command uses your configured LLM settings from your `.env` file:

```env
API_URL=http://127.0.0.1:1234/v1
API_KEY=not-needed
API_MODEL=local-model
```

## Tips

1. **Iterate on feedback**: Apply suggested changes and re-review to verify improvements
2. **Combine with linters**: Use alongside language-specific linters for comprehensive coverage
3. **Learn patterns**: Pay attention to recurring suggestions to improve your coding practices
4. **Version control**: Review code before staging changes to catch issues early

## Troubleshooting

### "Failed to read file"
- Check that the file path is correct
- Ensure you have read permissions for the file

### Slow responses
- Use `--stream` flag for real-time output
- Consider reviewing smaller files or specific functions
- Check your LLM API configuration and network connection

### Unexpected prompt
- Verify the file extension is correct
- Check the `get_review_prompt_for_file()` function in `main.rs` for supported extensions

## Advanced Usage

### Review Multiple Files
```bash
# Review all Rust files in src/
find src -name "*.rs" -exec squid review {} \;

# Review with specific focus
for file in src/*.ts; do
  squid review "$file" -m "Check for type safety"
done
```

### Integration with CI/CD
```bash
# In your CI pipeline
squid review src/main.rs > review-output.txt
# Parse review-output.txt for critical issues
```

### Custom Workflows
```bash
# Pre-commit hook
#!/bin/bash
staged_files=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(rs|ts|js)$')
for file in $staged_files; do
  echo "Reviewing $file..."
  squid review "$file" -m "Quick pre-commit check"
done
```

## Future Enhancements

Planned improvements:
- Additional language-specific prompts (Go, Java, C++, etc.)
- Batch review mode for multiple files
- Output formatting options (JSON, Markdown, HTML)
- Severity levels for issues
- Integration with popular IDEs

## Contributing

To add support for a new file type:

1. Create a new prompt file in `src/assets/review-{language}.md`
2. Add the constant in `main.rs`: `const CODE_REVIEW_{LANG}_PROMPT: &str = include_str!(...)`
3. Update `get_review_prompt_for_file()` to handle the new extension
4. Add example file(s) in `sample-files/`
5. Update documentation

See the existing prompts for structure and style guidelines.