# RAG Quick Test Checklist

Use this checklist to verify RAG implementation before UI work.

## ✅ Pre-Test Setup

- [ ] Built the project: `cargo build --release`
- [ ] Have documents in `documents/` folder (4 demo files exist)
- [ ] Have `squid.config.json` with RAG config (or use .env)

## ✅ Test 1: Basic Commands (No Embedding Service)

These should work even without an embedding service:

```bash
# Should show help for RAG commands
cargo run --release -- rag --help

# Should fail gracefully (no embeddings service)
cargo run --release -- rag stats
```

**Expected**: Commands run, shows 0 documents or error about embedding service.

## ✅ Test 2: With Embedding Service

### Setup:
- [ ] LM Studio is running with `text-embedding-nomic-embed-text-v1.5` model
- [ ] Server started on port 11434 (or configured port)

### Commands:

```bash
# Index documents (this is the BIG test)
cargo run --release -- rag init

# Should show 4 documents
cargo run --release -- rag list

# Should show statistics
cargo run --release -- rag stats

# Test rebuild
cargo run --release -- rag rebuild
```

**Expected Output for `rag init`**:
```
🦑: Scanning documents directory: documents
🦑: Indexing complete!
    Files found: 4
    Files processed: 4
    Total chunks: ~45
    Total embeddings: ~45
```

## ✅ Test 3: API Endpoints

Start server:
```bash
cargo run --release -- serve
```

Test endpoints (in another terminal):

```bash
# Stats
curl http://localhost:8080/api/rag/stats | jq

# List docs
curl http://localhost:8080/api/rag/documents | jq

# Query
curl -X POST http://localhost:8080/api/rag/query \
  -H 'Content-Type: application/json' \
  -d '{"query": "How do I use RAG?"}' | jq
```

**Expected**: All endpoints return JSON with proper data.

## ✅ Test 4: Database Verification

```bash
sqlite3 squid.db "SELECT COUNT(*) FROM rag_documents;"
# Should show: 4

sqlite3 squid.db "SELECT COUNT(*) FROM rag_chunks;"
# Should show: ~45

sqlite3 squid.db "SELECT COUNT(*) FROM rag_embeddings;"
# Should show: ~45

sqlite3 squid.db "SELECT filename, file_size FROM rag_documents;"
# Should list all 4 documents
```

## 🚀 If All Tests Pass

You're ready to proceed with:
1. Web UI components (DocumentManager, RagQueryPanel)
2. Integration with chatbot
3. Unit tests

## 🐛 Common Issues

| Issue | Fix |
|-------|-----|
| "Failed to initialize RAG system" | Check LM Studio is running |
| "Documents directory not found" | Create `documents/` folder |
| "Failed to generate embedding" | Load embedding model in LM Studio |
| Database locked | Close any open sqlite3 sessions |
| Wrong port | Update `embedding_url` in config |

## ⚡ Quick Smoke Test (1 minute)

If you just want to verify it compiles and runs:

```bash
cargo build --release && \
cargo run --release -- rag --help && \
echo "✅ RAG commands available!"
```

## 📊 Success Criteria

- [x] Code compiles without errors ✅
- [ ] `squid rag init` successfully indexes documents
- [ ] `squid rag list` shows indexed files
- [ ] `squid rag stats` displays correct counts
- [ ] API endpoints return valid JSON
- [ ] Database tables contain data
- [ ] Queries return relevant context

Once all criteria are met, the backend is ready! 🎉
