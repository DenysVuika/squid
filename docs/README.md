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

- **[REVIEW_GUIDE.md](REVIEW_GUIDE.md)** - AI-powered code review guide
  - Language-specific review prompts
  - File type support (Rust, TypeScript, HTML, CSS, etc.)
  - Usage examples and best practices
  - Testing with example files

- **[EXAMPLES.md](EXAMPLES.md)** - Comprehensive usage examples
  - Basic and advanced examples
  - Practical workflows
  - Best practices
  - File type support

### Technical Documentation

- **[FILE_CONTEXT.md](FILE_CONTEXT.md)** - File context feature architecture
  - How it works
  - Code architecture
  - Data flow diagrams
  - Performance considerations
  - Security considerations
  - Future enhancements

- **[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)** - Implementation details
  - What was implemented
  - Code changes
  - Technical details
  - Testing information
  - Deployment guide

### Project Information

- **[CHANGELOG.md](CHANGELOG.md)** - Version history and release notes
  - Feature additions
  - Changes and improvements
  - Technical details

## Resources

- **[sample.txt](sample.txt)** - Sample file for testing the file context feature

## Quick Reference

### Basic Commands

```bash
# Ask a question
squid ask "What is Rust?"

# Ask with file context
squid ask --file docs/sample.txt "What is this about?"

# Stream the response
squid ask -s "Explain async/await"

# File context + streaming
squid ask -f src/main.rs -s "Explain this code"

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
3. Read the [Code Review Guide](REVIEW_GUIDE.md) for code reviews
4. Review the [Technical Documentation](FILE_CONTEXT.md)
5. Try the [example files](../examples/README.md) for testing
6. Enable debug logging: `RUST_LOG=debug squid ask ...`

## Contributing

Found an issue or have a suggestion? The documentation is organized to be:
- **User-focused**: Start with QUICKSTART.md and EXAMPLES.md
- **Developer-focused**: Dive into FILE_CONTEXT.md and IMPLEMENTATION_SUMMARY.md
- **Maintainable**: Check CHANGELOG.md for version history

---

**Project**: squid 🦑  
**Repository**: [Back to main README](../README.md)