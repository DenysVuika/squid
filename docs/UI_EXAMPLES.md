# UI Examples

This document shows examples of Squid's enhanced terminal UI.

## Assistant Responses

All responses from the AI assistant are now prefixed with the squid emoji for personality:

```
$ squid ask "What is Rust?"

🦑: Rust is a systems programming language that focuses on safety, speed, 
and concurrency. It achieves memory safety without using a garbage collector...
```

## Tool Approval Prompts

Tool approval prompts now feature styled, colorful formatting with emoji icons.

### Reading a File

```
🦑 Can I read this file?
  📄 File: src/main.rs
→ Y to allow, N to deny [y/N]
```

### Writing a File

```
🦑 Can I write to this file?
  📄 File: config.json
  📝 Content preview:
{
  "api_url": "http://localhost:1234/v1",
  "model": "llama-3.2"
}
→ Y to allow, N to deny [y/N]
```

For large files, the content preview is truncated:

```
🦑 Can I write to this file?
  📄 File: large_document.md
  📝 Content preview:
# Large Document

This is a very long document with lots of content that will be truncated... (2048 bytes total)
→ Y to allow, N to deny [y/N]
```

### Searching Files (grep)

```
🦑 Can I search for this pattern?
  🔍 Pattern: TODO
  📂 Path: src
→ Y to allow, N to deny [y/N]
```

## Color Scheme

The terminal output uses a consistent color scheme:

- **Cyan Bold**: Headers and section titles
- **Yellow**: Actions being performed ("read this file", "write to this file")
- **Green**: File and directory paths
- **Magenta**: Search patterns
- **Dim**: Preview content
- **Red Bold**: "N" (No/Deny) indicator
- **Green Bold**: "Y" (Yes/Allow) indicator

## Emoji Guide

- 🦑 - Squid assistant / Tool requests
- 📄 - File operations
- 📂 - Directory/path operations
- 🔍 - Search operations
- 📝 - Content preview
- → - Action prompt indicator

## Interactive Examples

### Example 1: Code Review with Tool Call

```
$ squid review src/main.rs

🦑 Can I read this file?
  📄 File: src/main.rs
→ Y to allow, N to deny [y/N] y

🦑: I'll review this code for you.

The main.rs file shows a well-structured CLI application using Clap...
```

### Example 2: Ask with Multiple Tool Calls

```
$ squid ask "What dependencies does this project use?"

🦑 Tool Request wants to read a file
  📄 File: Cargo.toml
→ Y to allow, N to deny [y/N] y

🦑: Based on the Cargo.toml file, this project uses the following dependencies:

1. **async-openai** (v0.32) - For communicating with OpenAI-compatible APIs
2. **clap** (v4.5) - Command-line argument parsing with derive macros
3. **console** (v0.15) - Terminal styling and colors
...
```

### Example 3: Search Operation

```
$ squid ask "Find all TODO comments in the code"

🦑 Tool Request wants to search files
  🔍 Pattern: TODO
  📂 Path: src
→ Y to allow, N to deny [y/N] y

🦑: Found 3 matches for pattern 'TODO' in src:

  - src/main.rs:145 — // TODO: Add support for batch processing
  - src/tools.rs:89 — // TODO: Implement file watching
  - src/config.rs:23 — // TODO: Add validation for API URLs
```

### Example 4: Declining a Tool Call

```
$ squid ask "Delete all temporary files"

🦑 Tool Request wants to search files
  🔍 Pattern: *.tmp
  📂 Path: .
→ Y to allow, N to deny [y/N] n

🦑: I wasn't able to search for the files since the operation was declined. 
Would you like to manually specify which files to delete, or would you prefer 
a different approach?
```

## Non-Interactive Mode

When running in a non-interactive environment (CI/CD, scripts), the prompts 
gracefully fall back with appropriate error messages:

```
Error: Failed to get user approval: not a tty
```

## Terminal Compatibility

The enhanced UI works with:
- ✅ Modern terminals with color support (iTerm2, Terminal.app, Windows Terminal, etc.)
- ✅ tmux and screen sessions
- ✅ SSH sessions
- ✅ VS Code integrated terminal
- ⚠️  Terminals without color support will show plain text (graceful degradation)

## Accessibility

- Color is supplementary - information is also conveyed through emoji and text
- Help text clearly indicates keyboard shortcuts
- Default action is always "No" for safety
- All prompts are screen-reader friendly