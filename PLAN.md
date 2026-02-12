# Squid Development Plan

## Current Architecture Status

✅ **Completed:**
- Session management with in-memory storage
- Backend-controlled chat history
- File attachment tracking and sources
- Streaming API with session support
- Thin client architecture (web UI)
- Multi-file support per message
- Session ID generation and tracking

## Phase 1: Session Persistence (SQLite)

### 1.1 Database Schema Setup

**Files to Create/Modify:**
- `squid/src/db.rs` - Database module
- `squid/migrations/` - SQL migration files
- `Cargo.toml` - Add dependencies

**Dependencies to Add:**
```toml
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "sqlite"] }
# Or use rusqlite for simpler approach:
rusqlite = { version = "0.32", features = ["bundled"] }
```

**Database Schema:**
```sql
-- sessions table
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    metadata TEXT -- JSON for future extensibility
);

-- messages table
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL, -- 'user' or 'assistant'
    content TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- sources table
CREATE TABLE sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

-- indexes
CREATE INDEX idx_sessions_updated_at ON sessions(updated_at);
CREATE INDEX idx_messages_session_id ON messages(session_id);
CREATE INDEX idx_messages_timestamp ON messages(timestamp);
CREATE INDEX idx_sources_message_id ON sources(message_id);
```

**Tasks:**
- [ ] Create `db.rs` with connection pool
- [ ] Implement `DbSession` struct with CRUD operations
- [ ] Add migration runner
- [ ] Update `SessionManager` to use database as backend
- [ ] Add database path to config
- [ ] Handle database initialization on startup

### 1.2 Session Persistence Implementation

**Update `session.rs`:**
- [ ] Add `save_to_db()` method on `ChatSession`
- [ ] Add `load_from_db()` static method
- [ ] Implement lazy loading for large histories
- [ ] Add transaction support for atomic updates

**Update `SessionManager`:**
- [ ] Replace `HashMap` with database queries
- [ ] Keep memory cache for active sessions (optional)
- [ ] Implement write-through cache strategy
- [ ] Add batch operations for performance

### 1.3 Data Migration & Compatibility

**Tasks:**
- [ ] Create database if not exists on first run
- [ ] Handle schema versioning for future updates
- [ ] Implement data export/import (JSON format)
- [ ] Add backup/restore functionality

---

## Phase 2: Session Management API

### 2.1 REST Endpoints for Session Operations

**New API Routes:**
```
GET  /api/sessions           - List all sessions
GET  /api/sessions/:id       - Get session details
POST /api/sessions           - Create new session
DELETE /api/sessions/:id     - Delete session
GET  /api/sessions/:id/messages - Get session messages
```

**Implementation:**
- [ ] Add routes to `main.rs`
- [ ] Create handlers in `api.rs`
- [ ] Add pagination for session lists
- [ ] Add filtering (by date, search)
- [ ] Return session metadata (title, message count, last updated)

### 2.2 Session Metadata

**Features:**
- [ ] Auto-generate session title from first message
- [ ] Store token usage per session
- [ ] Track model used per session
- [ ] Add user-editable session names
- [ ] Session tags/categories

### 2.3 Web UI for Session Management

**New Components:**
- [ ] `SessionList.tsx` - Sidebar with past sessions
- [ ] `SessionItem.tsx` - Individual session preview
- [ ] `SessionControls.tsx` - New/delete/rename buttons
- [ ] Update `chatbot.tsx` to load sessions

**Features:**
- [ ] Click session to load conversation
- [ ] Delete session with confirmation
- [ ] Rename session
- [ ] Search sessions
- [ ] Sort by date/name
- [ ] Session export (download as JSON/Markdown)

---

## Phase 3: Advanced Features

### 3.1 Session Cleanup & Maintenance

**Background Tasks:**
- [ ] Implement session cleanup scheduler
- [ ] Add configurable retention policies
- [ ] Archive old sessions (compress/export)
- [ ] Database vacuum/optimization
- [ ] Session size limits

**Configuration:**
```toml
[session]
max_age_days = 30
max_sessions = 100
cleanup_interval_hours = 24
max_messages_per_session = 1000
```

### 3.2 Multi-User Support

**User Management:**
- [ ] Add `users` table
- [ ] Associate sessions with users
- [ ] Implement authentication (optional)
- [ ] User-specific session lists
- [ ] Shared sessions (collaboration)

**Schema Updates:**
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    created_at INTEGER NOT NULL
);

