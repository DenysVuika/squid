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
- [Tool Calling](#tool-calling)

## Ask Commands

### Ask a Question

Ask the AI assistant a question with optional context.

```bash
# Basic question (streaming by default)
squid ask "What is Rust?"

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
- `--no-stream` - Disable streaming, get complete response at once

### Ask About a File

Ask questions about a specific file's content.

```bash
# Basic file question (streams by default)
squid ask -f sample-files/sample.txt "What are the key features mentioned?"

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
- `--no-stream` - Disable streaming

## Review Command

Review code files with language-specific analysis.

```bash
# Review a file with language-specific prompts (streams by default)
squid review src/main.rs

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
# Start server on default port (3000)
squid serve

# Start on custom port
squid serve --port 8080

# Specify custom host
squid serve --host 0.0.0.0 --port 3000
```

The serve command starts:
- **Web UI** at `http://localhost:3000`
- **REST API** at `http://localhost:3000/api/*`
- **Health endpoint** at `http://localhost:3000/health`

**Options:**
- `-p, --port <PORT>` - Port to bind to (default: 3000)
- `-h, --host <HOST>` - Host to bind to (default: 127.0.0.1)

**API Endpoints:**
- `POST /api/chat` - Send chat messages
- `GET /api/sessions` - List chat sessions
- `GET /api/sessions/:id` - Get session details
- `DELETE /api/sessions/:id` - Delete session
- `PUT /api/sessions/:id/title` - Update session title
- `GET /api/logs` - Get application logs
- `GET /api/agents` - List available agents
- `POST /api/rag/index` - Index documents for RAG
- `GET /api/rag/documents` - List indexed documents
- And more...

See [REST API Documentation](../README.md#api-endpoints) for details.

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

Initialize Squid configuration for a project. Creates a `squid.config.json` file with your LLM connection settings and preferences.

### Interactive Mode (Default)

Run `squid init` to be prompted for all configuration values:

```bash
# Initialize in current directory
squid init

# Initialize in a specific directory
squid init ./my-project
squid init /path/to/project
```

**Interactive prompts:**
- **API URL**: The base URL for your LLM service (e.g., `http://127.0.0.1:1234/v1`)
- **API Model**: The model identifier (e.g., `local-model`, `qwen2.5-coder`, `gpt-4`)
- **API Key**: Optional API key (leave empty for local models like LM Studio or Ollama)
- **Context Window**: Maximum context window size in tokens (e.g., `32768`)
- **Log Level**: Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`)

**Example session:**
```
$ squid init
INFO: Initializing squid configuration in "."...
? API URL: http://127.0.0.1:1234/v1
? API Model: local-model
? API Key (optional, press Enter to skip): 
? Context Window (tokens): 32768
? Log Level: error

Configuration saved to: "squid.config.json"
  API URL: http://127.0.0.1:1234/v1
  API Model: local-model
  API Key: [not set]
  Context Window: 32768 tokens
  Log Level: error

✓ Default permissions configured
  Allowed: ["now"]

✓ Created .squidignore with default patterns
  Edit this file to customize which files squid should ignore
```

### Non-Interactive Mode

Provide configuration values via command-line arguments to skip interactive prompts:

```bash
# Initialize with all parameters
squid init --url http://127.0.0.1:1234/v1 --model local-model --log-level error

# Initialize in a specific directory with parameters
squid init ./my-project --url http://localhost:11434/v1 --model qwen2.5-coder --log-level error

# Partial parameters (will prompt for missing values)
squid init --url http://127.0.0.1:1234/v1 --model gpt-4
# Will still prompt for API Key and Log Level

# Include API key for cloud services
squid init --url https://api.openai.com/v1 --model gpt-4 --key sk-your-key-here --log-level error
```

**Available options:**
- `--url <URL>` - API URL (e.g., `http://127.0.0.1:1234/v1`)
- `--model <MODEL>` - API Model (e.g., `local-model`, `qwen2.5-coder`, `gpt-4`)
- `--key <KEY>` - API Key (optional for local models)
- `--context-window <SIZE>` - Context window size in tokens (e.g., `32768`)
- `--log-level <LEVEL>` - Log Level (`error`, `warn`, `info`, `debug`, `trace`)

### Re-running Init on Existing Config

When you run `squid init` on a directory that already has a config file, it will:
- Use existing values as defaults in prompts
- **Smart merge permissions**: Preserve your custom permissions + add new defaults
- Update version to match current app version

**Example:**
```
$ squid init --url http://127.0.0.1:1234/v1 --model local-model --log-level info
Found existing configuration, using current values as defaults...

Configuration saved to: "./squid.config.json"
  API URL: http://127.0.0.1:1234/v1
  API Model: local-model
  API Key: [configured]
  Log Level: info

✓ Added new default permissions: ["now"]

✓ Current tool permissions:
  Allowed: ["bash:git status", "bash:ls", "now"]
  Denied: ["write_file"]

✓ Using existing .squidignore file
```

In this example:
- User's existing permissions (`bash:git status`, `bash:ls`, `write_file` denial) are preserved
- New default permission (`now`) was automatically added
- Config version updated from 0.4.0 to 0.5.0

### Configuration File Format

The `squid.config.json` file created by `squid init`:

```json
{
  "api_url": "http://127.0.0.1:1234/v1",
  "api_model": "qwen2.5-coder",
  "context_window": 32768,
  "log_level": "error",
  "enable_env_context": true,
  "database_path": "squid.db",
  "agents": {
    "general-assistant": {
      "name": "General Assistant",
      "enabled": true,
      "description": "Full-featured coding assistant",
      "model": "qwen2.5-coder",
      "context_window": 32768,
      "permissions": {
        "allow": ["now", "read_file", "write_file", "grep", "bash:ls", "bash:git"],
        "deny": []
      }
    }
  },
  "default_agent": "general-assistant",
  "version": "0.7.0"
}
```

**Configuration options:**

| Field | Type | Description |
|-------|------|-------------|
| `api_url` | string | OpenAI-compatible API endpoint URL |
| `api_model` | string | Model identifier to use |
| `api_key` | string (optional) | API key for authentication |
| `context_window` | number | Maximum context window in tokens (global default; can be overridden per-agent) |
| `log_level` | string | Logging verbosity (error, warn, info, debug, trace) |
| `enable_env_context` | boolean | Include system info in prompts (default: true) |
| `database_path` | string | Path to SQLite database file |
| `agents` | object | Agent configurations (see Agents section in README) |
| `default_agent` | string | Default agent to use |
| `version` | string | Config file version |

### Alternative: Environment Variables

Instead of `squid.config.json`, you can create a `.env` file:

```bash
# OpenAI API Configuration
API_URL=http://127.0.0.1:1234/v1
API_MODEL=local-model
API_KEY=not-needed
CONTEXT_WINDOW=32768
DATABASE_PATH=squid.db
LOG_LEVEL=error

# Privacy Settings
ENABLE_ENV_CONTEXT=true
```

**Important Notes:**
- `squid.config.json` takes precedence over `.env` variables
- **Commit `squid.config.json`** to your repository to share project settings with your team
- **Keep `.env` private** - it should contain sensitive information like API keys and is excluded from git
- For cloud API services, store the actual API key in `.env` and omit `api_key` from `squid.config.json`

### Common API URLs

| Service | API URL |
|---------|---------|
| **LM Studio** | `http://127.0.0.1:1234/v1` |
| **Ollama** | `http://localhost:11434/v1` |
| **Docker Model Runner** | `http://localhost:12434/engines/v1` |
| **OpenAI** | `https://api.openai.com/v1` |
| **Mistral AI** | `https://api.mistral.ai/v1` |
| **OpenRouter** | `https://openrouter.ai/api/v1` |
| **Together AI** | `https://api.together.xyz/v1` |

## Tool Calling

The LLM has been trained to intelligently use tools when needed. It understands when to read, write, or search files based on your questions.

### Security Layers

1. **Path Validation** - Automatically blocks system directories (`/etc`, `/root`, `~/.ssh`, etc.)
2. **Ignore Patterns** - `.squidignore` file blocks specified files/directories (like `.gitignore`)
3. **User Approval** - Manual confirmation required for each operation

For details, see [Security Features](SECURITY.md).

### Examples

```bash
# LLM intelligently reads files when you ask about them
squid ask "Read the README.md file and summarize it"
squid ask "What dependencies are in Cargo.toml?"
squid ask "Analyze the main.rs file for me"
# You'll be prompted: "Allow reading file: [filename]? (Y/n)"

# LLM can write files
squid ask "Create a hello.txt file with 'Hello, World!'"
# You'll be prompted with a preview: "Allow writing to file: hello.txt?"

# Use custom prompts with tool calling
squid ask -p expert-coder.md "Read Cargo.toml and suggest optimizations"

# LLM can search for patterns in files using grep
squid ask "Search for all TODO comments in the src directory"
squid ask "Find all function definitions in src/main.rs"
squid ask "Search for 'API_URL' in the project"
squid ask "Find all uses of 'unwrap' in the codebase"
squid ask "Show me all error handling patterns in src/tools.rs"
# You'll be prompted: "Allow searching for pattern '...' in: [path]? (Y/n)"
# Results show file path, line number, and matched content

# LLM can get current date and time
squid ask "What time is it now?"
squid ask "What's the current date?"
# You'll be prompted: "Allow getting current date and time? (Y/n)"
# Returns datetime in RFC 3339 format

# LLM can execute safe bash commands
squid ask "What files are in this directory?"
squid ask "Show me the git status"
squid ask "List all .rs files in src/"
# You'll be prompted: "Allow executing bash command: [command]? (Y/n)"
# Dangerous commands (rm, sudo, chmod, dd, curl, wget, kill) are automatically blocked

# Use --no-stream for non-streaming mode
squid ask --no-stream "Read Cargo.toml and list all dependencies"
```

### Available Tools

- 📖 **read_file** - Read file contents from the filesystem
- 📝 **write_file** - Write content to files
- 🔍 **grep** - Search for patterns in files using regex (supports directories and individual files)
- 🕐 **now** - Get current date and time in RFC 3339 format (UTC or local timezone)
- 💻 **bash** - Execute safe, non-destructive bash commands (ls, git status, cat, etc.)

### Key Features

- 🤖 **Intelligent tool usage** - LLM understands when to read/write/search files from natural language
- 🛡️ **Path validation** - Automatic blocking of system and sensitive directories
- 📂 **Ignore patterns** - `.squidignore` file for project-specific file blocking
- 🔒 **Security approval** - All tool executions require user confirmation
- 📋 **Content preview** - File write operations show what will be written
- ⌨️ **Simple controls** - Press `Y` to allow or `N` to skip
- 📝 **Full logging** - All tool calls are logged for transparency
- 🔍 **Regex support** - Grep tool supports regex patterns with configurable case sensitivity
- 💻 **Bash execution** - Run safe, read-only commands for system inspection (dangerous commands **always** blocked, even with permissions)
- 🔐 **Privacy preserved** - With local models (LM Studio/Ollama), all file operations happen locally on your machine

### Using .squidignore

Create a `.squidignore` file in your project root to block specific files and directories:

```bash
# .squidignore - Works like .gitignore
*.log
.env
target/
node_modules/
__pycache__/
```

Patterns are automatically enforced - the LLM cannot access ignored files even if approved.

## See Also

- [Main README](../README.md) - Overview and Web UI documentation
- [Security Features](SECURITY.md) - Detailed security documentation
- [RAG Testing Guide](RAG_TESTING.md) - Testing RAG functionality
- [Examples](EXAMPLES.md) - More usage examples
- [Prompts Guide](PROMPTS.md) - Custom prompt development