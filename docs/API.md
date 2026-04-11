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

## Real-Time Updates (SSE)

### `GET /api/sessions/events`

Server-Sent Events stream for real-time session updates.

**Response (SSE stream):**
```json
{"type": "update", "session": {"session_id": "abc-123", "title": "...", ...}}
{"type": "deleted", "session_id": "abc-123"}
```

- Streams all session create/update/delete events in real-time
- Used by frontend to update sidebar without polling
- Connection persists across navigation

**Example using EventSource (JavaScript):**
```javascript
const eventSource = new EventSource('http://127.0.0.1:8080/api/sessions/events');
eventSource.addEventListener('session_update', (event) => {
  const data = JSON.parse(event.data);
  if (data.type === 'update') {
    console.log('Session updated:', data.session);
  } else if (data.type === 'deleted') {
    console.log('Session deleted:', data.session_id);
  }
});
```

### `GET /api/jobs/events`

Server-Sent Events stream for real-time job status updates.

**Response (SSE stream):**
```json
{"type": "update", "job": {"id": 1, "name": "...", "status": "running", ...}}
{"type": "deleted", "job_id": 1}
```

- Streams all job status changes (pending → running → completed/failed)
- Used by frontend to update job list and show notifications
- Triggered when jobs are created, updated, paused, resumed, or deleted

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

### `GET /api/agents/{agent_id}/content`

Fetch the raw markdown prompt content for a specific agent.

**Response:**
```json
{
  "id": "general-assistant",
  "name": "General Assistant",
  "content": "---\nname: General Assistant\n...\n---\n\nYou are a helpful coding assistant..."
}
```

**Errors:**
- `404` — Agent not found
- `500` — Failed to read agent file from disk

## Jobs

### `GET /api/jobs`

List all background jobs with their current status.

**Query Parameters:**

| Parameter | Default | Description                                                                            |
|-----------|---------|----------------------------------------------------------------------------------------|
| `status`  | —       | Filter by status (`pending`, `running`, `completed`, `failed`, `paused`, `cancelled`) |

**Response:**
```json
{
  "jobs": [
    {
      "id": 1,
      "name": "Daily Code Review",
      "schedule_type": "cron",
      "cron_expression": "0 9 * * *",
      "priority": 5,
      "max_cpu_percent": 50,
      "status": "pending",
      "last_run": "2026-04-10T08:00:00Z",
      "next_run": "2026-04-11T09:00:00Z",
      "is_active": true,
      "max_retries": 3,
      "timeout_seconds": 3600,
      "payload": {
        "agent_id": "code-reviewer",
        "prompt": "Review recent changes"
      },
      "created_at": "2026-04-01T00:00:00Z"
    }
  ]
}
```

### `POST /api/jobs`

Create a new background job.

**Request Body:**
```json
{
  "name": "Daily Code Review",
  "schedule_type": "cron",
  "cron_expression": "0 9 * * *",
  "priority": 5,
  "max_cpu_percent": 50,
  "max_retries": 3,
  "timeout_seconds": 3600,
  "payload": {
    "agent_id": "code-reviewer",
    "prompt": "Review recent changes in the repository"
  }
}
```

**Schedule Types:**
- `cron` — Recurring job with cron expression (requires `cron_expression`)
- `once` — One-time job executed immediately

**Response:**
```json
{
  "id": 1,
  "name": "Daily Code Review",
  "status": "pending",
  "created_at": "2026-04-10T10:00:00Z"
}
```

### `GET /api/jobs/{id}`

Get details for a specific job.

**Response:**
```json
{
  "id": 1,
  "name": "Daily Code Review",
  "schedule_type": "cron",
  "cron_expression": "0 9 * * *",
  "status": "completed",
  "last_run": "2026-04-10T09:00:00Z",
  "next_run": "2026-04-11T09:00:00Z",
  "payload": {
    "agent_id": "code-reviewer",
    "prompt": "Review recent changes"
  }
}
```

### `GET /api/jobs/{id}/executions`

Get execution history for a specific job.

**Response:**
```json
{
  "executions": [
    {
      "id": 1,
      "job_id": 1,
      "session_id": "abc-123-def-456",
      "status": "completed",
      "result": {"summary": "No issues found"},
      "error_message": null,
      "started_at": "2026-04-10T09:00:00Z",
      "completed_at": "2026-04-10T09:02:30Z",
      "duration_ms": 150000,
      "tokens_used": 1234,
      "cost_usd": 0.05
    }
  ]
}
```

### `POST /api/jobs/{id}/trigger`

Manually trigger a job to run immediately (creates a one-time execution).

**Response:**
```json
{
  "success": true,
  "message": "Job triggered successfully"
}
```

### `POST /api/jobs/{id}/pause`

Pause a scheduled job (prevents future automatic runs).

**Response:**
```json
{
  "success": true,
  "message": "Job paused successfully"
}
```

### `POST /api/jobs/{id}/resume`

Resume a paused job (re-enables automatic scheduling).

**Response:**
```json
{
  "success": true,
  "message": "Job resumed successfully"
}
```

### `POST /api/jobs/{id}/cancel`

Cancel a currently running job.

**Response:**
```json
{
  "success": true,
  "message": "Job cancellation requested"
}
```

### `DELETE /api/jobs/{id}`

Delete a job and all its execution history.

**Response:**
```json
{
  "success": true,
  "message": "Job deleted successfully"
}
```

**Note:** This permanently removes the job and cannot be undone.
