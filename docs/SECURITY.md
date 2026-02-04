# Security Features

This document describes the security features implemented in `squid` to protect users during LLM interactions with tool calling capabilities, including path validation and ignore patterns.

> **Quick Start:** See the "Tool Calling (with Security Approval)" section in the main [README.md](../README.md) for examples.

## Overview

When the LLM requests to use tools (such as reading, writing, or searching files), `squid` provides multiple layers of security:

1. **Path Validation** - Whitelist/blacklist rules prevent access to sensitive system directories
2. **Ignore Patterns** - `.squidignore` file blocks access to specific files and directories  
3. **User Approval** - Explicit confirmation required before executing any operation

This multi-layered approach prevents unauthorized or unintended file system access.

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
│ 3. squid validates path (whitelist/blacklist + .squidignore)│
│    → If blocked: Friendly message returned to LLM           │
│                  (NO user prompt, LLM relays message)       │
│    → If allowed: Continue to step 4                         │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. squid prompts user for approval                          │
│                                                             │
│    ┌───────────────────────────────────────────────────┐    │
│    │ Allow reading file: README.md? (Y/n)              │    │
│    │ [Y] Allow  [N] Skip                               │    │
│    └───────────────────────────────────────────────────┘    │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. User makes decision                                      │
│    • Press Y → Tool executes, result sent to LLM            │
│    • Press N → Tool skipped, error sent to LLM              │
└─────────────────────────────────────────────────────────────┘
```

**Note:** The LLM will still make tool call requests for protected files because it doesn't know which files are blocked beforehand. However, these requests are rejected instantly without user interaction, and the LLM receives a friendly message to relay to the user.

## Security Features

### 🛡️ Path Validation (Whitelist/Blacklist)

All file system operations are validated against whitelist and blacklist rules **before** requesting user approval.

**Default Whitelist:**
- Current directory (`.`) and subdirectories
- Current working directory path

**Default Blacklist (System Paths):**
- `/etc`, `/bin`, `/sbin`, `/usr/bin`, `/usr/sbin`
- `/root`, `/var`, `/sys`, `/proc`
- `C:\Windows`, `C:\Program Files` (Windows)

**Default Blacklist (Sensitive User Files):**
- `~/.ssh/` - SSH keys
- `~/.gnupg/` - GPG keys
- `~/.aws/` - AWS credentials
- `~/.config/gcloud/` - Google Cloud credentials

**Behavior:**
- Paths outside the whitelist are **automatically rejected**
- Paths in the blacklist are **automatically rejected**
- User is **never prompted** for rejected paths
- Clear error messages explain why access was denied

**Example:**

```bash
squid ask "Read /etc/passwd"

# What happens:
# 1. LLM requests to read /etc/passwd
# 2. Path validation blocks it (blacklisted)
# 3. Friendly message returned to LLM (no user prompt)
# 4. LLM relays the message:
#
# 🦑: I cannot access '/etc/passwd' because it's a protected system file 
# or directory. Access to this location is blocked for security reasons.
```

### 📂 Ignore Patterns (.squidignore)

Use a `.squidignore` file in your project root to specify files and directories that should never be accessed by squid tools. This works like `.gitignore`.

**Creating .squidignore:**

```bash
# .squidignore - One pattern per line
# Lines starting with # are comments

# Build outputs
target/
dist/
*.o

# Dependencies  
node_modules/
__pycache__/

# Secrets
.env
*.key
*.pem

# Logs
*.log
logs/

