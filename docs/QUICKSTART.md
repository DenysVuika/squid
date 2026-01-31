# Quick Start Guide - File Context Feature

Welcome! This guide will get you up and running with the file context feature in under 5 minutes.

## 1. Prerequisites

- Rust installed (for building)
- LM Studio running locally OR OpenAI API key

## 2. Build the Project

```bash
cd squid
cargo build --release
```

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

### Without a file (basic usage):

```bash
cargo run -- ask "What is Rust?"
```

### With a file (new feature!):

```bash
cargo run -- ask --file docs/sample.txt "What is this document about?"
```

That's it! 🎉

## 5. Common Use Cases

### Understand Code

```bash
cargo run -- ask -f src/main.rs "What does this code do?"
```

### Analyze Documents

```bash
cargo run -- ask -f README.md "Summarize this project"
```

### Get Help with Config

```bash
cargo run -- ask -f Cargo.toml "What dependencies does this use?"
```

### Use Streaming for Long Responses

```bash
cargo run -- ask -f large_file.txt -s "Provide a detailed analysis"
```

## 6. Command Syntax

```
squid ask [OPTIONS] <QUESTION>

Options:
  -s, --stream       Stream the response (real-time output)
  -f, --file <FILE>  Provide a file for context
  -h, --help         Print help
```

### Short Flags

```bash
# Long form
cargo run -- ask --file sample.txt --stream "Explain this"

# Short form (same thing)
cargo run -- ask -f sample.txt -s "Explain this"
```

## 7. Tips for Better Results

### ✅ Be Specific

```bash
# Good
cargo run -- ask -f code.rs "Explain the main function"

# Better
cargo run -- ask -f code.rs "Explain how error handling works in the main function"
```

### ✅ Use Appropriate File Types

Works great with:
- Source code (.rs, .py, .js, .go, etc.)
- Documentation (.md, .txt)
- Configuration (.toml, .json, .yaml)
- Data files (.csv, .log)

### ✅ Use Streaming for Long Content

```bash
# For detailed analysis or long files
cargo run -- ask -f big_document.md -s "Analyze this thoroughly"
```

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

- Check `EXAMPLES.md` for more advanced usage patterns
- Read `FILE_CONTEXT.md` to understand how it works
- See `README.md` for full documentation

## 10. Quick Reference

```bash
# Basic question
squid ask "question here"

# Question with file context
squid ask -f filename.txt "question here"

# Streaming response
squid ask -s "question here"

# File context + streaming
squid ask -f filename.txt -s "question here"

# Get help
squid ask --help
```

## Need More Help?

1. Enable debug logging:
   ```bash
   RUST_LOG=debug cargo run -- ask -f docs/sample.txt "test"
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