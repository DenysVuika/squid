-- Deduplicate and compress file contents
-- Version: 006
-- Description: Creates file_contents table for deduplication and makes sources.content nullable

-- File contents table
-- Stores unique file contents with compression
CREATE TABLE IF NOT EXISTS file_contents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_hash TEXT UNIQUE NOT NULL,
    content_compressed BLOB NOT NULL,
    original_size INTEGER NOT NULL,
    compressed_size INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

-- Recreate sources table with nullable content column
-- Step 1: Rename old table
ALTER TABLE sources RENAME TO sources_old;

-- Step 2: Create new table with nullable content
CREATE TABLE sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT,  -- Now nullable for compressed entries
    content_id INTEGER,  -- References file_contents
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (content_id) REFERENCES file_contents(id) ON DELETE RESTRICT
);

-- Step 3: Migrate existing data (preserve old content)
INSERT INTO sources (id, message_id, title, content, content_id)
SELECT id, message_id, title, content, NULL
FROM sources_old;

-- Step 4: Drop old table
DROP TABLE sources_old;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_file_contents_hash ON file_contents(content_hash);
CREATE INDEX IF NOT EXISTS idx_sources_message_id ON sources(message_id);
CREATE INDEX IF NOT EXISTS idx_sources_content_id ON sources(content_id);

-- Cleanup trigger: Remove orphaned file contents
CREATE TRIGGER IF NOT EXISTS cleanup_orphaned_contents
AFTER DELETE ON sources
WHEN OLD.content_id IS NOT NULL
BEGIN
    DELETE FROM file_contents
    WHERE id = OLD.content_id
    AND NOT EXISTS (
        SELECT 1 FROM sources WHERE content_id = OLD.content_id
    );
END;