ALTER TABLE sessions ADD COLUMN user_id TEXT REFERENCES users(id);
```

### 3.3 Conversation Context Management

**Smart Context:**
- [ ] Implement context window limits
- [ ] Summarize old messages when context is full
- [ ] Keep recent messages + summary of older ones
- [ ] Token counting per session
- [ ] Context pruning strategies

**Features:**
- [ ] Auto-summarization of long conversations
- [ ] Manual context reset while keeping session
- [ ] Branch conversations (fork from any message)
- [ ] Merge related sessions

### 3.4 Enhanced File Handling

**Improvements:**
- [ ] Store file hashes to detect duplicates
- [ ] Reference files instead of duplicating content
- [ ] File versioning (track changes)
- [ ] Binary file support (images, PDFs)
- [ ] File search across sessions

**Schema:**
```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hash TEXT UNIQUE NOT NULL,
    filename TEXT NOT NULL,
    content BLOB NOT NULL,
    mime_type TEXT,
    size INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE message_files (
    message_id INTEGER NOT NULL,
    file_id INTEGER NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    PRIMARY KEY (message_id, file_id)
);
```

### 3.5 Export & Sharing

**Export Formats:**
- [ ] JSON (full session data)
- [ ] Markdown (readable format)
- [ ] HTML (styled, shareable)
- [ ] PDF (via headless browser)

**Sharing:**
- [ ] Generate shareable links
- [ ] Public/private sessions
- [ ] Read-only shared views
- [ ] Embed sessions in other apps

---

## Phase 4: Performance & Optimization

### 4.1 Database Optimization

**Tasks:**
- [ ] Add database connection pooling
- [ ] Implement prepared statements
- [ ] Add query result caching
- [ ] Optimize indexes based on query patterns
- [ ] Implement database compression for old sessions

### 4.2 Memory Management

**Strategies:**
- [ ] LRU cache for active sessions
- [ ] Lazy loading of message history
- [ ] Stream large responses to disk
- [ ] Configurable memory limits
- [ ] Session eviction policies

### 4.3 API Performance

**Improvements:**
- [ ] Add response compression (gzip)
- [ ] Implement HTTP caching headers
- [ ] Add rate limiting
- [ ] Request batching for multiple operations
- [ ] WebSocket support for real-time updates

---

## Phase 5: Testing & Quality

### 5.1 Unit Tests

**Coverage:**
- [ ] Session CRUD operations
- [ ] Message persistence
- [ ] Source tracking
- [ ] Context window management
- [ ] File handling

### 5.2 Integration Tests

**Scenarios:**
- [ ] Full chat flow with persistence
- [ ] Session lifecycle (create, update, delete)
- [ ] Concurrent session access
- [ ] Database migration testing
- [ ] Error handling and recovery

### 5.3 Performance Tests

**Benchmarks:**
- [ ] Session load time
- [ ] Message streaming latency
- [ ] Database query performance
- [ ] Memory usage under load
- [ ] Concurrent user support

---

## Phase 6: Documentation

### 6.1 User Documentation

**Content:**
- [ ] Session management guide
- [ ] File attachment best practices
- [ ] Export/import instructions
- [ ] Configuration options
- [ ] Troubleshooting common issues

### 6.2 Developer Documentation

**Content:**
- [ ] Database schema documentation
- [ ] API endpoint reference
- [ ] Architecture diagrams
- [ ] Contributing guidelines
- [ ] Testing guide

### 6.3 Migration Guides

**Documents:**
- [ ] Upgrading from in-memory to SQLite
- [ ] Database schema migrations
- [ ] Breaking changes changelog
- [ ] Backup and restore procedures

---

## Implementation Priority

### High Priority (Phase 1 & 2)
1. SQLite persistence
2. Basic session CRUD API
3. Web UI session list

### Medium Priority (Phase 3)
1. Session cleanup
2. Context management
3. Enhanced file handling

### Low Priority (Phase 4-6)
1. Multi-user support
2. Advanced export options
3. Performance optimizations
4. Comprehensive testing

---

## Configuration Management

### Future Config Options

```toml
[database]
path = "./squid.db"
connection_pool_size = 10

[session]
max_age_days = 30
max_sessions_per_user = 100
cleanup_interval_hours = 24
max_messages_per_session = 1000

[context]
max_tokens = 4096
summarize_threshold = 0.8
keep_recent_messages = 10

[files]
max_file_size_mb = 10
max_files_per_message = 5
store_duplicates = false

[performance]
cache_size_mb = 100
enable_compression = true
rate_limit_requests_per_minute = 60
```

---

## Notes

- Keep backward compatibility when possible
- Add feature flags for gradual rollout
- Consider using environment variables for sensitive config
- Plan for horizontal scaling (multiple server instances)
- Consider adding telemetry/analytics (opt-in)
- Keep security in mind (SQL injection, XSS, etc.)

---

## Next Immediate Steps

To start Phase 1:

```bash
# 1. Add database dependency
cd squid
cargo add rusqlite --features bundled

# 2. Create db module
touch src/db.rs

# 3. Create migrations directory
mkdir -p migrations
touch migrations/001_initial_schema.sql

# 4. Update main.rs to initialize database

# 5. Test with: cargo run -- serve
```

Then follow this command to begin implementation:
> "Follow the plan in PLAN.md, starting with Phase 1.1 - Database Schema Setup"