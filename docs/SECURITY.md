# Security Features

This document describes the security features implemented in `squid` to protect users during LLM interactions with tool calling capabilities.

> **Quick Start:** See the "Tool Calling (with Security Approval)" section in the main [README.md](../README.md) for examples.

## Overview

When the LLM requests to use tools (such as reading, writing, or searching files), `squid` requires explicit user approval before executing any operation. This prevents unauthorized or unintended file system access.

## Tool Approval Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. User sends a query to the LLM                            │
│    Example: "Read README.md and summarize it"               │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. LLM determines it needs to use a tool                    │
│    Tool: read_file                                          │
│    Arguments: {"path": "README.md"}                         │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. squid intercepts tool call and prompts user              │
│                                                             │
│    ┌───────────────────────────────────────────────────┐    │
│    │ Allow reading file: README.md? (Y/n)              │    │
│    │ [Y] Allow  [N] Skip                               │    │
│    └───────────────────────────────────────────────────┘    │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. User makes decision                                      │
│    • Press Y → Tool executes, result sent to LLM            │
│    • Press N → Tool skipped, error sent to LLM              │
└─────────────────────────────────────────────────────────────┘
```

## Security Features

### 🔒 User Approval Required

All tool executions require explicit user approval. The LLM cannot read, write, or search files without user consent.

```bash
cargo run -- ask "Read my secret.txt file"
# Prompt: Allow reading file: secret.txt? (Y/n)
# → You control whether this happens

cargo run -- ask "Search for passwords in the project"
# Prompt: Allow searching for pattern 'passwords' in: .? (Y/n)
# → You control whether this happens
```

### 📋 Content Preview for Write Operations

When the LLM attempts to write a file, you see a preview of the content before approving:

```bash
cargo run -- ask "Create a config.json file with default settings"

# You'll see:
# Allow writing to file: config.json?
# Content preview:
# {
#   "version": "1.0",
#   "debug": false,
#   "port": 8080
# }
# (Y/n)
```

For large content (>100 bytes), only the first 100 bytes are shown with a total size indicator.

### 🔍 Search Pattern Transparency

When the LLM attempts to search files using grep, you see the exact pattern and path:

```bash
cargo run -- ask "Find all TODO comments in src"

# You'll see:
# Allow searching for pattern 'TODO' in: src? (Y/n)
```

**Search features:**
- Regex pattern matching with case sensitivity options
- Recursive directory search or single file search
- Results show file path, line number, and matched content
- Automatic binary file filtering for safety
- Configurable result limits (default: 50 matches)

### 📝 Comprehensive Logging

All tool calls are logged for transparency and audit purposes:

```
[INFO] Tool call: read_file with args: {"path":"README.md"}
[INFO] Successfully read file: README.md (2847 bytes)
```

Or if denied:

```
[INFO] Tool call: write_file with args: {"path":"secret.txt","content":"..."}
[INFO] Tool execution skipped by user: write_file
```

### 🚫 Denial of Service Protection

Users can deny any tool execution at any time:

- Press `N` to skip the operation
- The LLM receives an error message
- The LLM can adapt its response based on the denial
- No files are accessed or modified

## Supported Tools

### read_file

**Purpose:** Read contents from the file system

**Security measures:**
- Shows exact file path before approval
- No preview of file contents (you approve based on the path)
- Logged with file size after successful read

**Example prompt:**
```
Allow reading file: /path/to/file.txt? (Y/n)
```

### write_file

**Purpose:** Write content to the file system

**Security measures:**
- Shows exact file path before approval
- Displays content preview (first 100 bytes)
- Shows total byte count for large files
- Logged with file size after successful write

**Example prompt:**
```
Allow writing to file: output.txt?
Content preview:
Hello, World!
This is a test file.
(Y/n)
```

## Best Practices

### ✅ DO

- **Review file paths carefully** before approving read operations
- **Check content previews** for write operations
- **Deny suspicious requests** - You can always run the command again
- **Use absolute paths** when possible to avoid ambiguity
- **Enable debug logging** if you want more detailed information:
  ```bash
  RUST_LOG=debug cargo run -- ask "your question"
  ```

### ❌ DON'T

- **Blindly approve** all tool executions
- **Allow reads of sensitive files** (`.env`, private keys, passwords)
- **Approve writes without checking** the content preview
- **Ignore unexpected tool requests** - If you didn't expect file access, press N

## Security Scenarios

### Scenario 1: Preventing Sensitive File Access

**Command:**
```bash
cargo run -- ask "What environment variables do I have?"
```

**LLM may try:**
```
Tool: read_file
Path: .env
```

**Your response:**
- Press `N` to deny
- LLM receives error, cannot access `.env`
- Your sensitive data stays protected

### Scenario 2: Validating Write Operations

**Command:**
```bash
cargo run -- ask "Create a startup script for this project"
```

**LLM may try:**
```
Tool: write_file
Path: startup.sh
Content: #!/bin/bash
rm -rf / # Malicious content
```

**Your response:**
- Review the content preview
- See the malicious `rm -rf /` command
- Press `N` to deny
- File is not created

### Scenario 3: Safe Batch Operations

**Command:**
```bash
cargo run -- ask "Read all .rs files in src/ and create a summary"
```

**LLM behavior:**
- Requests to read each file individually
- You approve each one: src/main.rs ✓, src/lib.rs ✓, etc.
- Finally requests to write summary.txt
- You review the summary content
- Approve if it looks correct

## Logging and Audit

All tool operations are logged at the INFO level. To view these logs:

```bash
# Default (shows INFO and above)
RUST_LOG=info cargo run -- ask "your question"

# Debug (shows all logs including tool details)
RUST_LOG=debug cargo run -- ask "your question"

# Only errors
RUST_LOG=error cargo run -- ask "your question"
```

**Log examples:**

```
[INFO] Tool call: read_file with args: {"path":"README.md"}
[INFO] Successfully read file: README.md (2847 bytes)
[INFO] Tool call: write_file with args: {"path":"summary.txt","content":"..."}
[INFO] Successfully wrote file: summary.txt (156 bytes)
```

Or when denied:

```
[INFO] Tool call: read_file with args: {"path":".env"}
[INFO] Tool execution skipped by user: read_file
```

## Technical Details

### Implementation

The security approval is implemented using the `inquire` library, which provides:
- Cross-platform terminal prompts
- Default to `false` (deny by default)
- Clear yes/no options
- Keyboard-only operation (Y/N keys)

### Non-Interactive Mode

If `squid` is run in a non-interactive environment (e.g., piped input, CI/CD), tool approval will fail with an error:

```
[ERROR] Failed to get user approval: not a terminal
```

This is intentional - tools require human approval and cannot run unattended.

## Future Enhancements

Potential future security features (not yet implemented):

- [ ] Whitelist/blacklist for file paths
- [ ] Automatic approval for specific safe operations
- [ ] Audit log file for all tool executions
- [ ] Configuration option to disable tools entirely
- [ ] Sandboxing or restricted file system access
- [ ] Rate limiting for tool calls

## Reporting Security Issues

If you discover a security vulnerability in `squid`, please report it responsibly:

1. **DO NOT** create a public GitHub issue
2. Contact the maintainer directly (see `Cargo.toml` for author email)
3. Provide detailed information about the vulnerability
4. Allow time for a fix before public disclosure

## Summary

The tool approval system in `squid` provides:

✅ **User Control** - You approve every file operation  
✅ **Transparency** - See what's being accessed and written  
✅ **Logging** - Complete audit trail of all operations  
✅ **Prevention** - Stop unintended or malicious actions  

Your security is our priority. Use `squid` with confidence knowing you're in control.
