# squid 🦑

A CLI application for interacting with LLM APIs (OpenAI-compatible) with support for streaming responses.

## Features

- 🤖 Chat with LLMs via OpenAI-compatible APIs
- 📄 Provide file context for AI analysis
- 🔍 AI-powered code reviews with language-specific prompts
- 🔧 Tool calling support (file read/write/search operations) with security approval
- 🔒 User approval required for all tool executions (read/write files)
- 🌊 Streaming support for real-time responses
- ⚙️ Configurable via environment variables
- 🔌 Works with LM Studio, OpenAI, and other compatible services

## Prerequisites

Before you begin, you'll need:

1. **Rust toolchain** (for building squid)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **An OpenAI-compatible LLM service** (choose one):

### Option A: LM Studio (Recommended for Local Development)

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

### Option B: Ollama (Lightweight CLI Option)

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

### Option C: OpenAI API

Use OpenAI's cloud API for access to GPT models:

1. **Get an API key** from https://platform.openai.com/api-keys
2. **Add credits** to your OpenAI account
3. **Choose a model**: `gpt-4`, `gpt-4-turbo`, `gpt-3.5-turbo`, etc.

### Option D: Other OpenAI-Compatible Services

Squid works with any OpenAI-compatible REST API:
- **OpenRouter** (https://openrouter.ai/) - Access to multiple LLM providers
- **Together AI** (https://together.ai/) - Fast inference
- **Anyscale** (https://anyscale.com/) - Enterprise solutions
- **Local APIs** - Any custom OpenAI-compatible endpoint

## Installation

### Install to Your System

```bash
cargo install --path .
```

This installs the `squid` command globally. You can then use `squid` from anywhere.

### Or Build for Development

```bash
cargo build --release
```

For development, use `cargo run --` instead of `squid` in the examples below.

## Configuration

Create a `.env` file in the project root:

```bash
# OpenAI API Configuration (for LM Studio or OpenAI)
API_URL=http://127.0.0.1:1234/v1
API_MODEL=local-model
API_KEY=not-needed
```

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

## Usage

> **Note:** The examples below use the `squid` command (after installation with `cargo install --path .`).  
> For development, replace `squid` with `cargo run --` (e.g., `cargo run -- ask "question"`).

### Ask a Question

```bash
# Basic question (required)
squid ask "What is Rust?"

# With additional context using -m
squid ask "Explain Rust" -m "Focus on memory safety"
```

This will send the question to the LLM and display the complete response once it's ready.

### Ask with Streaming

```bash
squid ask --stream "Explain async/await in Rust"
# or use short flag
squid ask -s "Explain async/await in Rust"
```

This will stream the response in real-time, displaying tokens as they are generated.

### Ask About a File

```bash
# Basic file question
squid ask -f sample-files/sample.txt "What are the key features mentioned?"

# With streaming
squid ask -f code.rs -s "Explain what this code does"

# With additional context using -m
squid ask -f src/main.rs "What does this do?" -m "Focus on error handling"
```

This will read the file content and include it in the prompt, allowing the AI to answer questions based on the file's content.

### Review Code

```bash
# Review a file with language-specific prompts
squid review src/main.rs

# Stream the review in real-time
squid review app.ts --stream

# Focus on specific aspects
squid review styles.css -m "Focus on performance issues"
```

The review command automatically selects the appropriate review prompt based on file type:
- **Rust** (`.rs`) - Ownership, safety, idioms, error handling
- **TypeScript/JavaScript** (`.ts`, `.js`, `.tsx`, `.jsx`) - Type safety, modern features, security
- **HTML** (`.html`, `.htm`) - Semantics, accessibility, SEO
- **CSS** (`.css`, `.scss`, `.sass`) - Performance, responsive design, maintainability
- **Other files** - Generic code quality and best practices



### Tool Calling (with Security Approval)

The LLM has been trained to intelligently use tools when needed. It understands when to read, write, or search files based on your questions. For security, you'll be prompted to approve each tool execution:

```bash
# LLM intelligently reads files when you ask about them
squid ask "Read the README.md file and summarize it"
squid ask "What dependencies are in Cargo.toml?"
squid ask "Analyze the main.rs file for me"
# You'll be prompted: "Allow reading file: [filename]? (Y/n)"

# LLM can write files
squid ask "Create a hello.txt file with 'Hello, World!'"
# You'll be prompted with a preview: "Allow writing to file: hello.txt?"

# LLM can search for patterns in files using grep
squid ask "Search for all TODO comments in the src directory"
squid ask "Find all function definitions in src/main.rs"
squid ask "Search for 'API_URL' in the project"
squid ask "Find all uses of 'unwrap' in the codebase"
squid ask "Show me all error handling patterns in src/tools.rs"
# You'll be prompted: "Allow searching for pattern '...' in: [path]? (Y/n)"
# Results show file path, line number, and matched content

# Works with streaming too
squid ask -s "Read Cargo.toml and list all dependencies"
squid ask -s "Search for async functions in src and explain them"
```

**Available Tools:**
- 📖 **read_file** - Read file contents from the filesystem
- 📝 **write_file** - Write content to files
- 🔍 **grep** - Search for patterns in files using regex (supports directories and individual files)

**Key Features:**
- 🤖 **Intelligent tool usage** - LLM understands when to read/write/search files from natural language
- 🔒 **Security approval** - All tool executions require user confirmation
- 📋 **Content preview** - File write operations show what will be written
- ⌨️ **Simple controls** - Press `Y` to allow or `N` to skip
- 📝 **Full logging** - All tool calls are logged for transparency
- 🔍 **Regex support** - Grep tool supports regex patterns with configurable case sensitivity

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

### Other Commands

```bash
# Initialize a project (placeholder)
squid init

# Run a command (placeholder)
squid run <command>
```

## Examples

### Using with LM Studio

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
   squid ask -s "Write a hello world program in Rust"
   # Or with a file
   squid ask -f sample-files/sample.txt "What is this document about?"
   ```

### Using with Ollama

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
   # Or with streaming
   squid ask -s -f mycode.rs "Explain this code"
   ```

### Using with OpenAI

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
   ```

## License

Apache-2.0 License. See `LICENSE` file for details.
