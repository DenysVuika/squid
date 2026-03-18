# Squid Documentation

Welcome to the squid documentation! This directory contains comprehensive guides and references for using squid.

## Getting Started

New to squid? Start here:

- **[Quick Start Guide](QUICKSTART.md)** - Get up and running in 5 minutes
- **[Examples](EXAMPLES.md)** - Real-world usage examples and workflows

## Documentation

### User Guides

- **[CLI.md](CLI.md)** - Complete command-line interface reference
  - All CLI commands and options
  - Tool calling and security
  - Usage examples and patterns

- **[RAG.md](RAG.md)** - RAG (Retrieval-Augmented Generation) guide
  - Semantic search over your documents
  - Setup and configuration
  - CLI commands and Web UI integration
  - API endpoints and best practices
  - Troubleshooting

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

- **[SECURITY.md](SECURITY.md)** - Security features and tool permissions
  - Multi-layered security architecture
  - Path validation and ignore patterns
  - User approval workflows
  - Configuration and best practices

- **[PROMPTS.md](PROMPTS.md)** - System prompts reference
  - Understanding the modular prompt system
  - Customizing prompts for specific tasks
  - Language-specific review prompts

- **[CHANGELOG.md](../CHANGELOG.md)** - Version history and release notes

## Resources

- **[sample.txt](../sample-files/sample.txt)** - Sample file for testing the file context feature

## Quick Reference

### Web UI (Recommended)

```bash
# Start the web server (Docker)
docker compose up -d
# Access at http://localhost:3000

# Or start manually
squid serve --port 3000
```

The Web UI provides:
- 💬 Interactive chat interface
- 📊 Real-time token usage tracking
- 💰 Cost estimates
- 🗂️ Session management
- 📎 Multi-file attachments
- 🔍 RAG toggle for semantic search

### CLI Commands

For CLI usage, see the **[CLI Reference](CLI.md)** for complete documentation.

Quick examples:

```bash
# Ask a question
squid ask "What is Rust?"

# Review code
squid review src/main.rs

# Initialize RAG
squid rag init

# View logs
squid logs --level error

# Start web server
squid serve --port 3000
```

## Configuration

**Using Docker?** No configuration needed - everything is set up automatically!

**Manual installation?** Use `squid init` for interactive setup, or see the main [README](../README.md#configuration) for configuration options.

## Need Help?

1. Check the [Quick Start Guide](QUICKSTART.md)
2. See the [CLI Reference](CLI.md) for command-line usage
3. Review the [Security Features](SECURITY.md) documentation
4. Browse the [Examples](EXAMPLES.md)
5. Try the [example files](../sample-files/README.md) for testing
6. Enable debug logging: `RUST_LOG=debug squid ask ...`

## Contributing

Found an issue or have a suggestion? Check the documentation in this directory.

---

**Project**: squid 🦑  
**Repository**: [Back to main README](../README.md)