# Squid API Reference

This document provides examples of using squid's various features programmatically.

## REST API Endpoints

When running `squid serve`, the following endpoints are available:

### Chat

**POST** `/api/chat`

Stream chat completions with optional file attachments:

```json
{
  "message": "Review this code",
  "files": [
    {
      "filename": "api.rs",
      "content": "..."
    }
  ],
  "use_rag": true
}
```

### RAG Operations

**POST** `/api/rag/upload`

Upload a document for indexing:

```json
{
  "filename": "guide.md",
  "content": "..."
}
```

**GET** `/api/rag/documents`

List all indexed documents.

**DELETE** `/api/rag/documents/{id}`

Remove a document and its embeddings.

**POST** `/api/rag/query`

Query with RAG context:

```json
{
  "query": "How do I configure authentication?",
  "top_k": 5
}
```

### Sessions

**GET** `/api/sessions`

List all chat sessions.

**GET** `/api/sessions/{id}`

Get a specific session with messages.

**DELETE** `/api/sessions/{id}`

Delete a session.

## Tool System

Squid uses function calling for file operations:

### Available Tools

#### read_file

Read file contents:

```rust
{
  "path": "src/main.rs"
}
```

#### write_file

Write to a file:

```rust
{
  "path": "output.txt",
  "content": "Hello, world!"
}
```

#### grep

Search files with regex:

```rust
{
  "pattern": "fn main",
  "path": "src/"
}
```

#### bash

Execute safe commands:

```rust
{
  "command": "ls -la"
}
```

### Tool Approval

All tools require user approval by default. Configure in `squid.config.json`:

```json
{
  "permissions": {
    "allow": ["now", "bash:ls", "bash:git"],
    "deny": ["bash:rm"]
  }
}
```

## Environment Context

Squid automatically includes system context:

```json
{
  "os": "macos",
  "platform": "darwin",
  "timezone": "America/Los_Angeles",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

Disable with:

```json
{
  "enable_env_context": false
}
```

## Rust Examples

### Custom LLM Client

```rust
use async_openai::{Client, config::OpenAIConfig};

let config = OpenAIConfig::new()
    .with_api_base("http://localhost:1234/v1")
    .with_api_key("not-needed");

let client = Client::with_config(config);
```

### RAG Integration

```rust
use rig_core::embeddings::EmbeddingModel;

let embedding_model = rig_core::client::OpenAi::new(
    "http://localhost:11434/v1".to_string(),
    "your-api-key".to_string()
).embedding_model("nomic-embed-text");

let embeddings = embedding_model
    .create_embeddings(vec![text.clone()])
    .await?;
```

### Database Operations

```rust
use rusqlite::Connection;

let conn = Connection::open("squid.db")?;

// Query sessions
let mut stmt = conn.prepare(
    "SELECT id, created_at FROM sessions ORDER BY updated_at DESC"
)?;
```

## Security Notes

### Path Validation

All file paths are validated:

- Must be within project directory
- Respects `.squidignore` patterns
- No system paths allowed
- Symlinks checked

### Tool Permissions

Granular control:

- `"bash"` - Allow all bash commands
- `"bash:git"` - Only git commands
- `"bash:git status"` - Only git status

### API Keys

Never commit API keys:

```bash
# Use environment variables
export API_KEY=your-key-here

# Or .env file (gitignored)
echo "API_KEY=your-key" > .env
```

## Error Handling

### Common Errors

**401 Unauthorized**
```json
{
  "error": "Invalid API key"
}
```

**404 Not Found**
```json
{
  "error": "Session not found"
}
```

**500 Internal Server Error**
```json
{
  "error": "Database connection failed"
}
```

### Retry Logic

Implement exponential backoff:

```rust
let mut retries = 0;
let max_retries = 3;

while retries < max_retries {
    match api_call().await {
        Ok(result) => return Ok(result),
        Err(e) if retries < max_retries - 1 => {
            tokio::time::sleep(Duration::from_secs(2_u64.pow(retries))).await;
            retries += 1;
        }
        Err(e) => return Err(e),
    }
}
```

## Performance Tips

### Streaming

Always use streaming for better UX:

```typescript
const response = await fetch('/api/chat', {
  method: 'POST',
  body: JSON.stringify({ message, stream: true }),
});

for await (const chunk of response.body) {
  // Process chunks
}
```

### Token Management

Monitor token usage:

```json
{
  "total_tokens": 1234,
  "input_tokens": 800,
  "output_tokens": 434,
  "context_utilization": 0.038
}
```

### RAG Optimization

- Adjust `chunk_size` based on model context window
- Use `top_k: 3-5` for optimal context
- Rebuild index periodically

## More Information

- GitHub: https://github.com/DenysVuika/squid
- Documentation: See `docs/` folder
- Issues: Report bugs on GitHub
