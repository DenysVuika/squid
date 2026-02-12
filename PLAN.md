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
- **Phase 1: Session Persistence (SQLite)** ✅
  - Database schema with sessions, messages, and sources tables
  - SQLite integration with rusqlite
  - Automatic database migrations on startup
  - Write-through cache for optimal performance
  - Session CRUD operations with database backend
  - Persistent storage of all conversations
- **Phase 2.1: Basic Session API** ✅
  - GET `/api/sessions/{id}` endpoint for loading sessions
  - Session restoration on page reload
  - Full conversation history retrieval

## ✅ Phase 1: Session Persistence (SQLite) - COMPLETED

All tasks in this phase have been completed:
- ✅ Created `db.rs` with SQLite connection
- ✅ Implemented database CRUD operations
- ✅ Added automatic migration runner on startup
- ✅ Updated `SessionManager` to use database backend with write-through cache
- ✅ Added `database_path` to config (defaults to `squid.db`)
- ✅ Session and message persistence fully functional
- ✅ Sources (file attachments) stored and retrieved correctly
- ✅ Database initialization handled on first run

---

## Phase 2: Session Management API

### 2.1 REST Endpoints for Session Operations ✅ COMPLETED

**Completed Routes:**
- ✅ `GET /api/sessions/:id` - Get session details and full message history
- ✅ `GET /api/sessions` - List all sessions with metadata
- ✅ `DELETE /api/sessions/:id` - Delete session

**Completed Tasks:**
- ✅ Add routes to `main.rs` for list/delete operations
- ✅ Create handlers in `api.rs` for new endpoints
- ✅ Return session metadata (message count, preview, timestamps)
- ✅ Sort sessions by most recent activity

**Future Enhancements:**
- [ ] Add pagination for session lists
- [ ] Add filtering (by date, search)
- [ ] POST /api/sessions - Explicit session creation endpoint

### 2.2 Session Metadata ✅ COMPLETED

**Completed Features:**
- ✅ Auto-generate session title from first message
- ✅ Store title in database (title column added)
- ✅ Add user-editable session names (rename functionality)
- ✅ `PATCH /api/sessions/{id}` endpoint for renaming
- ✅ Inline edit dialog in session sidebar UI

**Future Enhancements:**
- [ ] Store token usage per session
- [ ] Track model used per session
- [ ] Session tags/categories

### 2.3 Web UI for Session Management ✅ COMPLETED

**Completed Features:**
- ✅ Session restoration on page reload (auto-loads last session)
- ✅ "New Chat" button to start fresh conversations
- ✅ Session ID persistence in localStorage
- ✅ `SessionList.tsx` - Sidebar with past sessions
- ✅ Click session to load conversation from sidebar
- ✅ Delete session with confirmation dialog
- ✅ Toggle sidebar visibility
- ✅ Session title display (auto-generated or custom)
- ✅ Session preview (first user message, truncated to 100 chars)
- ✅ Message count and last activity timestamp display
- ✅ Smart date formatting (time/day/date based on age)
- ✅ Auto-refresh session list on delete
- ✅ Auto-start new chat when deleting current session
- ✅ Rename session with inline edit dialog
- ✅ Edit button (pencil icon) on hover

**Future Enhancements:**
- [ ] Search sessions
- [ ] Manual sort options (by date/name)
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

### ✅ Completed
1. ✅ SQLite persistence (Phase 1)
2. ✅ Basic session GET API (Phase 2.1 partial)
3. ✅ Session restoration in Web UI

### ✅ Completed (Phase 2)
1. ✅ Complete session CRUD API (list, delete, update)
2. ✅ Web UI session list sidebar
3. ✅ Session management UI (delete, preview, load, rename)
4. ✅ Session metadata management (auto-generated titles, rename)

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

Phase 1 is complete! ✅

**Phase 2 is complete! ✅ All session management features implemented!**

```bash
# Test the new session management features:
cargo run -- serve

# Then visit http://127.0.0.1:3000
# - Try creating multiple chat sessions
# - Browse sessions in the sidebar
# - Load different sessions
# - Rename sessions with the edit button
# - Delete sessions
# - Notice auto-generated titles from first messages
```

**Suggested next steps (Phase 3):**
> "Implement session cleanup scheduler (Phase 3.1) with configurable retention policies"

or

> "Add context management (Phase 3.3) - smart context window limits and summarization"

or

> "Add token usage tracking per session (Phase 2.2 enhancement)"