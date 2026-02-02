# Quick Start Guide

Welcome! This guide will get you up and running with squid in under 5 minutes.

## 1. Prerequisites

- Rust installed (for building)
- LM Studio running locally OR OpenAI API key

## 2. Install squid

```bash
cd squid
cargo install --path .
```

This installs the `squid` command to your system. Alternatively, you can build it with `cargo build --release` and use `cargo run --` for development.

## 3. Configure Your Environment

Create a `.env` file:

### For LM Studio (Local - Recommended for Testing)

```bash
API_URL=http://127.0.0.1:1234/v1
API_MODEL=local-model
API_KEY=not-needed
```

### For OpenAI

```bash
API_URL=https://api.openai.com/v1
API_MODEL=gpt-4
API_KEY=sk-your-actual-api-key-here
```

## 4. Try Your First Command

### Ask a question:

```bash
squid ask "What is Rust?"
```

### Ask about a file:

```bash
squid ask -f sample-files/sample.txt "What is this document about?"
```

### Review code:

```bash
squid review src/main.rs
```

> **Development:** If you didn't install squid, use `cargo run --` instead of `squid`:
> ```bash
> cargo run -- ask "What is Rust?"
> ```

That's it! 🎉

## 5. Common Use Cases

### Ask Questions

```bash
# Basic question
squid ask "What is Rust?"

# With additional context
squid ask "Explain Rust" -m "Focus on memory safety"
```

### Analyze Files

```bash
squid ask -f src/main.rs "What does this code do?"
squid ask -f README.md "Summarize this project"
```

### Review Code

```bash
# Basic review
squid review src/main.rs

# Focused review
squid review src/auth.rs -m "Focus on security issues"

# With streaming
squid review components/App.tsx --stream
```

### Use Streaming for Real-Time Output

```bash
squid ask -f large_file.txt -s "Provide a detailed analysis"
squid review sample-files/example.rs -s
```

## 6. Command Syntax

### Ask Command

```
squid ask [OPTIONS] <QUESTION>

Arguments:
  <QUESTION>  The question to ask (required)

Options:
  -m, --message <MESSAGE>  Optional additional context or instructions
  -s, --stream             Stream the response (real-time output)
  -f, --file <FILE>        Provide a file for context
  -h, --help               Print help
```

### Review Command

```
squid review [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to the file to review (required)

Options:
  -m, --message <MESSAGE>  Optional additional message or specific question
  -s, --stream             Stream the response
  -h, --help               Print help
```

## 7. Tips for Better Results

### ✅ Use the Right Command

```bash
# For questions and analysis
squid ask "What is async/await in Rust?"
squid ask -f code.rs "Explain this code"

# For code reviews
squid review src/main.rs
squid review app.ts -m "Check for security issues"
```

### ✅ Be Specific with Context

```bash
# Good
squid ask -f code.rs "Explain the main function"

# Better - use -m for focus
squid ask -f code.rs "Explain the main function" -m "Focus on error handling"
squid review auth.rs -m "Focus on security vulnerabilities"
```

### ✅ Language-Specific Reviews

The `review` command automatically uses specialized prompts for:
- Rust (`.rs`) - Ownership, safety, idioms
- TypeScript/JavaScript (`.ts`, `.js`, `.tsx`, `.jsx`) - Type safety, modern features
- HTML (`.html`) - Semantics, accessibility
- CSS (`.css`, `.scss`) - Performance, responsive design

### ✅ Use Streaming for Long Content

```bash
squid ask -f big_document.md -s "Analyze this thoroughly"
squid review large_component.tsx --stream
```

### ✅ Tool Calling with Security

The LLM can read and write files when needed - you'll approve each action:

```bash
# LLM can read files (with your approval)
squid ask "Read the README.md and summarize it"
# You'll see: "Allow reading file: README.md? (Y/n)"

# LLM can write files (with preview and approval)
squid ask "Create a hello.txt file with 'Hello, World!'"
# You'll see the content preview and: "Allow writing to file: hello.txt?"
```

**Security features:**
- ✅ Every tool execution requires your approval
- ✅ File writes show content preview (first 100 bytes)
- ✅ Press `Y` to allow or `N` to skip
- ✅ All tool calls are logged

## 8. Troubleshooting

### "Failed to read file"
- Check the file path is correct
- Make sure you have read permissions
- Try using absolute path if relative doesn't work

### No response / Connection error
- **LM Studio**: Make sure it's running and server is enabled
- **OpenAI**: Check your API key is valid and has credits
- Verify API_URL in your `.env` file

### Response not relevant to file
- Make sure your question specifically asks about the file content
- Check the file was actually read (look for log message)
- Enable debug logging: `RUST_LOG=debug cargo run -- ask -f ...`

## 9. What's Next?

- Read `EXAMPLES.md` for more advanced usage patterns and workflows
- Check the code review section in `README.md` for review command details
- Try example files in `sample-files/` directory
- See `README.md` for full documentation
- Run `squid --help` to see all available commands and options

## 10. Quick Reference

```bash
# Ask questions
squid ask "question here"
squid ask "question" -m "additional context"
squid ask -f filename.txt "question here"
squid ask -f filename.txt -s "question here"

# Review code
squid review src/main.rs
squid review app.ts -m "Focus on security"
squid review styles.css --stream

# Tool calling (with approval prompts)
squid ask "Read README.md and summarize it"
squid ask "Create a notes.txt file with today's tasks"

# Get help
squid ask --help
squid review --help
```

## Need More Help?

1. Enable debug logging:
   ```bash
   RUST_LOG=debug squid ask -f sample-files/sample.txt "test"
   ```

2. Check your configuration:
   ```bash
   cat .env
   ```

3. Verify LM Studio is running:
   - Open LM Studio
   - Load a model
   - Enable local server (bottom left)
   - Default: `http://127.0.0.1:1234`

Happy coding! 🦑