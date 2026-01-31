# squid

A CLI application for interacting with LLM APIs (OpenAI-compatible) with support for streaming responses.

## Features

- 🤖 Chat with LLMs via OpenAI-compatible APIs
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

### Ask a Question (Non-streaming)

```bash
cargo run -- ask "What is Rust?"
```

This will send the question to the LLM and display the complete response once it's ready.

### Ask a Question (Streaming)

```bash
cargo run -- ask --stream "Explain async/await in Rust"
# or use the short flag
cargo run -- ask -s "Explain async/await in Rust"
```

This will stream the response in real-time, displaying tokens as they are generated.

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

## License

Apache-2.0 License. See `LICENSE` file for details.
