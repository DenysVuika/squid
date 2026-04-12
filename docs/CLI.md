# Squid CLI Reference

This document covers the command-line interface for Squid. For most users, we recommend using the [Web UI](../README.md#squid-web-ui) which provides a more intuitive experience.

> **Note:** The examples below use the `squid` command (after installation with `cargo install --path .`).  
> For development, replace `squid` with `cargo run --` (e.g., `cargo run -- ask "question"`).

## Table of Contents

- [Ask Commands](#ask-commands)
- [Review Command](#review-command)
- [Serve Command](#serve-command)
- [RAG Commands](#rag-commands)
- [Logs Command](#logs-command)
- [Init Command](#init-command)
- [Cleanup Command](#cleanup-command)
- [Tool Calling](#tool-calling)

## Ask Commands

### Ask a Question

Ask the AI assistant a question with optional context.

```bash
# Basic question (streaming by default, uses default agent)
squid ask "What is Rust?"

# Use a specific agent
squid ask "What is Rust?" --agent code-reviewer

# With additional context using -m
squid ask "Explain Rust" -m "Focus on memory safety"

# Use a custom system prompt
squid ask "Explain Rust" -p custom-prompt.md

# Disable streaming for complete response at once (useful for scripting)
squid ask "Explain async/await in Rust" --no-stream
```

By default, responses are streamed in real-time, displaying tokens as they are generated. Use `--no-stream` to get the complete response at once (useful for piping or scripting).

**Options:**
- `-m, --message <TEXT>` - Additional context message
- `-p, --prompt <FILE>` - Custom system prompt file
- `--agent <NAME>` - Agent to use (defaults to default_agent from config)
- `--no-stream` - Disable streaming, get complete response at once

### Ask About a File

Ask questions about a specific file's content.

```bash
# Basic file question (streams by default, uses default agent)
squid ask -f sample-files/sample.txt "What are the key features mentioned?"

# Use specific agent for file analysis
squid ask -f src/main.rs "What does this do?" --agent general-assistant

# With additional context using -m
squid ask -f src/main.rs "What does this do?" -m "Focus on error handling"

# Use a custom system prompt for specialized analysis
squid ask -f src/main.rs "Review this" -p expert-reviewer-prompt.md

# Disable streaming for complete response
squid ask -f code.rs --no-stream "Explain what this code does"
```

This will read the file content and include it in the prompt, allowing the AI to answer questions based on the file's content.

**Options:**
- `-f, --file <PATH>` - File to read and include in context
- `-m, --message <TEXT>` - Additional context message
- `-p, --prompt <FILE>` - Custom system prompt file
- `--agent <NAME>` - Agent to use (defaults to default_agent from config)
- `--no-stream` - Disable streaming

## Review Command

Review code files with language-specific analysis.

```bash
# Review a file with language-specific prompts (streams by default, uses default agent)
squid review src/main.rs

# Use a specific agent for review
squid review src/main.rs --agent code-reviewer

# Focus on specific aspects
squid review styles.css -m "Focus on performance issues"

# Get complete review at once (no streaming)
squid review app.ts --no-stream

# Review SQL files
squid review schema.sql
squid review migrations/001_create_users.ddl

# Review shell scripts
squid review deploy.sh
squid review scripts/backup.bash

# Review Docker files
squid review Dockerfile
squid review Dockerfile.prod

# Review Go files
squid review main.go
squid review pkg/server.go

# Review Java files
squid review Application.java
squid review controllers/UserController.java

# Review configuration files
squid review config.json
squid review docker-compose.yaml
squid review deployment.yml

# Review Makefiles
squid review Makefile
squid review Makefile.dev

# Review documentation
squid review README.md
squid review docs/API.markdown
```

### Supported File Types

The review command automatically selects the appropriate review prompt based on file type:

| File Type | Extensions | Focus Areas |
|-----------|-----------|-------------|
| **Rust** | `.rs` | Ownership, safety, idioms, error handling |
| **TypeScript/JavaScript** | `.ts`, `.js`, `.tsx`, `.jsx` | Type safety, modern features, security |
| **HTML** | `.html`, `.htm` | Semantics, accessibility, SEO |
| **CSS** | `.css`, `.scss`, `.sass` | Performance, responsive design, maintainability |
| **Python** | `.py`, `.pyw`, `.pyi` | PEP 8, security, performance, best practices |
| **SQL** | `.sql`, `.ddl`, `.dml` | Performance, security, correctness, best practices |
| **Shell Scripts** | `.sh`, `.bash`, `.zsh`, `.fish` | Security, robustness, performance, compliance |
| **Docker/Kubernetes** | `Dockerfile`, `Dockerfile.*` | Security, performance, reliability, best practices |
| **Go** | `.go` | Concurrency, performance, error handling, best practices |
| **Java** | `.java` | Performance, best practices, JVM specifics, Spring framework |
| **JSON** | `.json` | Security, correctness, performance, maintainability |
| **YAML** | `.yaml`, `.yml` | Security, correctness, performance, maintainability |
| **Makefile** | `Makefile`, `Makefile.*` | Correctness, portability, performance, security |
| **Markdown** | `.md`, `.markdown` | Structure, accessibility, consistency, content |
| **Other files** | - | Generic code quality and best practices |

**Options:**
- `-m, --message <TEXT>` - Additional review focus areas
- `--no-stream` - Disable streaming

## Serve Command

Start the Squid web server with REST API and Web UI.

```bash
squid serve                          # http://localhost:3000
squid serve --port 8080              # Custom port
squid serve --host 0.0.0.0 --port 3000  # LAN access
```

**Options:**
- `-p, --port <PORT>` — Port to bind to (default: 3000)
- `-h, --host <HOST>` — Host to bind to (default: 127.0.0.1)

The server launches the Web UI, REST API, and health endpoint. See [API.md](API.md) for full endpoint documentation.

## RAG Commands

Manage Retrieval-Augmented Generation (RAG) for semantic document search.

### Initialize RAG

Set up RAG for a project directory.

```bash
# Initialize RAG in current directory
squid rag init

# Initialize in specific directory
squid rag init /path/to/docs
squid rag init ./my-project
```

This creates a `.squid-rag` directory with vector database for semantic search.

### List Documents

View indexed documents.

```bash
# List all indexed documents
squid rag list

# List for specific directory
squid rag list /path/to/docs
```

### Rebuild Index

Rebuild the RAG index from scratch.

```bash
# Rebuild index for current directory
squid rag rebuild

# Rebuild for specific directory
squid rag rebuild /path/to/docs
```

Useful when:
- Documents have been updated
- Index is corrupted
- You want to re-process with new settings

### View Statistics

Show RAG index statistics.

```bash
# View stats for current directory
squid rag stats

# View stats for specific directory
squid rag stats /path/to/docs
```

Shows:
- Total documents indexed
- Total chunks created
- Storage size
- Last update time

### Supported File Types

RAG automatically indexes these file types:
- **Markdown**: `.md`, `.markdown`
- **Code**: `.rs`, `.ts`, `.js`, `.tsx`, `.jsx`, `.py`, `.go`, `.java`, `.c`, `.cpp`, `.h`, `.hpp`
- **Config**: `.json`, `.yaml`, `.yml`, `.toml`, `.ini`
- **Shell**: `.sh`, `.bash`, `.zsh`, `.fish`
- **Web**: `.html`, `.htm`, `.css`, `.scss`, `.sass`
- **SQL**: `.sql`, `.ddl`, `.dml`
- **Docker**: `Dockerfile`, `Dockerfile.*`
- **Make**: `Makefile`, `Makefile.*`
- **Text**: `.txt`

## Logs Command

View and manage application logs stored in the database.

### View Logs

```bash
# View recent logs (last 100 by default)
squid logs show

# View more logs
squid logs show --limit 100

# Filter by log level
squid logs show --level error
squid logs show --level warn
squid logs show --level info

# View logs for a specific session
squid logs show --session-id 72dd7601-7da4-4252-80f6-7012da923faf

# Combine filters
squid logs show --limit 20 --level error
```

**Options:**
- `-l, --limit <NUMBER>` - Number of logs to retrieve (default: 100)
- `--level <LEVEL>` - Filter by log level (error, warn, info, debug, trace)
- `--session-id <ID>` - Filter by session ID

### Clean Up Old Logs

```bash
# Remove logs older than 30 days (default)
squid logs cleanup

# Remove logs older than 7 days
squid logs cleanup --max-age-days 7

# Remove logs older than 90 days
squid logs cleanup --max-age-days 90
```

**Options:**
- `-m, --max-age-days <DAYS>` - Maximum age of logs to keep in days (default: 30)

This removes log entries older than the specified threshold, which is useful to:
- Reclaim database space on long-running servers
- Retain recent logs while discarding historical noise
- Automate log rotation (e.g. via a cron job)

### Clear Logs

```bash
# Clear all logs from the database
squid logs reset
```

This removes all log entries from the database, which can be useful to:
- Free up database space
- Start fresh after debugging
- Remove old logs that are no longer needed

**Warning:** This operation cannot be undone. All log entries will be permanently deleted.

The logs are stored in the SQLite database (`squid.db`) alongside your chat sessions. This makes it easy to:
- Debug issues by reviewing what happened during a session
- Track errors and warnings across server restarts
- Correlate logs with specific chat conversations
- Monitor application behavior over time

**Note:** Logs are automatically stored when running the `serve` command.

## Init Command

Initialize Squid configuration. Creates `squid.config.json` with LLM connection settings and default agents.

### Interactive Mode (Default)

```bash
squid init                        # Current directory
squid init ./my-project           # Specific directory
```

Prompts for API URL, API key, log level, and RAG setup. Creates an `agents/` folder with default agents (`general-assistant`, `code-reviewer`, `light`, `pirate`, `shakespeare`).

### Non-Interactive Mode

```bash
squid init --url http://127.0.0.1:1234/v1 --log-level error
squid init ./my-project --url http://localhost:11434/v1 --key sk-your-key --log-level info
```

**Options:** `--url <URL>`, `--key <KEY>`, `--log-level <LEVEL>`

**Re-running `squid init`** on an existing config preserves settings and uses current values as defaults.

Context windows and models are configured per-agent in `squid.config.json` after initialization.

### Configuration

The `squid.config.json` file created by `squid init`:

```json
{
  "api_url": "http://127.0.0.1:1234/v1",
  "context_window": 32768,
  "log_level": "error",
  "database_path": "squid.db",
  "default_agent": "general-assistant",
  "version": "0.13.0"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `api_url` | string | OpenAI-compatible API endpoint URL |
| `api_key` | string (optional) | API key for authentication |
| `context_window` | number | Max context tokens (global default; per-agent override available) |
| `log_level` | string | Logging verbosity |
| `database_path` | string | SQLite database path |
| `default_agent` | string | Default agent ID (loaded from `agents/` folder) |
| `version` | string | Config file version |

**Re-running `squid init`** on an existing config preserves your settings and uses current values as defaults.

**Alternative: `.env` file** — environment variables work, but `squid.config.json` takes precedence. Keep `.env` private (API keys), commit `squid.config.json` for team sharing.

See [Configuration](../README.md#configuration) in the main README for full details.

## Jobs Commands

Manage background jobs and scheduled tasks for automated AI agent sessions.

### List Jobs

View all jobs with their current status in a formatted table.

```bash
# List all jobs
squid jobs list

# Filter by status
squid jobs list --status active
squid jobs list --status paused

# Filter by type
squid jobs list --type cron
squid jobs list --type oneoff
```

The output shows:

- Active status indicator (● active, ○ paused)
- Job ID, name, type
- Current status (active/paused)
- Schedule (for cron jobs)
- Priority and retry count

**Options:**

- `--status <STATUS>` - Filter by status (active, paused)
- `--type <TYPE>` - Filter by type (cron, oneoff)

### Show Job Details

Display detailed information about a specific job.

```bash
squid jobs show <job-id>
```

Shows complete job configuration including:

- ID, name, type, status
- Schedule (for cron jobs)
- Agent and prompt information
- Priority, timeout, and retry settings
- Creation and execution timestamps

### Create Job

Create a new background job with either interactive prompts or command-line flags.

```bash
# Interactive mode (prompts for all settings)
squid jobs create

# Non-interactive with flags
squid jobs create \
  --name "Daily Code Review" \
  --agent code-reviewer \
  --schedule "0 9 * * *" \
  --prompt "Review recent changes" \
  --priority 5

# One-off job (no schedule)
squid jobs create \
  --name "Security Scan" \
  --agent security-auditor \
  --prompt "Scan for vulnerabilities" \
  --type oneoff
```

**Interactive mode features:**

- Agent selection from available agents in `agents/` folder
- Type selection (cron or one-off)
- Schedule validation for cron expressions (6-field format)
- Optional timeout and retry configuration

**Options:**

- `--name <NAME>` - Job name (required in non-interactive mode)
- `--agent <AGENT>` - Agent ID from agents folder (required)
- `--prompt <TEXT>` - Prompt text for the agent (required)
- `--schedule <CRON>` - Cron expression (6-field format, required for cron jobs)
- `--type <TYPE>` - Job type: `cron` or `oneoff` (default: cron)
- `--priority <NUM>` - Priority 1-10 (default: 5)
- `--timeout <SECS>` - Timeout in seconds (default: 300)
- `--max-retries <NUM>` - Maximum retry attempts (default: 3)

**Cron expression format (6 fields):**
```
sec min hour day month day_of_week
*   *   *    *   *     *

Examples:
  0 0 9 * * *        - Daily at 9:00 AM
  0 */15 * * * *     - Every 15 minutes
  0 0 0 * * MON      - Every Monday at midnight
  0 30 8 1 * *       - First day of month at 8:30 AM
```

### Delete Job

Remove a job from the system.

```bash
squid jobs delete <job-id>
```

This permanently deletes the job. The job will no longer execute.

### Pause Job

Temporarily pause a job without deleting it.

```bash
squid jobs pause <job-id>
```

Paused jobs remain in the system but will not execute until resumed. The schedule is preserved.

### Resume Job

Resume a previously paused job.

```bash
squid jobs resume <job-id>
```

The job will continue executing according to its schedule.

### Trigger Job

Manually trigger a job to execute immediately.

```bash
squid jobs trigger <job-id>
```

This runs the job once immediately, regardless of its schedule. The regular schedule is not affected.

**Use cases:**

- Test a new job before waiting for its schedule
- Run a periodic task on-demand
- Re-run a failed job

For detailed information about the jobs system architecture and database schema, see [JOBS.md](JOBS.md).

## Cleanup Command

Remove bundled assets (plugins and agents) extracted from the binary.

```bash
# Clean up extracted bundled assets
squid cleanup
```

This removes the `~/.local/share/squid/bundled/` directory containing plugins and agents that were extracted at runtime. Useful for:

- Cleaning up after `cargo uninstall squid-rs`
- Resetting bundled assets so they re-extract on next `squid serve`
- Freeing disk space from cached embedded files

**What it removes:**
- Extracted bundled plugins (`~/.local/share/squid/bundled/plugins/`)
- Extracted bundled agents (`~/.local/share/squid/bundled/agents/`)

**What it does NOT remove:**
- Your `agents/` directory in project directories
- Your `plugins/` directory in workspace or `~/.squid/plugins/`
- Configuration files, databases, or documents

## Tool Calling

The LLM can intelligently use tools when needed based on natural language.

### Available Tools

| Tool | Description |
|------|-------------|
| `read_file` | Read file contents |
| `write_file` | Write to files (with preview) |
| `grep` | Regex search across files |
| `now` | Get current date/time |
| `bash` | Execute safe commands (ls, git, cat, etc.) |

### Security Layers

1. **Path Validation** — Blocks system directories automatically
2. **`.squidignore`** — Project-specific file blocking (like `.gitignore`)
3. **User Approval** — Manual confirmation for each operation

```bash
# LLM reads files on request
squid ask "Read Cargo.toml and list dependencies"

# LLM writes files (you see a preview first)
squid ask "Create hello.txt with 'Hello, World!'"

# LLM searches with grep
squid ask "Find all TODO comments in src/"

# LLM runs safe bash
squid ask "What files are in this directory?"
```

**`.squidignore` example:**
```
*.log
.env
target/
node_modules/
```

For complete security documentation, see [SECURITY.md](SECURITY.md). For detailed tool usage examples, see [EXAMPLES.md](EXAMPLES.md).

## See Also

- [Main README](../README.md) — Overview and Web UI documentation
- [Security Features](SECURITY.md) — Detailed security documentation
- [Examples](EXAMPLES.md) — More usage examples
- [Prompts Guide](PROMPTS.md) — Custom prompt development
- [API Reference](API.md) — REST API documentation