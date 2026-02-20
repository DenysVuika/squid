# Getting Started with Squid 🦑

Welcome to squid! This AI-powered tool helps you with code reviews, questions, and semantic document search.

## Quick Start

You've just initialized squid! Here's what to do next:

### 1. Start the Server

```bash
squid serve
```

Access the web interface at `http://localhost:8080`

### 2. Try These Commands

Ask a question:
```bash
squid ask "How do I handle errors in Rust?"
```

Review code:
```bash
squid review src/main.rs
```

### 3. Enable Document Search

Index your documentation:
```bash
squid rag init
```

Now you can ask questions about your docs!

## Configuration

Your `squid.config.json` file controls:
- API endpoint and model
- Context window size
- Tool permissions
- RAG settings

## Adding Documents

Place documentation in the `./documents/` folder:

```bash
mkdir documents
cp README.md documents/
squid rag init
```

## Privacy

When using local models (LM Studio, Ollama):
- Your code **never leaves your machine**
- No data sent to external servers
- Complete privacy guaranteed

## Next Steps

1. Check out `rag-guide.md` to learn about document search
2. Read `example-project.md` for a sample use case
3. Add your own documentation to `./documents/`
4. Run `squid rag init` to index your docs
5. Start chatting in the Web UI!

## Getting Help

- Run `squid --help` for command reference
- Check the GitHub repository for documentation
- Report issues on GitHub

Happy coding! 🦑
