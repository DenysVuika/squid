# squid 🦑

<div align="center">
  <img src="docs/squid.JPG" alt="Squid Logo" width="300" />
</div>

An AI-powered assistant for code reviews and improvement suggestions. Privacy-focused and local-first - your code never leaves your hardware when using local models.

> [!WARNING]
> This is an ongoing research project under active development. Features and APIs may change without notice, and breaking changes may occur between versions. Use in production at your own risk.

## Features

- 🤖 Chat with LLMs via OpenAI-compatible APIs
- 📄 Provide file context for AI analysis
- 🔍 AI-powered code reviews with language-specific prompts
- 🧠 **RAG (Retrieval-Augmented Generation)** - Semantic search over your documents for context-aware responses
- 🔧 Tool calling support (file read/write/search/bash operations) with multi-layered security
- 🌍 **Environment awareness** - LLM receives system context (OS, platform, timezone, timestamps) for smarter responses
- 🌐 **Web UI** - Built-in web interface for interacting with Squid
- 💾 **Persistent Sessions** - Chat history automatically saved and restored across page reloads and server restarts
- 📁 **Session Management** - Browse, load, rename, and delete past conversations with visual sidebar
- 🧠 **Reasoning Mode** - View LLM's thinking process with collapsible reasoning sections (supports `<think>` tags)
- 📊 **Database Logging** - Application logs stored in SQLite for debugging and troubleshooting
- 🔒 Path validation (whitelist/blacklist) and .squidignore support
- 🛡️ User approval required for all tool executions (read/write files)
- 🌊 Streaming support for real-time responses
- 🎨 **Enhanced UI** with styled prompts, emoji icons, color-coded information
- 🦑 Friendly squid assistant personality with professional responses
- ⚙️ Configurable via environment variables
- 🔌 Works with LM Studio, OpenAI, Ollama, Mistral, and other compatible services

## Privacy & Local-First

**Your code never leaves your hardware** when using local LLM services (LM Studio, Ollama, etc.).

- 🔒 **Complete Privacy** - Run models entirely on your own machine
- 🏠 **Local-First** - No data sent to external servers with local models
- 🛡️ **You Control Your Data** - Choose between local models (private) or cloud APIs (convenient)
- 🔐 **Secure by Default** - Multi-layered security prevents unauthorized file access

**Privacy Options:**
- **Maximum Privacy**: Use LM Studio or Ollama - everything runs locally, no internet required for inference
- **Cloud Convenience**: Use OpenAI or other cloud providers - data sent to their servers for processing
- **Your Choice**: Squid works with both - you decide based on your privacy needs

All file operations require your explicit approval, regardless of which LLM service you use.

## Prerequisites

**For Docker installation (recommended):** Only Docker Desktop 4.34+ or Docker Engine with Docker Compose v2.38+ is required. All AI models are automatically managed.

**For manual installation:** You'll need:

1. **Rust toolchain** (for building squid)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **An OpenAI-compatible LLM service** (choose one):

<details open>
<summary><b>Local LLM Options</b></summary>

Run AI models locally with these tools:

**LM Studio** (Recommended for GUI)
- User-friendly interface for running local LLMs
- Download from https://lmstudio.ai/
- Recommended model: `lmstudio-community/Qwen2.5-Coder-7B-Instruct-MLX-4bit`
- Default endpoint: `http://127.0.0.1:1234/v1`
- No API key required

**Ollama** (Lightweight CLI)
- Command-line tool for running LLMs
- Install: `brew install ollama` (macOS) or https://ollama.com/
- Recommended model: `ollama pull qwen2.5-coder`
- Default endpoint: `http://localhost:11434/v1`
- No API key required

**Docker Model Runner**
- Manage AI models through Docker
- Enable in Docker Desktop Settings → AI tab
- Pull models: `docker model pull hf.co/bartowski/Qwen2.5-Coder-7B-Instruct-GGUF:Q4_K_M`
- Default endpoint: `http://localhost:12434/engines/v1`
- No API key required

</details>

<details>
<summary><b>Cloud API Services (OpenAI-Compatible)</b></summary>

All these services use the standard OpenAI API format - just change the endpoint URL and API key:

