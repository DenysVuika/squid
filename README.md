# squid

A CLI application for interacting with LLM APIs (OpenAI-compatible) with support for streaming responses.

## Features

- 🤖 Chat with LLMs via OpenAI-compatible APIs
- 🔥 Run local models directly from HuggingFace (using mistral.rs)
- 🌊 Streaming support for real-time responses
- ⚙️ Configurable via environment variables
- 🔌 Works with LM Studio, OpenAI, and other compatible services

## When to Use Which Mode?

### `ask` (API Mode)
**Best for:**
- ✅ Production deployments with dedicated API servers
- ✅ Sharing models across multiple clients
- ✅ Using cloud-hosted models (OpenAI, Anthropic, etc.)
- ✅ When you already have LM Studio running

**Pros:** Fast, optimized servers, streaming support  
**Cons:** Requires external server or API key

### `ask-local` (Direct Inference)
**Best for:**
- ✅ Complete offline usage
- ✅ Testing different models quickly
- ✅ Privacy-sensitive applications
- ✅ No API costs or rate limits
- ✅ Self-contained deployments

**Pros:** No server needed, fully local, free  
**Cons:** Slower first run (downloads model), uses more RAM

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

### Ask a Question via API (Non-streaming)

```bash
cargo run -- ask "What is Rust?"
```

This will send the question to the LLM and display the complete response once it's ready.

### Ask a Question via API (Streaming)

```bash
cargo run -- ask --stream "Explain async/await in Rust"
# or use the short flag
cargo run -- ask -s "Explain async/await in Rust"
```

This will stream the response in real-time, displaying tokens as they are generated.

### Ask a Question using Local Model

Run models directly from HuggingFace without needing an API server:

```bash
# Use any supported HuggingFace model
cargo run -- ask-local --model "Qwen/Qwen2.5-0.5B-Instruct" "What is Rust?"

# Try other models
cargo run -- ask-local -m "microsoft/Phi-3-mini-4k-instruct" "Explain async/await"
cargo run -- ask-local -m "TinyLlama/TinyLlama-1.1B-Chat-v1.0" "Write a hello world in Rust"
```

**Tested Working Models:**

| Model | Size | Speed | Quality | Use Case |
|-------|------|-------|---------|----------|
| `Qwen/Qwen2.5-0.5B-Instruct` | ~500MB | ⚡⚡⚡ | ⭐⭐⭐ | Quick testing, development |
| `TinyLlama/TinyLlama-1.1B-Chat-v1.0` | ~1GB | ⚡⚡⚡ | ⭐⭐ | Fastest responses |
| `Qwen/Qwen2.5-1.5B-Instruct` | ~1.5GB | ⚡⚡ | ⭐⭐⭐⭐ | Balanced performance |
| `microsoft/Phi-3-mini-4k-instruct` | ~4GB | ⚡ | ⭐⭐⭐⭐⭐ | High quality output |

**Note:** 
- Models are automatically downloaded from HuggingFace on first use
- Models are cached locally for subsequent runs
- Uses 4-bit quantization (Q4K) for efficiency
- First run may take a while to download the model
- Not all HuggingFace models are supported - check [mistral.rs compatibility](https://ericlbuehler.github.io/mistral.rs/supported_models.html)

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
   ```

### Using Local Models (No API Required)

You can run models directly without any external API or server:

```bash
# Small, fast model (good for testing)
cargo run -- ask-local -m "Qwen/Qwen2.5-0.5B-Instruct" "Hello!"

# Balanced model (recommended)
cargo run -- ask-local -m "Qwen/Qwen2.5-1.5B-Instruct" "Explain quantum computing"

# High-quality model
cargo run -- ask-local -m "microsoft/Phi-3-mini-4k-instruct" "Write a sorting algorithm in Python"

# Tiny model (fastest)
cargo run -- ask-local -m "TinyLlama/TinyLlama-1.1B-Chat-v1.0" "What is 2+2?"
```

**Supported Model Formats:**
- Text models compatible with mistral.rs (see [supported models](https://ericlbuehler.github.io/mistral.rs/supported_models.html))
- Includes: Qwen, Phi, Llama, Mistral, Gemma, and many more
- Automatically applies 4-bit quantization for efficient inference
- If you get "Unsupported model class" error, the model architecture isn't supported yet

## Dependencies

- `async-openai`: OpenAI API client
- `mistralrs`: Local model inference engine
- `clap`: Command-line argument parsing
- `tokio`: Async runtime
- `futures`: Stream handling
- `dotenvy`: Environment variable management
- `log` & `env_logger`: Logging

## Command Reference

```bash
# API-based chat (requires LM Studio or OpenAI API)
cargo run -- ask "your question"                    # Non-streaming
cargo run -- ask --stream "your question"           # Streaming
cargo run -- ask -s "your question"                 # Streaming (short)

# Local model chat (no API required)
cargo run -- ask-local --model "model-id" "question"
cargo run -- ask-local -m "model-id" "question"     # Short form

# Other commands
cargo run -- init                                   # Initialize project
cargo run -- run <command>                          # Run a command
```

## License

Apache-2.0 License. See `LICENSE` file for details.
