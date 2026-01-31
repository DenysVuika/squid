# Quick Start Guide

Get up and running with squid in 2 minutes!

## Installation

```bash
git clone <repository-url>
cd squid
cargo build --release
```

## Quick Test

### Option 1: Use Local Model (No Setup Required!)

The fastest way to get started - no API keys, no servers:

```bash
# Ask a simple question
cargo run -- ask-local -m "Qwen/Qwen2.5-0.5B-Instruct" "What is 2+2?"
```

**Note:** First run downloads the model (~500MB). Subsequent runs are instant!

### Option 2: Use LM Studio API

If you have LM Studio running:

1. Start LM Studio and load a model
2. Enable the local server (usually `http://localhost:1234`)
3. Run:

```bash
cargo run -- ask "What is Rust?"
cargo run -- ask --stream "Explain async/await"  # with streaming
```

### Option 3: Use OpenAI

1. Create `.env` file:
```bash
API_URL=https://api.openai.com/v1
API_MODEL=gpt-4
API_KEY=sk-your-api-key-here
```

2. Run:
```bash
cargo run -- ask "Hello, GPT!"
```

## Command Reference

### API-Based Chat
```bash
# Non-streaming
cargo run -- ask "your question"

# Streaming (real-time output)
cargo run -- ask -s "your question"
```

### Local Model Chat
```bash
# Basic usage
cargo run -- ask-local -m "model-id" "your question"

# Example models
cargo run -- ask-local -m "Qwen/Qwen2.5-0.5B-Instruct" "question"           # Fast, small
cargo run -- ask-local -m "Qwen/Qwen2.5-1.5B-Instruct" "question"           # Balanced
cargo run -- ask-local -m "microsoft/Phi-3-mini-4k-instruct" "question"     # High quality
cargo run -- ask-local -m "TinyLlama/TinyLlama-1.1B-Chat-v1.0" "question"   # Tiny, fastest
```

## Recommended Models

### For Speed (< 1GB)
- `Qwen/Qwen2.5-0.5B-Instruct` - Best balance of speed/quality
- `TinyLlama/TinyLlama-1.1B-Chat-v1.0` - Fastest

### For Quality (1-4GB)
- `Qwen/Qwen2.5-1.5B-Instruct` - Good all-rounder
- `microsoft/Phi-3-mini-4k-instruct` - High quality responses

### For Production
- Use API mode with LM Studio or OpenAI
- Set up proper `.env` configuration

## Troubleshooting

### "Unsupported model class" Error
The model architecture isn't supported by mistral.rs yet. Try a different model from the list above.

### Model Download Too Slow
Models are downloaded from HuggingFace on first use. This is normal. Subsequent runs will be instant as models are cached.

### Out of Memory
Try a smaller model like `Qwen/Qwen2.5-0.5B-Instruct` or `TinyLlama/TinyLlama-1.1B-Chat-v1.0`.

## Next Steps

- See [README.md](README.md) for full documentation
- Check `.env.example` for configuration options
- Browse [mistral.rs supported models](https://ericlbuehler.github.io/mistral.rs/supported_models.html)

## Common Workflows

### Quick Testing
```bash
# Test with tiny model (fastest)
cargo run -- ask-local -m "TinyLlama/TinyLlama-1.1B-Chat-v1.0" "test"
```

### Development
```bash
# Use LM Studio for interactive development
cargo run -- ask -s "explain this code: ..."
```

### Production
```bash
# Use OpenAI API with proper error handling
API_KEY=sk-xxx cargo run -- ask "production query"
```

---

**Need Help?** Check the full [README.md](README.md) or open an issue!