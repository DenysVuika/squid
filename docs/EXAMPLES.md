# Squid CLI - Usage Examples

## Basic Usage

```bash
squid ask "What is Rust?"                              # Streaming (default)
squid ask --no-stream "What is Rust?"                  # Complete response (for scripting)
squid ask --file sample-files/sample.txt "What is this about?"
squid ask -f README.md "Summarize this project"
squid ask -f src/main.rs "Explain what this code does"
```

## Advanced Patterns

```bash
# Custom system prompt
squid ask --prompt custom-prompt.md "Explain Rust ownership"
squid ask -f src/main.rs -p expert-reviewer.md "Review this code"

# Code review via ask
squid ask -f src/main.rs "Review and suggest improvements"

# Extract information
squid ask -f Cargo.toml "What dependencies does this project use?"
squid ask -f config.json "Are there any configuration issues?"

# Generate documentation
squid ask -f src/utils.rs "Generate documentation comments"

# Convert code
squid ask -f script.py "Convert this Python code to Rust"
```

## Practical Workflows

```bash
# Understand unfamiliar code
squid ask -f src/complex_module.rs "Break down this code into simple terms"

# Review documentation
squid ask -f README.md "Is this README clear? What's missing?"

# Explain configuration
squid ask -f .env "Explain what each option does"

# Track changes
squid ask -f CHANGELOG.md "What are the latest changes?"
```

## Tips

1. **Be specific** — "Explain how error handling works in `ask_llm_streaming`" beats "What does this function do?"
2. **Use custom prompts** for specialized tasks: `squid ask -f api.rs -p performance-reviewer.md "Review for performance issues"`
3. **Streaming** is default (real-time). Use `--no-stream` for piping/scripting: `squid ask --no-stream "question" > output.txt`
4. **Works with any text file** — source code, docs, configs, CSV, etc.

## Tool Calling

The LLM can read files, write files, search code, and run safe commands. All operations require your approval.

```bash
# LLM requests to read a file — you approve
squid ask "Read README.md and summarize it"

# LLM requests to write a file — you see a preview first
squid ask "Create hello.txt with 'Hello, World!'"

# Multiple tool calls — each requires approval
squid ask "Read Cargo.toml, extract dependencies, and create deps.txt"
```

For complete security documentation, see [SECURITY.md](SECURITY.md).

## Quick Test

```bash
squid ask --file sample-files/sample.txt "How many hearts does a squid have?"
# Answer: Three hearts
```