# OS files
.DS_Store
```

**Pattern Syntax:**

- `*.txt` - All .txt files in any directory
- `**/*.log` - All .log files recursively
- `target/` - Entire target directory
- `node_modules/**` - node_modules and all contents
- `# comment` - Comments start with #

**Priority:**

Ignore patterns are checked **after** whitelist/blacklist but **before** user approval:

```
1. Path validation (whitelist/blacklist) ← Automatic
2. Ignore patterns (.squidignore)        ← Automatic  
3. User approval prompt                  ← Manual
```

**Example:**

```bash
# Create .squidignore
echo "*.log" > .squidignore
echo ".env" >> .squidignore

squid ask "Read debug.log"

# What happens:
# 1. LLM requests to read debug.log
# 2. Path validation blocks it (.squidignore pattern match)
# 3. Friendly message returned to LLM (no user prompt)
# 4. LLM relays the message:
#
# 🦑: I cannot access 'debug.log' because it's protected by the project's 
# .squidignore file. This is a security measure to prevent access to 
# sensitive files.
```



### 🔒 User Approval Required

All tool executions require explicit user approval. The LLM cannot read, write, or search files without user consent.

```bash
squid ask "Read my secret.txt file"
# Prompt: Allow reading file: secret.txt? (Y/n)
# → You control whether this happens

squid ask "Search for passwords in the project"
# Prompt: Allow searching for pattern 'passwords' in: .? (Y/n)
# → You control whether this happens
```

### 📋 Content Preview for Write Operations

When the LLM attempts to write a file, you see a preview of the content before approving:

```bash
squid ask "Create a config.json file with default settings"

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
squid ask "Find all TODO comments in src"

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

## Direct File Access Commands

In addition to LLM tool calls, squid provides direct file access via command-line flags:
- `ask -f <file>` - Passes file content directly to the LLM with your question
- `review <file>` - Reads file for code review

**Security measures:**
- **Path validation applied BEFORE reading** - Same whitelist/blacklist and `.squidignore` checks as tool calls
- **No user approval prompt** - Validation happens immediately when command runs
- **Friendly error messages** - Clear explanation if file is blocked
- **Cannot bypass security** - No way to override validation rules

**Example blocked access:**
```bash
$ squid ask -f .env "what's in this file?"
🦑: I can't access that file - it's in your .squidignore list.

$ squid review ~/.ssh/id_rsa
🦑: I can't access that file - it's outside the project directory or in a protected system location.
```

**Example allowed access:**
```bash
$ squid ask -f src/main.rs "explain this code"
# File is validated, then read and passed to LLM
# No approval prompt needed - you explicitly requested it via -f flag
```

**Why direct file access is safe:**
1. You explicitly specify the file path in the command
2. Same security validation as tool calls (blacklist, whitelist, .squidignore)
3. Path validation happens before file is read
4. Clear error messages if access is denied

**Key difference from tool calls:**
- Tool calls: LLM decides what to read → validation → user approval → read
- Direct access: You decide what to read → validation → read (no approval needed since you chose the file)

## Best Practices

### ✅ DO

- **Review file paths carefully** before approving read operations
- **Check content previews** for write operations
- **Deny suspicious requests** - You can always run the command again
- **Use absolute paths** when possible to avoid ambiguity
- **Enable debug logging** if you want more detailed information:
  ```bash
  RUST_LOG=debug squid ask "your question"
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
squid ask "What environment variables do I have?"
```

**LLM may try:**
```
Tool: read_file
Path: .env
```

**What happens:**
1. LLM requests to read `.env` (doesn't know it's blocked)
2. Path validation checks `.env` against `.squidignore`
3. `.env` is blocked automatically (in default `.squidignore`)
4. **You are NOT prompted** - access denied immediately
5. LLM receives friendly message: "I cannot access '.env' because it's protected by the project's .squidignore file. This is a security measure to prevent access to sensitive files."
6. LLM relays the message to the user
7. Your sensitive data stays protected without user interaction needed

**Note:** The LLM will still consume tokens for the initial request and the follow-up response, but no sensitive data is ever accessed.

### Scenario 2: Validating Write Operations

**Command:**
```bash
squid ask "Create a startup script for this project"
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
squid ask "Read all .rs files in src/ and create a summary"
```

**LLM behavior:**
- Requests to read each file individually
- Path validation passes for project files
- You approve each one: src/main.rs ✓, src/lib.rs ✓, etc.
- If LLM tries to read ignored files (e.g., `target/`), they're blocked automatically
- Finally requests to write summary.txt
- Path validation passes, you review the summary content
- Approve if it looks correct

## Logging and Audit

All tool operations are logged at the INFO level. To view these logs:

```bash
# Default (shows INFO and above)
RUST_LOG=info squid ask "your question"

# Debug (shows all logs including tool details)
RUST_LOG=debug squid ask "your question"

# Only errors
RUST_LOG=error squid ask "your question"
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

## Security Layers Summary

Squid employs **three layers of security** for file operations:

```
┌─────────────────────────────────────────────────────┐
│ Layer 1: Path Validation (Whitelist/Blacklist)      │
│ ✓ Blocks system directories (/etc, /root, etc.)     │
│ ✓ Blocks sensitive files (~/.ssh, ~/.aws, etc.)     │
│ ✓ Only allows current directory and subdirectories  │
│ → Automatic rejection, no user prompt               │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ Layer 2: Ignore Patterns (.squidignore)             │
│ ✓ Blocks files matching .squidignore patterns       │
│ ✓ Configurable per-project rules                    │
│ ✓ Works like .gitignore                             │
│ → Automatic rejection, no user prompt               │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ Layer 3: User Approval                              │
│ ✓ Explicit Y/N prompt for each operation            │
│ ✓ Content preview for write operations              │
│ ✓ Full transparency of tool arguments               │
│ → Manual approval required                          │
└─────────────────────────────────────────────────────┘
```

**Example Flow:**

```bash
squid ask "Read all files in my project and create a summary"

# Attempt 1: Read ~/.ssh/id_rsa
# → Layer 1 blocks (blacklisted path)
# ✗ Automatic rejection

# Attempt 2: Read node_modules/package/index.js  
# → Layer 2 blocks (in .squidignore)
# ✗ Automatic rejection

# Attempt 3: Read src/main.rs
# → Layers 1&2 pass
# → Layer 3 prompts user
# ? Allow reading file: src/main.rs? (Y/n)
```

## Future Enhancements

Potential future security features (not yet implemented):

- [ ] Automatic approval for specific safe operations
- [ ] Audit log file for all tool executions
- [ ] Configuration option to disable tools entirely
- [ ] Sandboxing or restricted file system access
- [ ] Rate limiting for tool calls
- [ ] Custom whitelist paths via configuration

## Reporting Security Issues

If you discover a security vulnerability in `squid`, please report it responsibly:

1. **DO NOT** create a public GitHub issue
2. Contact the maintainer directly (see `Cargo.toml` for author email)
3. Provide detailed information about the vulnerability
4. Allow time for a fix before public disclosure

## Summary

The security system in `squid` provides:

✅ **Path Validation** - Automatic blocking of system and sensitive directories  
✅ **Ignore Patterns** - Project-specific file blocking via .squidignore  
✅ **User Control** - Final approval for every file operation  
✅ **Transparency** - See what's being accessed and written  
✅ **Logging** - Complete audit trail of all operations  
✅ **Prevention** - Multi-layered defense against unintended or malicious actions  

Your security is our priority. Use `squid` with confidence knowing you're protected at multiple levels.
