-- RAG vector storage with sqlite-vec
-- Version: 011
-- Description: Adds tables for document storage and vector embeddings for RAG features

-- Documents table
-- Stores original documents and their metadata
CREATE TABLE IF NOT EXISTS rag_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Create index on filename for fast lookups
CREATE INDEX IF NOT EXISTS idx_rag_documents_filename ON rag_documents(filename);
CREATE INDEX IF NOT EXISTS idx_rag_documents_hash ON rag_documents(content_hash);
CREATE INDEX IF NOT EXISTS idx_rag_documents_updated ON rag_documents(updated_at);

-- Document chunks table
-- Stores text chunks with references to parent documents
CREATE TABLE IF NOT EXISTS rag_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    chunk_tokens INTEGER NOT NULL,
    FOREIGN KEY (document_id) REFERENCES rag_documents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_rag_chunks_document ON rag_chunks(document_id);

-- Vector embeddings table using sqlite-vec
-- This is a virtual table provided by the sqlite-vec extension
-- Note: nomic-embed-text produces 768-dimensional vectors
CREATE VIRTUAL TABLE IF NOT EXISTS rag_embeddings USING vec0(
    chunk_id INTEGER PRIMARY KEY,
    embedding FLOAT[768]
);

-- Note: Foreign key constraint for chunk_id -> rag_chunks(id) is enforced at application level
-- sqlite-vec virtual tables don't support foreign key constraints directly
