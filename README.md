# squid 🦑

An AI-powered command-line tool for code reviews and suggestions. Privacy-focused and local-first - your code never leaves your hardware when using local models.

## Features

- 🤖 Chat with LLMs via OpenAI-compatible APIs
- 📄 Provide file context for AI analysis
- 🔍 AI-powered code reviews with language-specific prompts
- 🔧 Tool calling support (file read/write/search/bash operations) with multi-layered security
- 🌍 **Environment awareness** - LLM receives system context (OS, platform, timezone, timestamps) for smarter responses
- 🌐 **Web UI** - Built-in web interface for interacting with Squid
- 💾 **Persistent Sessions** - Chat history automatically saved and restored across page reloads and server restarts
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

Before you begin, you'll need:

1. **Rust toolchain** (for building squid)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **An OpenAI-compatible LLM service** (choose one):

<details open>
<summary><b>Option A: LM Studio (Recommended for Local Development)</b></summary>

[LM Studio](https://lmstudio.ai/) provides a user-friendly interface for running local LLMs.

1. **Download and install** LM Studio from https://lmstudio.ai/
2. **Download a model** - We recommend **Qwen2.5-Coder** for code-related tasks:
   - In LM Studio, search for: `lmstudio-community/Qwen2.5-Coder-7B-Instruct-MLX-4bit`
   - Or browse: https://huggingface.co/lmstudio-community/Qwen2.5-Coder-7B-Instruct-MLX-4bit
   - Click download and wait for it to complete
3. **Load the model** - Select the downloaded model in LM Studio
4. **Start the local server**:
   - Click the "Local Server" tab (↔️ icon on the left)
   - Click "Start Server"
   - Default endpoint: `http://127.0.0.1:1234/v1`
   - Note: No API key required for local server

**Alternative models in LM Studio:**
- `Meta-Llama-3.1-8B-Instruct` - General purpose
- `deepseek-coder` - Code-focused
- Any other model compatible with your hardware

</details>

<details>
<summary><b>Option B: Ollama (Lightweight CLI Option)</b></summary>

[Ollama](https://ollama.com/) is a lightweight, command-line tool for running LLMs.

1. **Install Ollama**:
   ```bash
   # macOS
   brew install ollama
   
   # Linux
   curl -fsSL https://ollama.com/install.sh | sh
   
   # Or download from https://ollama.com/
   ```

2. **Start Ollama service**:
   ```bash
   ollama serve
   ```

3. **Pull the recommended model** - **Qwen2.5-Coder**:
   ```bash
   ollama pull qwen2.5-coder
   ```
   - Model page: https://ollama.com/library/qwen2.5-coder
   - Available sizes: 0.5B, 1.5B, 3B, 7B, 14B, 32B
   - Default (7B) is recommended for most use cases

4. **Verify it's running**:
   ```bash
   ollama list  # Should show qwen2.5-coder
   curl http://localhost:11434/api/tags  # API check
   ```

**Alternative models in Ollama:**
- `codellama` - Code generation
- `deepseek-coder` - Code understanding
- `llama3.1` - General purpose
- See all at https://ollama.com/library

</details>

<details>
<summary><b>Option C: OpenAI API</b></summary>

Use OpenAI's cloud API for access to GPT models:

1. **Get an API key** from https://platform.openai.com/api-keys
2. **Add credits** to your OpenAI account
3. **Choose a model**: `gpt-4`, `gpt-4-turbo`, `gpt-3.5-turbo`, etc.

</details>

<details>
<summary><b>Option D: Mistral API</b></summary>

Use Mistral's cloud API for access to their powerful models:

1. **Get an API key** from https://console.mistral.ai/
2. **Choose a model**: `devstral-2512`, `mistral-large-latest`, `mistral-small-latest`, etc.
3. **Configure**: Mistral API is OpenAI-compatible, so it works seamlessly with Squid

</details>

<details>
<summary><b>Option E: Other OpenAI-Compatible Services</b></summary>

Squid works with any OpenAI-compatible REST API:
- **OpenRouter** (https://openrouter.ai/) - Access to multiple LLM providers
- **Together AI** (https://together.ai/) - Fast inference
- **Anyscale** (https://anyscale.com/) - Enterprise solutions
- **Local APIs** - Any custom OpenAI-compatible endpoint

</details>

## Installation

### From crates.io (Recommended)

```bash
cargo install squid-rs
```

This installs the `squid` command globally from crates.io. You can then use `squid` from anywhere.

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

You can configure squid in two ways:

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
? Log Level: error

Configuration saved to: "squid.config.json"
  API URL: http://127.0.0.1:1234/v1
  API Model: local-model
  API Key: [not set]
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
- `--api-key <KEY>` - API Key (optional for local models)
- `--log-level <LEVEL>` - Log Level (`error`, `warn`, `info`, `debug`, `trace`)

The configuration is saved to `squid.config.json` in the specified directory (or current directory if not specified). This file can be committed to your repository to share project settings with your team.

### Option 2: Manual Configuration

Create a `.env` file in the project root:

```bash
# OpenAI API Configuration (for LM Studio or OpenAI)
API_URL=http://127.0.0.1:1234/v1
API_MODEL=local-model
API_KEY=not-needed
```

**Important Notes:**
- `squid.config.json` takes precedence over `.env` variables. If both exist, the config file will be used.
- **Commit `squid.config.json`** to your repository to share project settings with your team
- **Keep `.env` private** - it should contain sensitive information like API keys and is excluded from git
- For cloud API services (OpenAI, etc.), store the actual API key in `.env` and omit `api_key` from `squid.config.json`

### Configuration Options

- `API_URL`: The base URL for the API endpoint
  - LM Studio: `http://127.0.0.1:1234/v1` (default)
  - Ollama: `http://localhost:11434/v1`
  - OpenAI: `https://api.openai.com/v1`
  - Other: Your provider's base URL
  
- `API_MODEL`: The model to use
  - LM Studio: `local-model` (uses whatever model is loaded)
  - Ollama: `qwen2.5-coder` (recommended) or any pulled model
  - OpenAI: `gpt-4`, `gpt-3.5-turbo`, etc.
  - Other: Check your provider's model names
  
- `API_KEY`: Your API key
  - LM Studio: `not-needed` (no authentication required)
  - Ollama: `not-needed` (no authentication required)
  - OpenAI: Your actual API key (e.g., `sk-...`)
  - Other: Your provider's API key

- `LOG_LEVEL`: Logging verbosity (optional, default: `error`)
  - `error`: Only errors (default)
  - `warn`: Warnings and errors
  - `info`: Informational messages
  - `debug`: Detailed debugging information
  - `trace`: Very verbose output

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

## Usage

> **Note:** The examples below use the `squid` command (after installation with `cargo install --path .`).  
> For development, replace `squid` with `cargo run --` (e.g., `cargo run -- ask "question"`).

### Ask a Question

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

### Ask About a File

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

### Review Code

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

The review command automatically selects the appropriate review prompt based on file type:
- **Rust** (`.rs`) - Ownership, safety, idioms, error handling
- **TypeScript/JavaScript** (`.ts`, `.js`, `.tsx`, `.jsx`) - Type safety, modern features, security
- **HTML** (`.html`, `.htm`) - Semantics, accessibility, SEO
- **CSS** (`.css`, `.scss`, `.sass`) - Performance, responsive design, maintainability
- **Python** (`.py`, `.pyw`, `.pyi`) - PEP 8, security, performance, best practices
- **SQL** (`.sql`, `.ddl`, `.dml`) - Performance, security, correctness, best practices
- **Shell Scripts** (`.sh`, `.bash`, `.zsh`, `.fish`) - Security, robustness, performance, compliance
- **Docker/Kubernetes** (`Dockerfile`, `Dockerfile.*`) - Security, performance, reliability, best practices
- **Go** (`.go`) - Concurrency, performance, error handling, best practices
- **Java** (`.java`) - Performance, best practices, JVM specifics, Spring framework
- **JSON** (`.json`) - Security, correctness, performance, maintainability
- **YAML** (`.yaml`, `.yml`) - Security, correctness, performance, maintainability
- **Makefile** (`Makefile`, `Makefile.*`) - Correctness, portability, performance, security
- **Markdown** (`.md`, `.markdown`) - Structure, accessibility, consistency, content
- **Other files** - Generic code quality and best practices

### Squid Web UI

Start the built-in web interface for Squid:

```bash
# Start Web UI on default port (8080)
squid serve

# Specify a custom port
squid serve --port 3000
squid serve -p 3000
```

The web server will:
- Launch the Squid Web UI at `http://127.0.0.1:8080` (or your specified port)
- Provide a browser-based interface for interacting with Squid
- Expose a REST API endpoint at `/api/chat` for streaming chat responses
- Display the server URL and API endpoint on startup

The web UI and API are served from the same server, so the chatbot automatically connects to the local API endpoint.

Press `Ctrl+C` to stop the server.

#### API Endpoint

The web server exposes a REST API for programmatic access:

**Endpoint:** `POST /api/chat`

**Request Body:**
```json
{
  "message": "Your question here",
  "file_content": "optional file content",
  "file_path": "optional/file/path.rs",
  "system_prompt": "optional custom system prompt"
}
```

**Response:** Server-Sent Events (SSE) stream with JSON events:
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
      if (event.type === 'content') {
        console.log(event.text);
      }
    }
  }
}
```

See `web/src/lib/chat-api.ts` for a complete TypeScript client implementation.

**Note:** The chatbot UI is served from the same server as the API, so it automatically uses the relative path `/api/chat` without requiring any configuration.

### View Application Logs

View logs stored in the database for debugging and troubleshooting:

```bash
# View recent logs (last 50 by default)
squid logs

# View more logs
squid logs --limit 100

# Filter by log level
squid logs --level error
squid logs --level warn
squid logs --level info

# View logs for a specific session
squid logs --session-id 72dd7601-7da4-4252-80f6-7012da923faf

# Combine filters
squid logs --limit 20 --level error
```

The logs are stored in the SQLite database (`squid.db`) alongside your chat sessions. This makes it easy to:
- Debug issues by reviewing what happened during a session
- Track errors and warnings across server restarts
- Correlate logs with specific chat conversations
- Monitor application behavior over time

**Note:** The `logs` command reads from the database. Logs are automatically stored when running the `serve` command.

### Tool Calling (with Multi-Layered Security)

The LLM has been trained to intelligently use tools when needed. It understands when to read, write, or search files based on your questions. 

**Security Layers:**
1. **Path Validation** - Automatically blocks system directories (`/etc`, `/root`, `~/.ssh`, etc.)
2. **Ignore Patterns** - `.squidignore` file blocks specified files/directories (like `.gitignore`)
3. **User Approval** - Manual confirmation required for each operation

For details, see [Security Features](docs/SECURITY.md).

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

**Available Tools:**
- 📖 **read_file** - Read file contents from the filesystem
- 📝 **write_file** - Write content to files
- 🔍 **grep** - Search for patterns in files using regex (supports directories and individual files)
- 🕐 **now** - Get current date and time in RFC 3339 format (UTC or local timezone)
- 💻 **bash** - Execute safe, non-destructive bash commands (ls, git status, cat, etc.)

**Key Features:**
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

**Using .squidignore:**

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

## Documentation

- **[Quick Start Guide](docs/QUICKSTART.md)** - Get started in 5 minutes
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
