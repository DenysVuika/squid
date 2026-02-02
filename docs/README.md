# Squid Documentation

Welcome to the squid CLI documentation! This directory contains comprehensive guides and references for using squid.

## Getting Started

New to squid? Start here:

- **[Quick Start Guide](QUICKSTART.md)** - Get up and running in 5 minutes
- **[Examples](EXAMPLES.md)** - Real-world usage examples and workflows

## Documentation

### User Guides

- **[QUICKSTART.md](QUICKSTART.md)** - Quick start guide for new users
  - Prerequisites and setup
  - Basic usage examples
  - Common troubleshooting
  - Tips for better results

- **[EXAMPLES.md](EXAMPLES.md)** - Comprehensive usage examples
  - Basic and advanced examples
  - Practical workflows
  - Best practices
  - File type support

- **[CHANGELOG.md](../CHANGELOG.md)** - Version history and release notes

## Resources

- **[sample.txt](../sample-files/sample.txt)** - Sample file for testing the file context feature

## Quick Reference

### Basic Commands

```bash
# Ask a question (required)
squid ask "What is Rust?"

# With additional context using -m
squid ask "Explain Rust" -m "Focus on memory safety"

# Ask with file context
squid ask -f sample-files/sample.txt "What is this about?"

# Stream the response
squid ask -s "Explain async/await"

# File context + streaming
squid ask -f src/main.rs -s "Explain this code"

# File with additional context
squid ask -f src/main.rs "Explain this" -m "Focus on error handling"

# Review code
squid review src/main.rs

# Review with streaming
squid review app.ts --stream

# Focused review
squid review styles.css -m "Focus on performance"
```

### Help

```bash
# General help
squid --help

# Command-specific help
squid ask --help
squid review --help
```

## File Context Feature

The file context feature allows you to provide files to the AI for analysis. Simply use the `--file` or `-f` flag:

```bash
squid ask --file path/to/file.txt "Your question about the file"
```

Supported file types:
- Source code (.rs, .py, .js, .go, etc.)
- Documentation (.md, .txt, .rst)
- Configuration (.toml, .json, .yaml)
- Data files (.csv, .tsv, .log)
- Any text-based file

## Configuration

Squid uses environment variables for configuration. Create a `.env` file in the project root:

```env
# For LM Studio (local)
API_URL=http://127.0.0.1:1234/v1
API_MODEL=local-model
API_KEY=not-needed

# For OpenAI
API_URL=https://api.openai.com/v1
API_MODEL=gpt-4
API_KEY=sk-your-key-here
```

## Need Help?

1. Check the [Quick Start Guide](QUICKSTART.md)
2. Browse the [Examples](EXAMPLES.md)
3. See code review examples in the main [README](../README.md) and [EXAMPLES.md](EXAMPLES.md)
4. Try the [example files](../sample-files/README.md) for testing
5. Enable debug logging: `RUST_LOG=debug squid ask ...`

## Contributing

Found an issue or have a suggestion? Check the documentation in this directory.

---

**Project**: squid 🦑  
**Repository**: [Back to main README](../README.md)