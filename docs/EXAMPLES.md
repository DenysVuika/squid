# Squid CLI - Usage Examples

This file contains practical examples of using the `squid` CLI with file context.

## Basic Usage

### Simple Question (No File)

```bash
squid ask "What is Rust?"
```

### Simple Question with Streaming

```bash
squid ask --stream "Explain async/await in Rust"
# or shorter:
squid ask -s "Explain async/await in Rust"
```

## File Context Examples

### Analyze a Text Document

```bash
squid ask --file docs/sample.txt "What is this document about?"
```

### Summarize a File

```bash
squid ask --file README.md "Provide a brief summary of this project"
```

### Code Analysis

```bash
squid ask --file src/main.rs "Explain what this code does"
```

### Code Review

```bash
squid ask --file src/main.rs "Review this code and suggest improvements"
```

### Documentation Questions

```bash
squid ask --file EXAMPLES.md "What examples are provided in this file?"
```

### Streaming with File Context

```bash
squid ask --file docs/sample.txt --stream "List the key features mentioned"
# or shorter:
squid ask -f docs/sample.txt -s "List the key features mentioned"
```

## Advanced Examples

### Find Specific Information

```bash
squid ask --file Cargo.toml "What dependencies does this project use?"
```

### Compare or Analyze Data

```bash
squid ask --file data.csv "What trends do you see in this data?"
```

### Explain Complex Code

```bash
squid ask --file algorithm.rs "Explain this algorithm step by step"
```

### Generate Documentation

```bash
squid ask --file src/utils.rs "Generate documentation comments for these functions"
```

### Translate or Convert

```bash
squid ask --file script.py "Convert this Python code to Rust"
```

### Find Issues

```bash
squid ask --file config.json "Are there any configuration issues in this file?"
```

## Practical Workflows

### 1. Code Understanding

When you encounter unfamiliar code:

```bash
squid ask -f src/complex_module.rs "Break down this code into simple terms"
```

### 2. Documentation Review

```bash
squid ask -f README.md -s "Is this README clear? What's missing?"
```

### 3. Configuration Help

```bash
squid ask -f .env "Explain what each configuration option does"
```

### 4. Learning from Examples

```bash
squid ask -f sample-files/tutorial.rs "What concepts does this example teach?"
```

### 5. Quick File Summary

```bash
squid ask -f CHANGELOG.md "What are the latest changes?"
```

## Tips

1. **Be Specific**: The more specific your question, the better the answer
   ```bash
   # Good
   squid ask -f main.rs "What does the ask_llm_streaming function do?"
   
   # Better
   squid ask -f main.rs "Explain how error handling works in the ask_llm_streaming function"
   ```

2. **Use Streaming for Long Responses**: If you expect a detailed answer, use `-s` flag
   ```bash
   squid ask -f large_file.txt -s "Provide a detailed analysis"
   ```

3. **Combine Context with Questions**: Frame your questions in context
   ```bash
   squid ask -f auth.rs "How does this authentication system prevent CSRF attacks?"
   ```

4. **File Types Supported**: Works with any text-based file
   - Source code (.rs, .py, .js, .go, etc.)
   - Documentation (.md, .txt, .rst)
   - Configuration (.toml, .json, .yaml, .env)
   - Data files (.csv, .tsv)
   - And more!

## Environment Setup

Make sure your `.env` file is configured:

```env
# For LM Studio (local)
API_URL=http://127.0.0.1:1234/v1
API_MODEL=local-model
API_KEY=not-needed

# For OpenAI
API_URL=https://api.openai.com/v1
API_MODEL=gpt-4
API_KEY=sk-your-key-here
```

## Tool Calling with Security Approval

The LLM can use tools to read and write files, but all operations require your approval for security:

### Reading Files

```bash
squid ask "Read the README.md file and summarize it"
```

**What happens:**
1. LLM requests to read README.md
2. You see: `Allow reading file: README.md? (Y/n)`
3. Press `Y` to approve or `N` to skip
4. If approved, the file is read and LLM continues

### Writing Files

```bash
squid ask "Create a hello.txt file with 'Hello, World!'"
```

**What happens:**
1. LLM requests to write to hello.txt
2. You see a preview:
   ```
   Allow writing to file: hello.txt?
   Content preview:
   Hello, World!
   ```
3. Press `Y` to approve or `N` to skip
4. If approved, the file is written

### Multiple Tool Calls

```bash
squid ask "Read Cargo.toml, extract all dependencies, and create a deps.txt file with the list"
```

**What happens:**
1. First approval: `Allow reading file: Cargo.toml? (Y/n)` → Press `Y`
2. LLM processes the content
3. Second approval: `Allow writing to file: deps.txt?` (with content preview) → Press `Y`
4. Both operations complete successfully

### Streaming with Tools

```bash
squid ask -s "Read the CHANGELOG.md and tell me what's new in the latest version"
```

Tool approvals work the same way in streaming mode - you'll be prompted before each tool execution.

### Security Tips

- **Always review file paths** before approving read/write operations
- **Check content previews** for write operations (shows first 100 bytes)
- **Press N to skip** any suspicious or unintended operations
- **All tool calls are logged** - check logs if you need to audit what happened

## Sample Test

Try this with the included sample file:

```bash
squid ask --file docs/sample.txt "How many hearts does a squid have?"
```

Expected answer: Three hearts (as mentioned in the docs/sample.txt fun facts section)