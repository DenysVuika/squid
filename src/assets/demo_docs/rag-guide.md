# RAG (Retrieval-Augmented Generation) Quick Guide

RAG lets squid search your documents and provide informed answers using semantic search.

## What Can RAG Do?

With RAG enabled, you can:
- **Search documentation**: "How do I configure the API?"
- **Find examples**: "Show me authentication code"
- **Get context-aware answers**: Based on YOUR documentation

## Setup (3 Easy Steps)

### 1. Add Documents

Place your docs in the `documents/` folder:

```bash
# This folder was created during squid init
ls documents/
# getting-started.md  rag-guide.md  example-project.md
```

### 2. Index Documents

```bash
squid rag init
```

This generates embeddings and stores them for fast search.

### 3. Use RAG

Start the server:
```bash
squid serve
```

In the Web UI, toggle RAG mode and ask questions!

## Example Queries

Try asking:
- "How do I get started with squid?"
- "What RAG commands are available?"
- "Explain the example project setup"

Squid will find relevant sections from your documents and use them to answer.

## RAG Commands

```bash
squid rag init      # Index documents
squid rag list      # Show indexed docs
squid rag stats     # View statistics
squid rag rebuild   # Rebuild index
```

## How It Works

1. Your documents are split into chunks
2. Each chunk is converted to a vector (embedding)
3. When you ask a question, it's also converted to a vector
4. Similar chunks are found using vector search
5. The LLM uses those chunks to answer your question

## Supported File Types

- Markdown (`.md`)
- Text (`.txt`)
- Code (`.rs`, `.py`, `.js`, `.ts`, `.go`, etc.)

## Best Practices

✅ **Do:**
- Use clear, descriptive filenames
- Keep documents focused on one topic
- Include code examples
- Run `squid rag init` after adding new docs

❌ **Don't:**
- Add large binary files
- Include sensitive information
- Forget to rebuild after major changes

## Configuration

Edit `squid.config.json` to customize:

```json
{
  "rag": {
    "enabled": true,
    "chunk_size": 512,
    "top_k": 5
  }
}
```

## Troubleshooting

**No results found?**
- Check that documents are in `./documents/`
- Run `squid rag list` to see indexed files
- Try `squid rag rebuild`

**Poor results?**
- Use more specific queries
- Add more relevant documents
- Increase `top_k` in config

## Next Steps

- Add your project's README and docs
- Run `squid rag init` to index them
- Try asking questions in the Web UI
- Adjust settings based on results

RAG makes squid much more useful by giving it knowledge of YOUR project! 🎯
