# Quick Start Guide

Get up and running with squid in under 5 minutes.

## 1. Install

```bash
cargo install --path .
# Or for development: cargo build --release (use `cargo run --` instead of `squid`)
```

## 2. Configure

### Interactive (Recommended)

```bash
squid init
```

Prompts for API URL, model, API key, and log level. Creates `squid.config.json`.

### Manual (.env)

```bash
API_URL=http://127.0.0.1:1234/v1   # LM Studio
# API_URL=http://localhost:11434/v1  # Ollama
# API_URL=https://api.openai.com/v1  # OpenAI
API_KEY=not-needed
```

See [LLM Provider Reference](../README.md#llm-provider-reference) for all supported providers.

## 3. Try It

```bash
squid ask "What is Rust?"
squid ask -f sample-files/sample.txt "What is this about?"
squid review src/main.rs
squid ask -f src/main.rs "Explain what this code does"
```

Streaming is default. Use `--no-stream` for scripting: `squid ask --no-stream "question" > output.txt`

## 4. Common Patterns

```bash
# Focused question
squid ask "Explain async/await" -m "Focus on error handling"

# Code review with focus
squid review src/auth.rs -m "Focus on security issues"

# Custom system prompt
squid ask -f api.rs -p performance-reviewer.md "Review for performance issues"
```

## 5. Troubleshooting

| Issue | Fix |
|-------|-----|
| Connection error | Verify LLM provider is running and `API_URL` is correct |
| Response not relevant | Make your question specifically reference the file content |
| Failed to read file | Check path exists and you have read permissions |

**Debug mode:** `RUST_LOG=debug squid ask "test question"`

## 6. What's Next?

- [EXAMPLES.md](EXAMPLES.md) — Advanced usage patterns
- [CLI.md](CLI.md) — Complete command reference
- [README.md](../README.md) — Full documentation
