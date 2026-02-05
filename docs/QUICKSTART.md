# Quick Start Guide

Welcome! This guide will get you up and running with squid in under 5 minutes.

## 1. Prerequisites

### Required

- **Rust toolchain** (for building squid)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Choose Your LLM Provider

You need **one** of the following:

#### Option A: LM Studio (Recommended for Beginners)

[LM Studio](https://lmstudio.ai/) - Easy-to-use GUI for running local LLMs.

1. Download from https://lmstudio.ai/
2. Install and launch LM Studio
3. Download recommended model: **Qwen2.5-Coder-7B-Instruct**
   - Search in LM Studio: `lmstudio-community/Qwen2.5-Coder-7B-Instruct-MLX-4bit`
   - Or visit: https://huggingface.co/lmstudio-community/Qwen2.5-Coder-7B-Instruct-MLX-4bit
4. Load the model (click on it in the "My Models" tab)
5. Start local server:
   - Click "Local Server" tab (↔️ icon)
   - Click "Start Server"
   - Default: `http://127.0.0.1:1234/v1`

#### Option B: Ollama (Lightweight CLI)

[Ollama](https://ollama.com/) - Command-line tool for running LLMs.

1. **Install**:
   ```bash
   # macOS
   brew install ollama
   
   # Linux
   curl -fsSL https://ollama.com/install.sh | sh
   
   # Windows/Other - download from https://ollama.com/
   ```

2. **Start the service**:
   ```bash
   ollama serve
   ```

3. **Pull recommended model** (in a new terminal):
   ```bash
   ollama pull qwen2.5-coder
   ```
   - Model info: https://ollama.com/library/qwen2.5-coder
   - Size options: 0.5B, 1.5B, 3B, 7B (default/recommended), 14B, 32B

4. **Verify**:
   ```bash
   ollama list  # Should show qwen2.5-coder
   ```

#### Option C: OpenAI API

Use OpenAI's cloud service:

1. Get API key from https://platform.openai.com/api-keys
2. Add credits to your account
3. Choose a model: `gpt-4`, `gpt-4-turbo`, or `gpt-3.5-turbo`

#### Option D: Mistral API

Use Mistral's cloud service:

1. Get API key from https://console.mistral.ai/
2. Choose a model: `devstral-2512`, `mistral-large-latest`, or `mistral-small-latest`
3. Mistral API is OpenAI-compatible, so it works seamlessly

#### Option E: Other OpenAI-Compatible APIs

Any OpenAI-compatible REST API works:
- OpenRouter (https://openrouter.ai/)
- Together AI (https://together.ai/)
- Anyscale (https://anyscale.com/)
- Custom endpoints

## 2. Install squid

```bash
cd squid
cargo install --path .
```

This installs the `squid` command to your system. Alternatively, you can build it with `cargo build --release` and use `cargo run --` for development.

## 3. Configure Your Environment

You can configure squid in two ways:

### Option A: Interactive Setup (Recommended)

Run the `init` command to create a `squid.config.json` file:

```bash
squid init
```

This will prompt you for:
- API URL (e.g., `http://127.0.0.1:1234/v1` for LM Studio)
- API Model (e.g., `local-model`, `qwen2.5-coder`, `gpt-4`)
- API Key (optional, leave empty for local models)
- Log Level (error, warn, info, debug, trace)

Example session:
```
$ squid init
INFO: Initializing squid configuration...
? API URL: http://127.0.0.1:1234/v1
? API Model: local-model
? API Key (optional, press Enter to skip): 
? Log Level: error

Configuration:
  API URL: http://127.0.0.1:1234/v1
  API Model: local-model
  API Key: [not set]
  Log Level: error
```

### Option B: Manual Configuration

Create a `.env` file in the project root:

### For LM Studio

```bash
API_URL=http://127.0.0.1:1234/v1
API_MODEL=local-model
API_KEY=not-needed
LOG_LEVEL=error
```

### For Ollama

```bash
API_URL=http://localhost:11434/v1
API_MODEL=qwen2.5-coder
API_KEY=not-needed
LOG_LEVEL=error
```

### For OpenAI

```env
API_URL=https://api.openai.com/v1
API_MODEL=gpt-4
API_KEY=sk-your-actual-api-key-here
LOG_LEVEL=error
```

### For Mistral API

```env
API_URL=https://api.mistral.ai/v1
API_MODEL=devstral-2512
API_KEY=your-mistral-api-key-here
LOG_LEVEL=error
```

### For Other Providers

Check your provider's documentation for the correct `API_URL`, `API_MODEL`, and `API_KEY` values.

**Important Notes:**
- If both `squid.config.json` and `.env` exist, the config file takes precedence
- **Commit `squid.config.json`** to your repository to share project settings with your team
- **Keep `.env` private** - it should contain sensitive information like API keys and is excluded from git
- For cloud API services (OpenAI, etc.), store the actual API key in `.env` and omit `api_key` from `squid.config.json`

## 4. Try Your First Command

### Ask a question (streaming by default):

```bash
squid ask "What is Rust?"
```

### Ask about a file:

```bash
squid ask -f sample-files/sample.txt "What is this document about?"
```

### Review code:

```bash
squid review src/main.rs
```

### Disable streaming (for scripting/piping):

```bash
squid ask --no-stream "Generate a hello world function"
```

> **Development:** If you didn't install squid, use `cargo run --` instead of `squid`:
> ```bash
> cargo run -- ask "What is Rust?"
> ```

That's it! 🎉

> **Note:** All commands stream responses by default. Use `--no-stream` when you need the complete response at once (e.g., for piping to other commands or scripting).

## 5. Common Use Cases

### Ask Questions

```bash
# Basic question (streams by default)
squid ask "What is Rust?"

# With additional context
squid ask "Explain Rust" -m "Focus on memory safety"

# Disable streaming for complete response
squid ask --no-stream "What is Rust?"
```

### Analyze Files

```bash
# Streams by default
squid ask -f src/main.rs "What does this code do?"
squid ask -f README.md "Summarize this project"

# Complete response at once
squid ask -f config.json --no-stream "Extract the API settings"
```

### Review Code

```bash
# Basic review (streams by default)
squid review src/main.rs

# Focused review
squid review src/auth.rs -m "Focus on security issues"

# Complete review at once
squid review components/App.tsx --no-stream
```

### Disable Streaming for Scripting

```bash
# Get complete response for piping
result=$(squid ask --no-stream "Generate a variable name for user data")
echo $result | tr '[:upper:]' '[:lower:]'

# Use in scripts
squid review sample-files/example.rs --no-stream > review.txt
```

## 6. Command Syntax

### Ask Command

```
squid ask [OPTIONS] <QUESTION>

Arguments:
  <QUESTION>  The question to ask (required)

Options:
  -m, --message <MESSAGE>  Optional additional context or instructions
  --no-stream              Disable streaming (return complete response at once)
  -f, --file <FILE>        Provide a file for context
  -h, --help               Print help
```

**Note:** Responses are streamed by default. Use `--no-stream` for complete response.

### Review Command

```
squid review [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to the file to review (required)

Options:
  -m, --message <MESSAGE>  Optional additional message or specific question
  --no-stream              Disable streaming (return complete response at once)
  -h, --help               Print help
```

**Note:** Reviews are streamed by default. Use `--no-stream` for complete response.

## 7. Tips for Better Results

### ✅ Use the Right Command

```bash
# For questions and analysis (streams by default)
squid ask "What is async/await in Rust?"
squid ask -f code.rs "Explain this code"

# For code reviews (streams by default)
squid review src/main.rs
squid review app.ts -m "Check for security issues"

# Use --no-stream for scripting
squid ask --no-stream "List three Rust features" | head -n 3
```

### ✅ Be Specific with Context

```bash
# Good
squid ask -f code.rs "Explain the main function"

# Better - use -m for focus
squid ask -f code.rs "Explain the main function" -m "Focus on error handling"
squid review auth.rs -m "Focus on security vulnerabilities"
```

### ✅ Language-Specific Reviews

The `review` command automatically uses specialized prompts for:
- Rust (`.rs`) - Ownership, safety, idioms
- TypeScript/JavaScript (`.ts`, `.js`, `.tsx`, `.jsx`) - Type safety, modern features
- HTML (`.html`) - Semantics, accessibility
- CSS (`.css`, `.scss`) - Performance, responsive design

### ✅ Streaming is Default (Disable When Needed)

```bash
# Streaming is automatic - great for watching progress
squid ask -f big_document.md "Analyze this thoroughly"
squid review large_component.tsx

# Disable for scripting or piping
squid ask --no-stream "Brief answer" | grep "keyword"
squid review code.rs --no-stream > review-results.txt
```

### ✅ Tool Calling with Security

The LLM can read and write files when needed - you'll approve each action:

```bash
# LLM can read files (with your approval)
squid ask "Read the README.md and summarize it"
# You'll see: "Allow reading file: README.md? (Y/n)"

# LLM can write files (with preview and approval)
squid ask "Create a hello.txt file with 'Hello, World!'"
# You'll see the content preview and: "Allow writing to file: hello.txt?"
```

**Security features:**
- ✅ Every tool execution requires your approval
- ✅ File writes show content preview (first 100 bytes)
- ✅ Press `Y` to allow or `N` to skip
- ✅ All tool calls are logged

## 8. Troubleshooting

### "Failed to read file"
- Check the file path is correct
- Make sure you have read permissions
- Try using absolute path if relative doesn't work

### No response / Connection error
- **LM Studio**: 
  - Make sure LM Studio is running
  - Verify a model is loaded
  - Check that local server is started (↔️ tab → "Start Server")
  - Default URL: `http://127.0.0.1:1234/v1`
- **Ollama**: 
  - Make sure `ollama serve` is running
  - Verify model is pulled: `ollama list`
  - Default URL: `http://localhost:11434/v1`
- **OpenAI**: 
  - Check your API key is valid and has credits
  - Verify URL: `https://api.openai.com/v1`
- **Mistral API**:
  - Check your API key is valid
  - Verify URL: `https://api.mistral.ai/v1`
- **All providers**: Verify `API_URL`, `API_MODEL`, and `API_KEY` in your `.env` file

### Response not relevant to file
- Make sure your question specifically asks about the file content
- Check the file was actually read (look for log message)
- Enable debug logging: `RUST_LOG=debug cargo run -- ask -f ...`

## 9. What's Next?

- Read `EXAMPLES.md` for more advanced usage patterns and workflows
- Check the code review section in `README.md` for review command details
- Try example files in `sample-files/` directory
- See `README.md` for full documentation
- Run `squid --help` to see all available commands and options

## 10. Quick Reference

```bash
# Ask questions (streaming by default)
squid ask "question here"
squid ask "question" -m "additional context"
squid ask -f filename.txt "question here"
squid ask --no-stream "question here"  # Complete response at once

# Review code (streaming by default)
squid review src/main.rs
squid review app.ts -m "Focus on security"
squid review styles.css --no-stream  # Complete review at once

# Tool calling (with approval prompts)
squid ask "Read README.md and summarize it"
squid ask "Create a notes.txt file with today's tasks"

# Scripting/piping
result=$(squid ask --no-stream "Generate a name")
squid review code.rs --no-stream > output.txt

# Get help
squid ask --help
squid review --help
```

## Need More Help?

1. Enable debug logging:
   ```bash
   RUST_LOG=debug squid ask "test question"
   ```

2. Check your configuration:
   ```bash
   cat .env
   ```

3. Verify your LLM provider is running:
   - **LM Studio**: Model loaded, local server started
   - **Ollama**: `ollama serve` running, model pulled
   - **OpenAI**: API key valid, account has credits
   - **Mistral API**: API key valid

Happy coding! 🦑