# Getting Started with Squid 🦑

Squid is an AI-powered command-line tool for code reviews, assistance, and now **semantic document search** with RAG (Retrieval-Augmented Generation).

## Quick Start

### Initialize Your Project

```bash
squid init
```

Follow the interactive prompts to configure:
- API endpoint (LM Studio, OpenAI, etc.)
- Model selection
- RAG document search setup
- Demo documents (optional)

### Start the Web UI

```bash
squid serve
```

Access the web interface at `http://localhost:8080`

## Basic Commands

### Ask Questions

```bash
squid ask "How do I implement error handling in Rust?"
```

Add file context:
```bash
squid ask "Review this code" --file src/main.rs
```

### Code Review

```bash
squid review src/api.rs
```

With specific questions:
```bash
squid review src/api.rs --message "Check for security issues"
```

### RAG Commands

Index your documents:
```bash
squid rag init
```

List indexed documents:
```bash
squid rag list
```

View statistics:
```bash
squid rag stats
```

## Configuration

Squid looks for `squid.config.json` in your project directory:

```json
{
  "api_url": "http://127.0.0.1:1234/v1",
  "api_model": "qwen2.5-coder-7b-instruct",
  "context_window": 32768,
  "log_level": "info"
}
```

## Privacy & Security

- **Local-first**: Your code never leaves your machine when using local models
- **User approval**: All file operations require explicit confirmation
- **Path validation**: `.squidignore` prevents access to sensitive files

## Next Steps

- Place documentation in `./documents/` folder
- Run `squid rag init` to enable semantic search
- Use the Web UI for interactive conversations
- Check out the RAG Guide for advanced features