**OpenAI**
- Endpoint: `https://api.openai.com/v1`
- API Key: https://platform.openai.com/api-keys
- Models: `gpt-4`, `gpt-4-turbo`, `gpt-3.5-turbo`, etc.

**Mistral AI**
- Endpoint: `https://api.mistral.ai/v1`
- API Key: https://console.mistral.ai/
- Models: `devstral-2512`, `mistral-large-latest`, `mistral-small-latest`, etc.

**Other Compatible Services**
- **OpenRouter** (https://openrouter.ai/) - Access to multiple LLM providers
- **Together AI** (https://together.ai/) - Fast inference
- **Anyscale** (https://anyscale.com/) - Enterprise solutions
- **Groq** (https://groq.com/) - Ultra-fast inference
- Any custom OpenAI-compatible endpoint

</details>

## Installation

### Docker with AI Models (Recommended)

The easiest way to get started - automated setup with helpful checks:

```bash
# Clone the repository
git clone https://github.com/DenysVuika/squid.git
cd squid

# Run the setup script (recommended)
chmod +x docker-setup.sh
./docker-setup.sh setup

# Or use Docker Compose directly
docker compose up -d
```

The setup script will:
- ✓ Verify Docker and Docker Compose versions
- ✓ Check Docker AI features are enabled
- ✓ Check available disk space (10GB+ recommended)
- ✓ Build Squid server image
- ✓ Pull AI models (~4.3GB total)
- ✓ Start all services with health checks

This automatically pulls and runs:
- **Squid server** (web UI + API) on http://localhost:3000
- **Qwen2.5-Coder 7B** (bartowski/Q4_K_M, ~4GB) - Main LLM
- **Nomic Embed Text v1.5** (~270MB) - Embeddings for RAG

**Requirements:**
- Docker Desktop 4.34+ with AI features enabled, or
- Docker Engine with Docker Compose v2.38+
- 10GB RAM available for Docker

**Apple Silicon:** Default config uses CPU inference (optimized for M1/M2/M3/M4). See `docker-compose.yml` for GPU options.

**Useful commands:**
```bash
./docker-setup.sh status    # Check service status
./docker-setup.sh logs      # View logs
./docker-setup.sh stop      # Stop services
./docker-setup.sh restart   # Restart services
./docker-setup.sh update    # Update models and images
```

### From crates.io

For manual installation with your own LLM service:

```bash
cargo install squid-rs
```

This installs the `squid` command globally. You'll need to configure it to connect to your LLM service (see Configuration section).

### From Source

Clone the repository and install locally:

```bash
git clone https://github.com/DenysVuika/squid.git
cd squid
cargo install --path .
```

### For Development

```bash
cargo build --release
```

For development, use `cargo run --` instead of `squid` in the examples below.

### Building the Web UI

The web UI is built separately and embedded into the binary. To build the complete application with the web UI:

```bash
# Build the web UI
cd web
npm install
npm run build

# The build output is automatically copied to ../static/
# Build the Rust application (which embeds the static files)
cd ..
cargo build --release
```

The `squid serve` command will then serve both the web UI and the API from the same server.

**Note:** If you're using a pre-built binary from crates.io or releases, the web UI is already included.

## Configuration

**Using Docker?** No configuration needed! Docker Compose automatically sets up everything. Skip to [Usage](#usage).

**Manual installation?** Configure squid to connect to your LLM service:

### Option 1: Interactive Setup (Recommended)

Use the `init` command to create a `squid.config.json` file:

#### Interactive Mode (Default)

```bash
# Initialize in current directory
squid init

# Initialize in a specific directory
squid init ./my-project
squid init /path/to/project
```

This will prompt you for:
- **API URL**: The base URL for your LLM service (e.g., `http://127.0.0.1:1234/v1`)
- **API Model**: The model identifier (e.g., `local-model`, `qwen2.5-coder`, `gpt-4`)
- **API Key**: Optional API key (leave empty for local models like LM Studio or Ollama)
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

**Re-running init on existing config:**

When you run `squid init` on a directory that already has a config file, it will:
- Use existing values as defaults in prompts
- **Smart merge permissions**: Preserve your custom permissions + add new defaults
- Update version to match current app version

```
$ squid init --url http://127.0.0.1:1234/v1 --model local-model --api-key "" --log-level info
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

#### Non-Interactive Mode

You can also provide configuration values via command-line arguments to skip the interactive prompts:

```bash
# Initialize with all parameters
squid init --url http://127.0.0.1:1234/v1 --model local-model --log-level error

# Initialize in a specific directory with parameters
squid init ./my-project --url http://localhost:11434/v1 --model qwen2.5-coder --log-level error

# Partial parameters (will prompt for missing values)
squid init --url http://127.0.0.1:1234/v1 --model gpt-4
# Will still prompt for API Key and Log Level

# Include API key for cloud services
squid init --url https://api.openai.com/v1 --model gpt-4 --api-key sk-your-key-here --log-level error
```

**Available options:**
- `--url <URL>` - API URL (e.g., `http://127.0.0.1:1234/v1`)
- `--model <MODEL>` - API Model (e.g., `local-model`, `qwen2.5-coder`, `gpt-4`)
- `--key <KEY>` - API Key (optional for local models)
- `--context-window <SIZE>` - Context window size in tokens (e.g., `32768`)
- `--log-level <LEVEL>` - Log Level (`error`, `warn`, `info`, `debug`, `trace`)

The configuration is saved to `squid.config.json` in the specified directory (or current directory if not specified). This file can be committed to your repository to share project settings with your team.

**Example `squid.config.json`:**
```json
{
  "api_url": "http://127.0.0.1:1234/v1",
  "api_model": "qwen2.5-coder",
  "context_window": 32768,
  "log_level": "error",
  "enable_env_context": true,
  "permissions": {
    "allow": ["now"],
    "deny": []
  },
  "database_path": "squid.db",
  "version": "0.7.0"
}
```

### Option 2: Manual Configuration

Create a `.env` file in the project root:

```bash
# OpenAI API Configuration (for LM Studio or OpenAI)
API_URL=http://127.0.0.1:1234/v1
API_MODEL=local-model
API_KEY=not-needed
CONTEXT_WINDOW=32768
DATABASE_PATH=squid.db

# Privacy Settings
ENABLE_ENV_CONTEXT=true
```

**Important Notes:**
- `squid.config.json` takes precedence over `.env` variables. If both exist, the config file will be used.
- **Commit `squid.config.json`** to your repository to share project settings with your team
- **Keep `.env` private** - it should contain sensitive information like API keys and is excluded from git
- For cloud API services (OpenAI, etc.), store the actual API key in `.env` and omit `api_key` from `squid.config.json`

### Configuration Options

- `API_URL`: The base URL for the OpenAI-compatible API endpoint
  - LM Studio: `http://127.0.0.1:1234/v1`
  - Ollama: `http://localhost:11434/v1`
  - Docker Model Runner: `http://localhost:12434/engines/v1`
  - OpenAI: `https://api.openai.com/v1`
  - Mistral AI: `https://api.mistral.ai/v1`
  - Other OpenAI-compatible services: Check provider's documentation
  
- `API_MODEL`: The model identifier to use (can be overridden in Web UI)
  - LM Studio/Ollama/Docker: Use the model name you loaded/pulled
  - OpenAI: `gpt-4`, `gpt-3.5-turbo`, etc.
  - Mistral AI: `devstral-2512`, `mistral-large-latest`, etc.
  - **Note**: The Web UI can fetch available models via the `/api/models` endpoint
  
- `API_KEY`: Your API key
  - Local services (LM Studio, Ollama, Docker): `not-needed`
  - Cloud services (OpenAI, Mistral, etc.): Your actual API key

- `CONTEXT_WINDOW`: Maximum context window size in tokens (optional, default: `8192`)
  - Used to calculate context utilization and prevent exceeding limits
  - Set via `squid init --context-window 32768` or in config file
  - See [Common Context Window Sizes](#common-context-window-sizes) below for popular models

- `LOG_LEVEL`: Logging verbosity (optional, default: `error`)
  - `error`: Only errors (default)
  - `warn`: Warnings and errors
  - `info`: Informational messages
  - `debug`: Detailed debugging information
  - `trace`: Very verbose output

- `DATABASE_PATH`: Path to the SQLite database file (optional, default: `squid.db`)
  - Used to store chat sessions, messages, and logs
  - Can be relative (e.g., `squid.db`) or absolute (e.g., `/path/to/squid.db`)
  - When relative, resolved based on:
    1. Config file location (if `squid.config.json` exists)
    2. Existing database in parent directories (searches upward)
    3. Current working directory (creates new database)
  - **Important**: The server automatically finds the correct database when running from subdirectories
  - Set via `.env` file to override automatic detection
  - Example: `DATABASE_PATH=/Users/you/squid-data/squid.db`

- `enable_env_context`: Include environment context in LLM prompts (optional, default: `true`)
  - When enabled, the LLM receives system information (OS, platform, timezone, timestamps) to provide more accurate responses
  - When disabled, no environment information is shared with the LLM (enhanced privacy)
  - Useful to disable when:
    - Using cloud-based LLM APIs where privacy is a concern
    - Working with sensitive projects that restrict system information sharing
    - Compliance requirements prevent sharing environmental data
    - Testing/debugging prompts without environmental variables
  - Set via `squid.config.json`:
    ```json
    {
      "enable_env_context": false
    }
    ```
  - Or via environment variable: `ENABLE_ENV_CONTEXT=false`
  - **Note**: Even when enabled, hostname and working directory are excluded by default for privacy

- `permissions`: Tool execution permissions (optional)
  - `allow`: Array of tool names that run without confirmation (default: `["now"]`)
  - `deny`: Array of tool names that are completely blocked (default: `[]`)
  - **Granular bash permissions**: Use `"bash:command"` format for specific commands
    - `"bash"` - allows all bash commands (dangerous patterns still blocked)
    - `"bash:ls"` - allows only `ls` commands (ls, ls -la, etc.)
    - `"bash:git status"` - allows only `git status` commands
  - ⚠️ **Important**: Dangerous bash commands (`rm`, `sudo`, `chmod`, `dd`, `curl`, `wget`, `kill`) are **always blocked** regardless of permissions
  - Example:
    ```json
    "permissions": {
      "allow": ["now", "read_file", "grep", "bash:ls", "bash:git status"],
      "deny": ["write_file", "bash:rm"]
    }
    ```
  - When prompted for tool approval, you can choose:
    - **Yes (this time)** - Allow once, ask again next time
    - **No (skip)** - Deny once, ask again next time
    - **Always** - Add to allow list and auto-save config (bash commands save as `bash:command`)
    - **Never** - Add to deny list and auto-save config (bash commands save as `bash:command`)
  - See [Security Documentation](docs/SECURITY.md#-tool-permissions-allowdeny-lists) for details

### Common Context Window Sizes

<details>
<summary><b>📊 Click to expand - Context window sizes for popular models</b></summary>

| Model | Context Window | Config Value |
|-------|---------------|--------------|
| **Qwen2.5-Coder-7B** | 32K tokens | `32768` |
| **GPT-4** | 128K tokens | `128000` |
| **GPT-4o** | 128K tokens | `128000` |
| **GPT-3.5-turbo** | 16K tokens | `16385` |
| **Claude 3 Opus** | 200K tokens | `200000` |
| **Claude 3.5 Sonnet** | 200K tokens | `200000` |
| **Llama 3.1-8B** | 128K tokens | `131072` |
| **Mistral Large** | 128K tokens | `131072` |
| **DeepSeek Coder** | 16K tokens | `16384` |
| **CodeLlama** | 16K tokens | `16384` |

**How to find your model's context window:**
1. Check your model's documentation on Hugging Face
2. Look in the model card or `config.json`
3. Check your LLM provider's documentation
4. For LM Studio: Look at the model details in the UI

**Why it matters:**
- ✅ Real-time utilization percentage (e.g., "45% of 32K context used")
- ✅ Prevents API errors from exceeding model capacity
- ✅ Accurate token usage statistics displayed in web UI
- ✅ Better planning for long conversations

**Example configuration:**
```bash
# For Qwen2.5-Coder with 32K context
squid init --context-window 32768

# For GPT-4 with 128K context
squid init --context-window 128000
```

</details>

## Usage

Squid provides both a modern Web UI and a command-line interface. **We recommend the Web UI** for the best experience.

### Squid Web UI (Recommended)

![Squid Web UI](docs/assets/screenshot.png)

*Modern chat interface with session management, token usage tracking, and real-time cost estimates*

Start the built-in web interface for Squid:

```bash
# Start Web UI on default port (8080)
squid serve

# Specify a custom port
squid serve --port 3000
squid serve -p 3000

# Use a custom database file
squid serve --db=/path/to/custom.db

# Use a custom working directory
squid serve --dir=/path/to/project

# Combine all options
squid serve --port 3000 --db=custom.db --dir=/path/to/project
```

The web server will:
- Launch the Squid Web UI at `http://127.0.0.1:8080` (or your specified port)
- Provide a browser-based interface for interacting with Squid
- Expose REST API endpoints for chat, sessions, and logs
- Display the server URL and API endpoint on startup

**Server Options:**
- `--port` / `-p`: Port number to run the server (default: `8080`)
- `--db`: Path to custom database file (default: `squid.db` in current/config directory)
- `--dir`: Working directory for the server (changes to this directory before starting)

**Use Cases:**
- Use `--db` to specify a different database file for separate projects or testing
- Use `--dir` to run the server in a specific project directory without navigating there first
- The database path is relative to the working directory (after `--dir` is applied)

**Web UI Features:**
- **Chat Page** - Interactive chat interface with session management sidebar
  - 📊 **Token usage indicator** - Real-time context utilization percentage (e.g., "5.6% • 7.1K / 128K")
  - 💰 **Cost tracking** - Displays estimated cost for both cloud and local models
  - 🗂️ **Session sidebar** - Browse and switch between past conversations
  - ✏️ **Auto-generated titles** - Sessions titled from first message, editable inline
  - 📎 **Multi-file attachments** - Add context from multiple files
- **Logs Page** - View application logs with pagination
  - 🔍 Filter by log level (error, warn, info, debug, trace)
  - 📄 Adjustable page size (25, 50, 100, 200 entries)
  - 🎨 Color-coded log levels and timestamps
  - 🔗 Session ID tracking for debugging

The web UI and API are served from the same server, so the chatbot automatically connects to the local API endpoint.

**Web UI Development (Hot Reload):**

For development with instant hot reloading:

```bash
# Terminal 1 - Backend server
cargo run serve --port 8080

# Terminal 2 - Frontend dev server
cd web && npm run dev
```

Then open `http://localhost:5173` in your browser. Changes to frontend code will appear instantly. The Vite dev server proxies API requests to the Rust backend.

To build for production: `cd web && npm run build` (outputs to `static/` directory).

### Command-Line Interface

For advanced users and automation, Squid provides a full CLI. See the [CLI Reference](docs/CLI.md) for detailed documentation on:

- **`squid ask`** - Ask questions with optional file context
- **`squid review`** - Review code with language-specific analysis
- **`squid rag`** - Manage RAG document indexing
- **`squid logs`** - View application logs
- **`squid init`** - Initialize project configuration

**Quick Examples:**

```bash
# Ask a question
squid ask "What is Rust?"

# Review a file
squid review src/main.rs

# Initialize RAG for a project
squid rag init

# View application logs
squid logs --level error
```

For complete CLI documentation, examples, and advanced usage, see [docs/CLI.md](docs/CLI.md).

### RAG (Retrieval-Augmented Generation)

Squid includes RAG capabilities for semantic search over your documents, enabling context-aware AI responses using your own documentation.

**Quick Start:**

```bash
# Docker (already configured)
docker compose up -d

# Or manually setup
mkdir documents
cp docs/*.md documents/
squid rag init
squid serve
# Click the RAG toggle (🔍) in the Web UI
```

**Features:**
- 📚 **Semantic search** over your documentation
- 🔍 **One-click toggle** in Web UI
- 💾 **Persistent knowledge base** - index once, query many times
- 📎 **Source attribution** - see which documents were used
- 🔄 **Auto-indexing** - supports Markdown, code, configs, and more

**Using RAG:**
1. Add documents to `./documents` directory
2. Run `squid rag init` to index them
3. Toggle RAG in the Web UI to enable semantic search
4. Ask questions about your documentation

For complete RAG documentation including configuration, API endpoints, best practices, and troubleshooting, see **[docs/RAG.md](docs/RAG.md)**.



**Database & Persistence:**
- All chat sessions, messages, and logs are automatically saved to `squid.db` (SQLite database)
- Sessions persist across server restarts - your conversation history is always preserved
- The database location is automatically detected:
  - If `squid.config.json` exists, database is stored relative to the config file
  - If no config file, searches parent directories for existing `squid.db`
  - Falls back to current directory if no database found
- You can override the location with `DATABASE_PATH` environment variable or in config file
- Run the server from any subdirectory - it will find and use the same database

Press `Ctrl+C` to stop the server.

#### API Endpoints

The web server exposes REST API endpoints for programmatic access:

**Chat Endpoint:** `POST /api/chat`

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

**Response:** Server-Sent Events (SSE) stream with JSON events:
```json
{"type": "content", "text": "response text chunk"}
{"type": "done"}
```

**Sessions Endpoints:**
- `GET /api/sessions` - List all sessions with metadata
- `GET /api/sessions/{id}` - Load full session history
- `DELETE /api/sessions/{id}` - Delete a session

**Logs Endpoint:** `GET /api/logs`

**Query Parameters:**
- `page` - Page number (default: 1)
- `page_size` - Entries per page (default: 50)
- `level` - Filter by level (error, warn, info, debug, trace)
- `session_id` - Filter by session ID

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

**Models Endpoint:** `GET /api/models`

Fetches available models from your LLM provider (LM Studio, Ollama, etc.) and augments them with metadata like context window sizes.

**Response:**
```json
{
  "models": [
    {
      "id": "qwen2.5-coder-7b-instruct",
      "name": "Qwen 2.5 Coder 7B Instruct",
      "max_context_length": 32768,
      "provider": "Qwen"
    },
    {
      "id": "llama-3.1-8b",
      "name": "Llama 3.1 8B",
      "max_context_length": 131072,
      "provider": "Meta"
    }
  ]
}
```

**Features:**
- Automatically fetches models from your LLM provider's `/models` endpoint
- Augments response with friendly names and context window sizes from built-in metadata
- Falls back to sensible defaults (8192 tokens) for unknown models
- Filters out embedding models
- Sorts with Qwen models first (preferred for coding)

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
      if (event.type === 'content') {
        console.log(event.text);
      }
    }
  }
}
```

See `web/src/lib/chat-api.ts` for a complete TypeScript client implementation.

**Note:** The chatbot UI is served from the same server as the API, so it automatically uses the relative path `/api/chat` without requiring any configuration.

#### Session Management API

The web server also provides REST endpoints for managing chat sessions:

**List all sessions:** `GET /api/sessions`

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

**Get session details:** `GET /api/sessions/{session_id}`

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

**Update a session (rename):** `PATCH /api/sessions/{session_id}`

**Request:**
```json
{
  "title": "My Custom Session Title"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Session updated successfully"
}
```

**Delete a session:** `DELETE /api/sessions/{session_id}`

**Response:**
```json
{
  "success": true,
  "message": "Session deleted successfully"
}
```

**Web UI Features:**
- Browse all conversations in the sidebar
- Sessions automatically titled from first user message
- Click any session to load its full history
- Rename sessions with inline edit dialog (pencil icon)
- Delete sessions with confirmation dialog
- Toggle sidebar visibility
- Sessions show title (or preview), message count, and last activity time

### Tool Calling & Security

Squid's LLM can intelligently use tools (read files, write files, search code, execute safe commands) when needed. All tool operations are protected by multiple security layers and require user approval.

**Security Features:**
- 🛡️ **Path Validation** - Blocks system directories automatically
- 📂 **Ignore Patterns** - `.squidignore` file (like `.gitignore`)
- 🔒 **User Approval** - Manual confirmation for each operation
- 💻 **Safe Bash** - Dangerous commands always blocked

**Available Tools:**
- 📖 **read_file** - Read file contents
- 📝 **write_file** - Write to files with preview
- 🔍 **grep** - Search code with regex
- 🕐 **now** - Get current date/time
- 💻 **bash** - Execute safe commands (ls, git, cat, etc.)

For complete security documentation and tool usage examples, see:
- [Security Features](docs/SECURITY.md) - Detailed security layers and best practices
- [CLI Reference](docs/CLI.md#tool-calling) - Tool calling examples and usage

## Documentation

- **[Quick Start Guide](docs/QUICKSTART.md)** - Get started in 5 minutes
- **[CLI Reference](docs/CLI.md)** - Complete command-line interface documentation
- **[RAG Guide](docs/RAG.md)** - Retrieval-Augmented Generation (semantic document search)
- **[Security Features](docs/SECURITY.md)** - Tool approval and security safeguards
- **[System Prompts Reference](docs/PROMPTS.md)** - Guide to all system prompts and customization
- **[Examples](docs/EXAMPLES.md)** - Comprehensive usage examples and workflows
- **[Changelog](CHANGELOG.md)** - Version history and release notes
- **[Sample File](sample-files/sample.txt)** - Test file for trying out the file context feature
- **[Example Files](sample-files/README.md)** - Test files for code review prompts
- **[AI Agents Guide](AGENTS.md)** - Instructions for AI coding assistants working on this project

### Testing

Try the code review and security features with the provided test scripts:

```bash
# Test code reviews (automated)
./tests/test-reviews.sh

# Test security approval (interactive)
./tests/test-security.sh

# Or test individual examples
squid review sample-files/example.rs
squid review sample-files/example.ts --stream
squid review sample-files/example.html -m "Focus on accessibility"
```

See **[tests/README.md](tests/README.md)** for complete testing documentation and **[sample-files/README.md](sample-files/README.md)** for details on each example file.



## Examples

<details open>
<summary><b>Using with LM Studio</b></summary>

1. Download and install LM Studio from https://lmstudio.ai/
2. Download the recommended model: `lmstudio-community/Qwen2.5-Coder-7B-Instruct-MLX-4bit`
3. Load the model in LM Studio
4. Start the local server (↔️ icon → "Start Server")
5. Set up your `.env`:
   ```bash
   API_URL=http://127.0.0.1:1234/v1
   API_MODEL=local-model
   API_KEY=not-needed
   ```
6. Run:
   ```bash
   squid ask "Write a hello world program in Rust"
   # Or with a file
   squid ask -f sample-files/sample.txt "What is this document about?"
   # Use --no-stream for complete response at once
   squid ask --no-stream "Quick question"
   ```

</details>

<details>
<summary><b>Using with Ollama</b></summary>

1. Install Ollama from https://ollama.com/
2. Start Ollama service:
   ```bash
   ollama serve
   ```
3. Pull the recommended model:
   ```bash
   ollama pull qwen2.5-coder
   ```
4. Set up your `.env`:
   ```bash
   API_URL=http://localhost:11434/v1
   API_MODEL=qwen2.5-coder
   API_KEY=not-needed
   ```
5. Run:
   ```bash
   squid ask "Write a hello world program in Rust"
   # Or with a file
   squid ask -f mycode.rs "Explain this code"
   # Use --no-stream if needed
   squid ask --no-stream "Quick question"
   ```

</details>

<details>
<summary><b>Using with OpenAI</b></summary>

1. Get your API key from https://platform.openai.com/api-keys
2. Set up your `.env`:
   ```bash
   API_URL=https://api.openai.com/v1
   API_MODEL=gpt-4
   API_KEY=sk-your-api-key-here
   ```
3. Run:
   ```bash
   squid ask "Explain the benefits of Rust"
   # Or analyze a file
   squid ask -f mycode.rs "Review this code for potential improvements"
   # Use --no-stream for scripting
   result=$(squid ask --no-stream "Generate a function name")
   ```

</details>

<details>
<summary><b>Using with Mistral API</b></summary>

1. Get your API key from https://console.mistral.ai/
2. Set up your `.env`:
   ```bash
   API_URL=https://api.mistral.ai/v1
   API_MODEL=devstral-2512
   API_KEY=your-mistral-api-key-here
   ```
3. Run:
   ```bash
   squid ask "Write a function to parse JSON"
   # Or use code review
   squid review myfile.py
   # Mistral models work great for code-related tasks
   ```

</details>

## License

Apache-2.0 License. See `LICENSE` file for details.
