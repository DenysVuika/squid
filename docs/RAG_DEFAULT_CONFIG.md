# Default RAG Configuration

The default embedding model is now set to `text-embedding-nomic-embed-text-v1.5`.

## Code Change

In `src/config.rs`:

```rust
fn default_embedding_model() -> String {
    "text-embedding-nomic-embed-text-v1.5".to_string()
}
```

## Default Configuration Values

When RAG is initialized without a config file, these are the defaults:

```json
{
  "rag": {
    "enabled": true,
    "embedding_model": "text-embedding-nomic-embed-text-v1.5",
    "embedding_url": "http://127.0.0.1:11434",
    "chunk_size": 512,
    "chunk_overlap": 50,
    "top_k": 5,
    "documents_path": "documents"
  }
}
```

## Using in LM Studio

1. Download the model in LM Studio:
   - Search for: `nomic-embed-text-v1.5`
   - Or the full path: `nomic-ai/nomic-embed-text-v1.5-GGUF`

2. Load the model and start the server

3. The model name to use in config is: `text-embedding-nomic-embed-text-v1.5`
   - This is the OpenAI-compatible API endpoint name

**Important**: The `embedding_url` should be the **base URL without `/v1`** suffix. 
- ✅ Correct: `http://127.0.0.1:11434`
- ❌ Wrong: `http://127.0.0.1:11434/v1`

The Rig library automatically appends `/v1/embeddings` to the base URL.

## Alternative Models

You can use any OpenAI-compatible embedding model by updating the config:

```json
{
  "rag": {
    "embedding_model": "text-embedding-3-small",  // OpenAI
    "embedding_url": "https://api.openai.com/v1"
  }
}
```

Or:

```json
{
  "rag": {
    "embedding_model": "mxbai-embed-large",  // Ollama
    "embedding_url": "http://localhost:11434/v1"
  }
}
```

## Model Naming Convention

The `text-embedding-` prefix follows OpenAI's API naming convention:
- OpenAI models: `text-embedding-3-small`, `text-embedding-3-large`, `text-embedding-ada-002`
- Nomic models in LM Studio: `text-embedding-nomic-embed-text-v1.5`

This ensures compatibility across different embedding providers.

## Summary

✅ Default model: `text-embedding-nomic-embed-text-v1.5`
✅ All documentation updated
✅ Test scripts updated
✅ Configuration examples updated
✅ Follows OpenAI API naming conventions

The model name matches what LM Studio's OpenAI-compatible API expects.

