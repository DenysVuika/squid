# RAG (Retrieval-Augmented Generation) Guide

RAG enables squid to search through your documents and provide context-aware responses using semantic search.

## What is RAG?

RAG combines:
- **Retrieval**: Finding relevant information from your documents
- **Augmentation**: Adding that context to LLM prompts
- **Generation**: LLM generates informed responses

## Setting Up RAG

### 1. Prepare Documents

Place your documentation in the `documents/` folder:

```bash
mkdir documents
cp docs/*.md documents/
```

Supported formats:
- Markdown (`.md`)
- Text files (`.txt`)
- Source code (`.rs`, `.py`, `.js`, `.ts`, etc.)

### 2. Index Documents

Run the indexing command:

```bash
squid rag init
```

This will:
- Scan the `documents/` folder
- Split files into chunks (512 tokens each)
- Generate embeddings using `nomic-embed-text`
- Store vectors in SQLite database

### 3. Start Querying

Use the Web UI with RAG enabled:

```bash
squid serve
```

Toggle RAG mode in the chat interface to search your documents.

## RAG Commands

### Initialize Index

```bash
squid rag init
```

First-time indexing of all documents in `./documents/`

### List Documents

```bash
squid rag list
```

Shows all indexed documents with metadata:
- Filename
- Size
- Number of chunks
- Last updated

### Rebuild Index

```bash
squid rag rebuild
```

Clears and recreates the entire index. Useful after:
- Changing chunk size in config
- Major document updates
- Troubleshooting

### View Statistics

```bash
squid rag stats
```

Displays:
- Total documents
- Total chunks
- Average chunks per document
- Storage usage

## Example Queries

With RAG enabled, you can ask questions like:

- "How do I configure the API endpoint?" → Finds relevant config documentation
- "What are the available commands?" → Searches command reference
- "Show me authentication examples" → Retrieves code samples
- "Explain the RAG architecture" → Finds design docs

## How It Works

1. **Your Query**: "How do I use RAG?"
2. **Embedding**: Query converted to 768-dim vector
3. **Search**: sqlite-vec finds top 5 similar chunks
4. **Context**: Retrieved chunks added to LLM prompt
5. **Response**: LLM answers using your documentation

## Configuration

Customize RAG settings in `squid.config.json`:

```json
{
  "rag": {
    "enabled": true,
    "embedding_model": "nomic-embed-text",
    "embedding_url": "http://localhost:11434/v1",
    "chunk_size": 512,
    "chunk_overlap": 50,
    "top_k": 5
  }
}
```

## Best Practices

### Document Organization

- **Be specific**: Use clear, descriptive filenames
- **Stay focused**: One topic per document
- **Use headings**: Helps with chunking and context
- **Include examples**: Code samples improve retrieval

### Indexing

- **Initial index**: Run `squid rag init` before first use
- **Auto-update**: File watcher handles live changes during `squid serve`
- **Rebuild periodically**: Keeps index fresh

### Querying

- **Be specific**: "How to handle errors in Rust?" vs "errors"
- **Use keywords**: Include technical terms from your docs
- **Enable RAG**: Toggle RAG mode in Web UI for document search

## Troubleshooting

### "No documents found"

Check that:
- Documents are in `./documents/` folder
- Files have supported extensions
- You've run `squid rag init`

### "sqlite-vec extension not loaded"

Install the extension:
```bash
# See docs/SQLITE_VEC_SETUP.md for installation
```

### Poor search results

Try:
- More specific queries
- Adding more relevant documents
- Adjusting `top_k` in config
- Rebuilding index: `squid rag rebuild`

## Technical Details

- **Embedding Model**: nomic-embed-text (768 dimensions)
- **Chunk Size**: 512 tokens (configurable)
- **Chunk Overlap**: 50 tokens (prevents context loss)
- **Vector Store**: sqlite-vec (efficient KNN search)
- **Distance Metric**: Cosine similarity

## Next Steps

- Add your project documentation to `./documents/`
- Run `squid rag init` to index
- Try example queries in Web UI
- Adjust config based on your needs
