#!/bin/bash
set -e

echo "🦑 RAG Integration Test Script"
echo "================================"
echo ""

# Build the project
echo "📦 Step 1: Building squid..."
cargo build --release --quiet
echo "✅ Build successful"
echo ""

# Set up the binary path
SQUID="./target/release/squid"

# Create a test config if it doesn't exist
if [ ! -f "squid.config.json" ]; then
    echo "⚙️  Step 2: Creating test configuration..."
    cat > squid.config.json <<EOF
{
  "api_url": "http://127.0.0.1:1234/v1",
  "api_model": "qwen2.5-coder",
  "api_key": "not-needed",
  "context_window": 32768,
  "log_level": "info",
  "database_path": "squid.db",
  "permissions": {
    "allow": ["now"],
    "deny": []
  },
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
EOF
    echo "✅ Configuration created"
else
    echo "⚙️  Step 2: Using existing squid.config.json"
fi
echo ""

# Check if documents directory exists
if [ ! -d "documents" ]; then
    echo "❌ Error: documents/ directory not found"
    echo "   Please create it and add some documents first"
    exit 1
fi

# Count documents
DOC_COUNT=$(ls -1 documents/ | wc -l | tr -d ' ')
echo "📄 Found $DOC_COUNT documents in documents/ directory"
echo ""

# Test 1: Check RAG stats (before indexing)
echo "🧪 Test 1: Check initial RAG stats"
echo "-----------------------------------"
$SQUID rag stats 2>&1 || echo "⚠️  No documents indexed yet (expected)"
echo ""

# Test 2: Initialize RAG index
echo "🧪 Test 2: Index documents"
echo "-----------------------------------"
$SQUID rag init
echo ""

# Test 3: List indexed documents
echo "🧪 Test 3: List indexed documents"
echo "-----------------------------------"
$SQUID rag list
echo ""

# Test 4: Check RAG stats (after indexing)
echo "🧪 Test 4: Check RAG stats after indexing"
echo "-----------------------------------"
$SQUID rag stats
echo ""

# Test 5: Test RAG query via API (requires server running)
echo "🧪 Test 5: Test RAG API endpoints"
echo "-----------------------------------"
echo "To test the API endpoints, you need to:"
echo "  1. Start LM Studio with an embedding model (nomic-embed-text)"
echo "  2. Run: $SQUID serve"
echo "  3. In another terminal, test the API:"
echo ""
echo "     # Query for context"
echo "     curl -X POST http://localhost:8080/api/rag/query \\"
echo "       -H 'Content-Type: application/json' \\"
echo "       -d '{\"query\": \"How do I use RAG?\"}'"
echo ""
echo "     # Get stats"
echo "     curl http://localhost:8080/api/rag/stats"
echo ""
echo "     # List documents"
echo "     curl http://localhost:8080/api/rag/documents"
echo ""

# Summary
echo "================================"
echo "✅ RAG CLI Tests Complete!"
echo ""
echo "📊 Summary:"
echo "  - Documents directory: documents/"
echo "  - Documents found: $DOC_COUNT"
echo "  - Database: squid.db"
echo ""
echo "🚀 Next steps:"
echo "  1. Start an embedding service (LM Studio with text-embedding-nomic-embed-text-v1.5)"
echo "  2. Run: $SQUID serve"
echo "  3. Open http://localhost:8080"
echo "  4. Ask questions about your documents!"
echo ""
