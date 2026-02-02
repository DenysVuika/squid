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

## Installation

```bash
cargo build --release
```

## Configuration

Create a `.env` file in the project root (or copy from `.env.example`):

```bash
# OpenAI API Configuration (for LM Studio or OpenAI)
API_URL=http://127.0.0.1:1234/v1
API_MODEL=local-model
API_KEY=not-needed

# Optional: Database configuration
DATABASE_URL=postgresql://localhost/mydb
```

### Configuration Options

- `API_URL`: The base URL for the API endpoint
  - Default: `http://127.0.0.1:1234/v1` (LM Studio)
  - For OpenAI: `https://api.openai.com/v1`
  
- `API_MODEL`: The model to use
  - Default: `local-model` (LM Studio uses whatever model is loaded)
  - For OpenAI: `gpt-4`, `gpt-3.5-turbo`, etc.
  
- `API_KEY`: Your API key
  - Default: `not-needed` (LM Studio doesn't require authentication)
  - For OpenAI: Your actual API key (e.g., `sk-...`)

## Usage

### Ask a Question

```bash
# Basic question (required)
cargo run -- ask "What is Rust?"

# With additional context using -m
cargo run -- ask "Explain Rust" -m "Focus on memory safety"
```

This will send the question to the LLM and display the complete response once it's ready.

### Ask with Streaming

```bash
cargo run -- ask --stream "Explain async/await in Rust"
# or use short flag
cargo run -- ask -s "Explain async/await in Rust"
```

This will stream the response in real-time, displaying tokens as they are generated.

### Ask About a File

```bash
# Basic file question
cargo run -- ask -f sample-files/sample.txt "What are the key features mentioned?"

# With streaming
cargo run -- ask -f code.rs -s "Explain what this code does"

# With additional context using -m
cargo run -- ask -f src/main.rs "What does this do?" -m "Focus on error handling"
```

This will read the file content and include it in the prompt, allowing the AI to answer questions based on the file's content.

### Review Code

```bash
# Review a file with language-specific prompts
cargo run -- review src/main.rs

# Stream the review in real-time
cargo run -- review app.ts --stream

# Focus on specific aspects
cargo run -- review styles.css -m "Focus on performance issues"
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
cargo run -- ask "Read the README.md file and summarize it"
cargo run -- ask "What dependencies are in Cargo.toml?"
cargo run -- ask "Analyze the main.rs file for me"
# You'll be prompted: "Allow reading file: [filename]? (Y/n)"

# LLM can write files
cargo run -- ask "Create a hello.txt file with 'Hello, World!'"
# You'll be prompted with a preview: "Allow writing to file: hello.txt?"

# LLM can search for patterns in files using grep
cargo run -- ask "Search for all TODO comments in the src directory"
cargo run -- ask "Find all function definitions in src/main.rs"
cargo run -- ask "Search for 'API_URL' in the project"
cargo run -- ask "Find all uses of 'unwrap' in the codebase"
cargo run -- ask "Show me all error handling patterns in src/tools.rs"
# You'll be prompted: "Allow searching for pattern '...' in: [path]? (Y/n)"
# Results show file path, line number, and matched content

# Works with streaming too
cargo run -- ask -s "Read Cargo.toml and list all dependencies"
cargo run -- ask -s "Search for async functions in src and explain them"
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
cargo run -- review sample-files/example.rs
cargo run -- review sample-files/example.ts --stream
cargo run -- review sample-files/example.html -m "Focus on accessibility"
```

See **[tests/README.md](tests/README.md)** for complete testing documentation and **[sample-files/README.md](sample-files/README.md)** for details on each example file.

### Other Commands

```bash
# Initialize a project (placeholder)
cargo run -- init

# Run a command (placeholder)
cargo run -- run <command>
```

## Examples

### Using with LM Studio

1. Start LM Studio and load a model
2. Enable the local server (default: `http://127.0.0.1:1234`)
3. Set up your `.env`:
   ```bash
   API_URL=http://127.0.0.1:1234/v1
   API_MODEL=local-model
   API_KEY=not-needed
   ```
4. Run:
  ```bash
   cargo run -- ask -s "Write a hello world program in Rust"
   # Or with a file
   cargo run -- ask -f sample-files/sample.txt "What is this document about?"
   ```

### Using with OpenAI

1. Get your API key from OpenAI
2. Set up your `.env`:
   ```bash
   API_URL=https://api.openai.com/v1
   API_MODEL=gpt-4
   API_KEY=sk-your-api-key-here
   ```
3. Run:
   ```bash
   cargo run -- ask "Explain the benefits of Rust"
   # Or analyze a file
   cargo run -- ask -f mycode.rs "Review this code for potential improvements"
   ```

## License

Apache-2.0 License. See `LICENSE` file for details.
