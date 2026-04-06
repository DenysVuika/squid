# REST API Reference

The Squid web server exposes REST API endpoints for programmatic access. All endpoints are served from the same server as the Web UI.

## Chat

### `POST /api/chat`

Send a chat message and receive a Server-Sent Events (SSE) stream.

**Request Body:**
```json
{
  "message": "Your question here",
  "file_content": "optional file content",
  "file_path": "optional/file/path.rs",
  "system_prompt": "optional custom system prompt",
  "model": "optional model ID (overrides config default)"
}
```

**Response (SSE stream):**
```json
{"type": "content", "text": "response text chunk"}
{"type": "done"}
```

**Example using curl:**
```bash
curl -X POST http://127.0.0.1:8080/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Explain Rust async/await"}' \
  -N
```

**Example using fetch (JavaScript):**
```javascript
const response = await fetch('http://127.0.0.1:8080/api/chat', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ message: 'Explain async/await in Rust' })
});

const reader = response.body.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  const chunk = decoder.decode(value);
  const lines = chunk.split('\n');
  for (const line of lines) {
    if (line.startsWith('data: ')) {
      const event = JSON.parse(line.slice(6));
      if (event.type === 'content') console.log(event.text);
    }
  }
}
```

See `web/src/lib/chat-api.ts` for a complete TypeScript client implementation.

## Sessions

### `GET /api/sessions`

List all sessions with metadata.

**Response:**
```json
{
  "sessions": [
    {
      "session_id": "abc-123-def-456",
      "message_count": 8,
      "created_at": 1707654321,
      "updated_at": 1707658921,
      "preview": "Explain async/await in Rust",
      "title": "Async/await in Rust"
    }
  ],
  "total": 1
}
```

### `GET /api/sessions/{session_id}`

Load full session history.

**Response:**
```json
{
  "session_id": "abc-123-def-456",
  "messages": [
    {
      "role": "user",
      "content": "Explain async/await in Rust",
      "sources": [],
      "timestamp": 1707654321
    },
    {
      "role": "assistant",
      "content": "Async/await in Rust...",
      "sources": [{"title": "sample.rs"}],
      "timestamp": 1707654325
    }
  ],
  "created_at": 1707654321,
  "updated_at": 1707658921,
  "title": "Async/await in Rust"
}
```

### `PATCH /api/sessions/{session_id}`

Update a session (rename).

**Request:**
```json
{ "title": "My Custom Session Title" }
```

**Response:**
```json
{ "success": true, "message": "Session updated successfully" }
```

### `DELETE /api/sessions/{session_id}`

Delete a session.

**Response:**
```json
{ "success": true, "message": "Session deleted successfully" }
```

## Logs

### `GET /api/logs`

View application logs with pagination.

**Query Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `page` | 1 | Page number |
| `page_size` | 50 | Entries per page |
| `level` | — | Filter by level (`error`, `warn`, `info`, `debug`, `trace`) |
| `session_id` | — | Filter by session ID |

**Response:**
```json
{
  "logs": [
    {
      "id": 1,
      "timestamp": 1234567890,
      "level": "info",
      "target": "squid::api",
      "message": "Server started",
      "session_id": null
    }
  ],
  "total": 100,
  "page": 1,
  "page_size": 50,
  "total_pages": 655
}
```

## Agents

### `GET /api/agents`

Fetch available agents configured in `squid.config.json`.

**Response:**
```json
{
  "agents": [
    {
      "id": "general-assistant",
      "name": "General Assistant",
      "description": "Full-featured coding assistant with all tools available",
      "model": "qwen2.5-coder-7b-instruct",
      "enabled": true,
      "pricing_model": "gpt-4o",
      "suggestions": [
        "Read and summarize the main source files",
        "Show me the recent git log"
      ],
      "permissions": {
        "allow": ["now", "read_file", "write_file", "grep", "bash"]
      }
    }
  ],
  "default_agent": "general-assistant"
}
```

- Returns all enabled agents loaded from the `agents/` folder
- Each agent includes its model, description, and tool permissions (allow-only)
- Optional `pricing_model` field for cost estimation (useful for local models)
- Used by Web UI agent selector to display available assistants
