# 🔒 Security Approval Feature

## Overview

The `squid` CLI now includes **mandatory user approval** for all tool executions. This prevents the LLM from reading, writing, or searching files without your explicit consent.

## How It Works

When the LLM requests to use a tool, you'll see an interactive prompt:

### Reading Files

```bash
$ cargo run -- ask "Read the README.md and summarize it"

# The LLM requests to read a file:
Allow reading file: README.md? (Y/n)
> _
```

**Press:**
- `Y` or `Enter` → Allow the operation
- `N` → Skip/deny the operation

### Writing Files

```bash
$ cargo run -- ask "Create a hello.txt file with 'Hello, World!'"

# The LLM requests to write a file:
Allow writing to file: hello.txt?
Content preview:
Hello, World!

(Y/n)
> _
```

**You'll see:**
- The exact file path
- A preview of the content (first 100 bytes)
- Total byte count for larger files

### Searching Files (Grep)

```bash
$ cargo run -- ask "Search for TODO comments in the src directory"

# The LLM requests to search files:
Allow searching for pattern 'TODO' in: src? (Y/n)
> _
```

**Press:**
- `Y` or `Enter` → Allow the search operation
- `N` → Skip/deny the operation

**You'll see:**
- The regex pattern being searched
- The file or directory path to search
- Results include file path, line number, and matched content

## Available Tools

The LLM has access to these tools (all require approval):

1. **read_file** - Read file contents from the filesystem
2. **write_file** - Write content to files  
3. **grep** - Search for patterns in files using regex

## Key Features

✅ **Default Deny** - All prompts default to "No" for safety  
✅ **Content Preview** - See what will be written before approving  
✅ **Full Transparency** - Exact paths, patterns, and content shown  
✅ **Complete Logging** - All tool calls logged for audit  
✅ **Interactive Only** - Won't run unattended (security by design)  
✅ **Regex Support** - Grep tool supports pattern matching with case sensitivity options

## Examples

### Example 1: Safe Read Operation

```bash
$ cargo run -- ask "What dependencies are in Cargo.toml?"

Allow reading file: Cargo.toml? (Y/n)
> Y ✓

[INFO] Successfully read file: Cargo.toml (432 bytes)
# LLM receives the content and responds...
```

### Example 2: Denying Sensitive Access

```bash
$ cargo run -- ask "What's in my .env file?"

Allow reading file: .env? (Y/n)
> N ✗

[INFO] Tool execution skipped by user: read_file
# LLM receives error, your .env stays private
```

### Example 3: Validating Write Content

```bash
$ cargo run -- ask "Create a notes.txt with my TODO list"

Allow writing to file: notes.txt?
Content preview:
TODO List:
1. Review security features
2. Test tool approval
3. Update documentation

(Y/n)
> Y ✓

[INFO] Successfully wrote file: notes.txt (87 bytes)
```

### Example 4: Multiple Approvals

```bash
$ cargo run -- ask "Read package.json and create a summary.txt"

# First approval
Allow reading file: package.json? (Y/n)
> Y ✓

# Second approval
Allow writing to file: summary.txt?
Content preview:
Project: my-app
Version: 1.0.0
Dependencies: 15

(Y/n)
> Y ✓
```

## Testing the Feature

Run the provided test script:

```bash
./tests/test-security.sh
```

This interactive script demonstrates:
1. File read approval
2. File write approval with preview
3. Multiple tool call approvals
4. Denying operations

## Security Best Practices

### ✅ DO

- Review file paths carefully before approving
- Check content previews for write operations
- Deny unexpected or suspicious requests
- Use absolute paths when possible
- Enable logging: `RUST_LOG=info cargo run -- ask "..."`

### ❌ DON'T

- Blindly approve all prompts
- Allow reads of sensitive files (`.env`, keys, passwords)
- Approve writes without checking content
- Run in non-interactive mode expecting tools to work

## Technical Details

- **Library:** Uses `inquire` crate for cross-platform prompts
- **Default:** All prompts default to `false` (deny)
- **Non-Interactive:** Fails with error if not a terminal
- **Logging:** All operations logged at INFO level

## Logs

View approval decisions in the logs:

```bash
RUST_LOG=info cargo run -- ask "your question"

# Example logs:
[INFO] Tool call: read_file with args: {"path":"README.md"}
[INFO] Successfully read file: README.md (2847 bytes)
[INFO] Tool call: write_file with args: {"path":"output.txt",...}
[INFO] Tool execution skipped by user: write_file
```

## Documentation

For complete security documentation, see:
- **[SECURITY.md](SECURITY.md)** - Comprehensive security guide
- **[EXAMPLES.md](EXAMPLES.md)** - Usage examples
- **[../README.md](../README.md)** - Main documentation

## Summary

Every file operation requires your approval. You're in control. Use `squid` with confidence! 🔒