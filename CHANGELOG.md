# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Reasoning Mode Support**: View LLM's thinking process in the Web UI
  - Added support for `<think>...</think>` tags in streaming responses
  - Reasoning content parsed and displayed separately from regular response
  - Collapsible reasoning sections with automatic open/close during streaming
  - **Real-time duration tracking**: Shows "Thinking..." shimmer while reasoning, then "Thought for X seconds"
  - Frontend parses `<think>` tags from streaming content for live display
  - Backend parses tags from accumulated content before saving to database
  - **Reasoning persisted to database** - thinking process saved with messages and restored on session load
  - Added `reasoning` column to messages table (migration 007)
  - Reasoning content displayed on page reload (shows "Thought for a few seconds")
  - "Thinking..." placeholder hidden once reasoning starts
  - Content saved without `<think>` tags; reasoning stored separately for clean display
  - Duration calculated from when `<think>` appears to when `</think>` closes
  - Reasoning component manages timing internally via `isStreaming` prop

- **Centralized Pricing Model Metadata**: Token pricing estimation now uses backend metadata
  - Added `pricing_model` field to model metadata JSON for mapping models to pricing references
  - Removed hardcoded model-to-pricing mappings from frontend
  - Backend now provides pricing model information via `/api/models` endpoint
  - All models (Qwen, DeepSeek, Llama, Mistral, etc.) now have consistent pricing estimation
  - Added `default_pricing_model` to metadata for unknown models (defaults to `gpt-4o`)

- **Model Selection in Web UI**: Dynamic model selector with real-time model discovery
  - New `/api/models` endpoint fetches available models from LLM provider (LM Studio, Ollama, etc.)
  - Models augmented with metadata including friendly names, context window sizes, and providers
  - Model selector defaults to Qwen Coder 2.5 (preferred for coding tasks)
  - Models grouped by provider for easy navigation
  - Selected model is passed to API with each request, overriding config default
  - Built-in model metadata for 30+ popular models (Qwen, Llama, GPT, Claude, Gemini, etc.)
  - Unknown models handled gracefully with 8192 token default context
- **Per-Request Model Override**: Chat requests can now specify a model
  - Optional `model` field in `/api/chat` endpoint
  - Falls back to `api_model` from config if not specified
  - Allows Web UI to use different models without changing configuration

### Fixed

- **Cost Calculation for Liquid Models**: Added support for LFM/Liquid models in pricing estimation
  - Models like `lfm2.5-1.2b-thinking` and `liquid/lfm2.5-1.2b` now properly mapped to GPT-4o-mini for cost comparison
- **Model Selector Sync**: Model selector now updates when switching between sessions
  - Previously, the model selector would not reflect the model used in a loaded session
  - Now correctly displays and selects the model associated with each session
  - Includes fuzzy matching to handle model ID changes (e.g., `qwen2.5-coder` → `qwen2.5-coder-7b-instruct-mlx`)
  - Old sessions with outdated model IDs will automatically select the closest matching current model
- **Context Window Updates**: Context window size now updates when switching models
  - Ensures accurate token usage percentage and prevents calculation errors
  - Automatically uses the selected model's max context length

### Changed

- **Default Model**: Changed from `local-model` to `qwen2.5-coder-7b-instruct` for better defaults
  - More specific default that works well with LM Studio and Ollama
  - Still overridable via config or Web UI model selector

## [0.8.0] - 2026-02-13

### Added

- **Context Window Configuration**: Configure model context limits for accurate usage tracking
  - New `context_window` field in `squid.config.json` (e.g., `32768` for Qwen2.5-Coder)
  - Context utilization percentage displayed in web UI
  - Helps prevent API errors from exceeding context limits
  - Can be set via `squid init --context-window 32768` or environment variable `CONTEXT_WINDOW`
- **Token Usage & Cost Tracking**: Real-time token counts and cost estimates
  - Visual token usage indicator in chat header with percentage and breakdown
  - Track input, output, reasoning, and cache tokens separately
  - Cost calculation based on model pricing (via tokenlens/models.dev)
  - Automatic token estimation for local models (LM Studio, Ollama) using tiktoken-rs
  - Fallback pricing for local models maps them to similar OpenAI models (e.g., Qwen → GPT-4o) to show estimated cost savings
- **Session Management**: Browse, rename, and organize chat sessions
  - Sessions automatically titled from first user message
  - Sidebar for browsing all past conversations
  - Rename any session with inline editing
  - Delete sessions with confirmation
  - Smart date formatting and auto-refresh
- **Shimmer Loading Indicator**: Visual feedback while AI is thinking
- **File Content Deduplication**: Identical file attachments are stored only once in the database
  - Reduces database size when same files are attached multiple times
  - Uses SHA-256 hashing to identify duplicate content
- **File Content Compression**: All file attachments are compressed with gzip in the database
  - Automatic compression/decompression when saving and loading files
  - Significantly reduces storage requirements
  - Logging shows compression ratio for each file
- **File Size Limits**: Files larger than 10MB are rejected with clear error messages
  - Prevents accidentally uploading very large files
  - Protects database from growing too large
- **Clickable Source Files**: Click on "Used X sources" to view file contents in a right sidebar
  - Syntax-highlighted code view with line numbers
  - Auto-detects language from file extension (supports 40+ languages)
  - Copy button to copy entire file content
  - Horizontal and vertical scrolling for large files
  - 600px wide sidebar slides in from the right
  - Easy to close and review what context the AI used
- **File Size Validation**: Client-side validation for file uploads
  - 10MB file size limit enforced in browser before upload
  - User-friendly error messages when files are too large
  - Backend also validates file size and content type

### Fixed

- **Sources Display on Restored Sessions**: "Used X sources" now only shows on assistant messages, not user messages
- **User Messages Not Persisting**: Fixed messages being deleted when switching sessions
- **Token Counts for Local Models**: Token estimates now display correctly in UI
- **Database Migrations**: Prevented duplicate migrations and "duplicate column" errors
- **Session List Scrolling**: Session sidebar now properly scrolls when many sessions are present

### Changed

- New chats start empty (no demo messages)
- Sidebar layout for session management
- Navigation bar with Chat and Logs pages

## [0.7.0] - 2026-02-11

### Added

- **Environment Context**: LLM receives system information with each request for context-aware responses
  - Includes OS details, CPU architecture, platform, timezone, and timestamps
  - Privacy-focused: hostname and working directory excluded by default
  - Helps LLM provide OS-specific commands and timezone-aware suggestions
- **Extended Code Review Language Support**: Added language-specific review prompts for 10 additional file types
  - **Python** (`.py`, `.pyw`, `.pyi`) - PEP 8, security, performance
  - **SQL** (`.sql`, `.ddl`, `.dml`) - SQL injection, indexes, transactions
  - **Shell Scripts** (`.sh`, `.bash`, `.zsh`, `.fish`) - Command injection, portability
  - **Docker** (`Dockerfile`, `Dockerfile.*`) - Security, resource limits, health checks
  - **Go** (`.go`) - Concurrency, race conditions, error handling
  - **Java** (`.java`) - Collections, memory leaks, Spring framework
  - **JSON** (`.json`) - Sensitive data, schema validation
  - **YAML** (`.yaml`, `.yml`) - Sensitive data, schema validation
  - **Makefile** (`Makefile`, `Makefile.*`) - POSIX compliance, .PHONY targets
  - **Markdown** (`.md`, `.markdown`) - Accessibility, broken links, heading hierarchy

### Changed

- **Improved Code Review Prompts**: All review prompts refactored to focus on issues first
  - Issue-focused structure: Problem → Fix → Why
  - Eliminates praise and style preferences
  - Now supports 14 file types: Rust, TypeScript/JavaScript, HTML, CSS, Python, SQL, Shell, Docker, Go, Java, JSON, YAML, Makefile, Markdown
- **Token Usage Logging**: Changed from INFO to DEBUG level for cleaner default output

### Removed

- **`now` Tool**: Removed redundant datetime tool since environment context now provides comprehensive time information (local time, UTC time, Unix timestamp, and timezone) with every request

## [0.6.0] - 2026-02-05

### Added

- **Config Version Warnings**: Get notified when your config file is outdated
  - Shows ⚠️ warning when config doesn't match current app version
  - Suggests running `squid init` to update

- **Smart Config Updates**: Re-running `squid init` preserves your settings
  - Uses existing values as defaults in prompts
  - **Smart permission merging**: Preserves your custom permissions + adds new defaults
  - Automatically adds new default permissions (e.g., `"now"`) while keeping your customizations
  - Easy way to update config without starting from scratch

- **Tool Permissions**: Configure which tools can run automatically or should be blocked
  - New `permissions` section in `squid.config.json` with `allow` and `deny` arrays
  - Interactive prompts now offer: Yes (once), No (skip), Always (auto-allow), Never (auto-deny)
  - Choosing Always/Never automatically saves to config file
  - See [Security Documentation](docs/SECURITY.md#-tool-permissions-allowdeny-lists) for details

- **Mistral API Support**: Works with Mistral's OpenAI-compatible endpoint
  - Example: `API_URL=https://api.mistral.ai/v1`, `API_MODEL=devstral-2512`
  - Supports all Mistral models

- **Datetime Tool**: New `now` tool for current date/time queries
  - Supports UTC and local timezones
  - Returns RFC 3339 format

- **Bash Tool**: New `bash` tool for executing safe, non-destructive commands
  - Execute read-only commands like `ls`, `git status`, `cat`, `pwd`
  - **MANDATORY blocking** of dangerous commands - cannot be bypassed by configuration or user approval
  - Dangerous patterns (`rm`, `sudo`, `chmod`, `dd`, `curl`, `wget`, `kill`) blocked at code level
  - Configurable timeout (default 10 seconds, max 60 seconds)
  - Integrated with permission system (allow/deny lists)
  - User approval required for each command execution (except dangerous commands, blocked immediately)
  - **Granular permissions**: Fine-grained control with `"bash:command"` format
    - `"bash:ls"` - allows only ls commands
    - `"bash:git status"` - allows only git status
    - `"bash"` - allows all bash commands (dangerous patterns still blocked)
    - Automatically saves granular permissions when choosing "Always" or "Never"

- **Loading Spinner**: Shows "Waiting for squid..." during streaming responses

### Changed

- **Enhanced Tool Availability**: Tools now available in code review commands (previously only in `ask`)

### Security

- **Mandatory Dangerous Command Blocking**: Bash tool security cannot be bypassed
  - Dangerous commands (`rm`, `sudo`, `chmod`, `dd`, `curl`, `wget`, `kill`) are **always blocked**
  - Blocking happens at code level before any permission checks or user approval
  - Cannot be bypassed by adding to allow list or any configuration setting
  - See [Security Documentation](docs/SECURITY.md#bash) for details

## [0.5.0] - 2026-02-04

### Added

- **.squidignore Support**: Protect sensitive files with project-specific ignore patterns
  - Works like `.gitignore` - one pattern per line, `#` for comments
  - Glob pattern support: `*.log`, `**/*.rs`, `target/`, `node_modules/**`
  - Automatically prevents the AI from accessing ignored files
  - Run `squid init` to create a `.squidignore` file with sensible defaults
  - Example patterns: `.env`, `*.key`, `**/.git/**`, `node_modules/**`

- **Enhanced Security**: Automatic protection for sensitive system files
  - Blocks access to system directories like `/etc`, `/root`, `~/.ssh`, `~/.aws`
  - Protects Windows system folders like `C:\Windows`, `C:\Program Files`
  - Blocks sensitive files before asking for your approval
  - Works alongside `.squidignore` for comprehensive protection

- **Friendly Error Messages**: Clear, conversational feedback when things go wrong
  - File not found: "🦑: I can't find that file. Please check the path and try again."
  - Permission denied: "🦑: I don't have permission to read that file."
  - Blocked files: "I cannot access '.env' because it's protected by the project's .squidignore file."
  - No more cryptic technical error messages

### Fixed

- **Security Gap Closed**: The `ask -f` and `review` commands now respect security rules
  - Previously could bypass `.squidignore` and path validation
  - Now properly blocks sensitive files before reading them
  - Provides friendly error messages explaining why access was denied

- **Cleaner Output**: Improved formatting for better readability
  - Removed extra blank lines in assistant responses
  - Content appears directly after `🦑:` emoji
  - Smoother streaming experience

### Changed

- **Documentation Updates**: Improved README and security documentation
  - Emphasizes privacy-focused and local-first design
  - Clarifies data privacy with local models vs. cloud APIs
  - Comprehensive security guide with examples and best practices

## [0.4.0] - 2026-02-03

### Fixed

- **File Context in Ask Command**: Fixed issue where LLM didn't know the actual filename when using `-f` flag
  - Previously, only file content was sent without the filename
  - LLM would guess incorrect file paths (e.g., `./config.json` instead of `squid.config.json`)
  - Now includes filename in the context: "Here is the content of the file 'squid.config.json':"
  - Applies to both streaming and non-streaming modes

### Changed

- **Default Log Level**: Changed default log level from `info` to `error`
  - Reduces noise in normal operation
  - Fixed in both `logger.rs` and `config.rs` to ensure consistent defaults
  - Users can still set to `info`, `debug`, or `trace` for more verbose output
  - Configure via `squid init`, `LOG_LEVEL` environment variable, or `squid.config.json`

- **Personalized Tool Approval Prompts**: Tool requests now use first-person conversational language
  - Changed from "Tool Request wants to read a file" to "Can I read this file?"
  - Changed from "Tool Request wants to write to a file" to "Can I write to this file?"
  - Changed from "Tool Request wants to search files" to "Can I search for this pattern?"
  - Makes the assistant feel more personal and conversational

- **Modular Prompt Architecture**: Introduced `persona.md` for shared AI personality
  - Separated persona definition from task-specific instructions
  - All prompts now composed at runtime: `persona.md` + task prompt
  - `ask-prompt.md` now focuses only on tool usage instructions
  - All review prompts updated to remove conflicting "You are an expert..." statements
  - Review prompts now use instruction-based headers (e.g., "## Code Review Instructions")
  - Easier to maintain consistent personality across all commands
  - Single source of truth for AI assistant behavior and tone

### Added

- **Personality Enhancement**: Assistant responses now prefixed with squid emoji 🦑
  - Adds friendly personality while maintaining professional tone
  - Updated system prompt to reflect intelligent squid assistant persona
  - Applied to both streaming and non-streaming responses

- **Enhanced Tool Approval UI**: Styled and visually improved tool approval prompts
  - Added `console` crate for colored terminal output
  - Tool requests now display with emoji icons (🦑 📄 🔍 📂 📝)
  - Color-coded information (cyan headers, green files, yellow actions, magenta patterns)
  - Multi-line formatted prompts with clear visual hierarchy
  - Styled help text with bold Y/N indicators
  - See `docs/TUI_OPTIONS.md` for more UI enhancement options

- **Custom System Prompts**: New `-p`/`--prompt` flag for `ask` command
  - Override the default system prompt with a custom prompt from a file
  - Useful for specialized tasks (security analysis, performance review, domain-specific expertise)
  - Can be combined with file context (`-f`) and other flags
  - Examples: `squid ask -p expert.md "question"` or `squid ask -f code.rs -p reviewer.md "review"`
  - No rebuild required - change prompts on the fly
  - See `docs/PROMPTS.md` for detailed guide on creating custom prompts

- **Init Command**: Interactive and non-interactive configuration setup via `squid init`
  - Accepts optional directory parameter (defaults to current directory)
  - Usage: `squid init` or `squid init /path/to/project`
  - Creates directory if it doesn't exist
  - **Interactive mode**: Prompts for API URL, API Model, optional API Key, and Log Level
  - **Non-interactive mode**: Use CLI flags to skip prompts
    - `--url <URL>` - API URL
    - `--model <MODEL>` - API Model
    - `--api-key <KEY>` - API Key (optional)
    - `--log-level <LEVEL>` - Log Level
    - Example: `squid init --url http://127.0.0.1:1234/v1 --model local-model --log-level info`
    - Partial parameters supported (will prompt for missing values)
  - Creates `squid.config.json` in the specified directory for project settings
  - Configuration file can be committed to share team settings (like `.eslintrc`, `.prettierrc`)
  - Configuration file takes precedence over environment variables
  - Falls back to `.env` variables if config file doesn't exist
  - Supports all existing LLM providers (LM Studio, Ollama, OpenAI, etc.)
  - Best practice: commit `squid.config.json`, keep sensitive API keys in `.env`
- **Configurable Log Level**: Control logging verbosity via config or environment
  - Set via `squid init` with interactive prompt (error, warn, info, debug, trace)
  - Stored in `squid.config.json` or `LOG_LEVEL` environment variable
  - Default level is `error` (minimal noise)
  - Config file setting takes precedence over `LOG_LEVEL` environment variable

### Removed

- **Run Command**: Removed unimplemented `squid run` command from CLI
  - Command was never fully implemented and had no practical use
  - Simplified CLI interface to focus on core features (init, ask, review)
  - Updated documentation to remove all references to the run command

## [0.3.0] - 2026-02-02

### Changed

- **Streaming is now the default behavior**: Responses stream in real-time by default
  - Replaced `--stream` / `-s` flag with `--no-stream` flag
  - Use `--no-stream` to get complete response at once (useful for scripting/piping)
  - Improved user experience with immediate feedback
  - Both streaming and non-streaming modes fully support tool calling
  - Updated all documentation and examples to reflect new default behavior

- **Enhanced Documentation Prerequisites**: Comprehensive setup guides for multiple LLM providers
  - Detailed LM Studio setup with Qwen2.5-Coder model recommendation
  - Complete Ollama installation and configuration guide
  - OpenAI API setup instructions
  - Support for other OpenAI-compatible services (OpenRouter, Together AI, etc.)
  - Updated `.env.example` with examples for all providers
  - Enhanced configuration documentation in README.md and QUICKSTART.md

### Added

- **AGENTS.md**: Added comprehensive guidelines for AI coding assistants working on this project
  - Minimal documentation philosophy
  - File organization rules
  - Documentation anti-patterns to avoid
  - Guidelines for adding new features

- **Tool Calling with Security Approval**: LLM can now interact with the file system safely
  - **Tools available**:
    - `read_file` - Read file contents from the filesystem
    - `write_file` - Write content to files
    - `grep` - Search for patterns in files using regex (supports files and directories)
  - **Intelligent tool usage**:
    - Comprehensive system prompt guides LLM on when to use tools
    - LLM understands natural language requests like "read Cargo.toml" or "analyze main.rs"
    - Proactive file reading based on context and user questions
  - **Grep tool features**:
    - Regex pattern matching with configurable case sensitivity
    - Recursive directory search or single file search
    - Configurable result limits (default: 50)
    - Automatic binary file filtering
    - Returns file path, line number, and matched content
  - **Security features**:
    - ✅ User approval required for every tool execution
    - ✅ File write operations show content preview before approval
    - ✅ Interactive Y/N prompts for each operation
    - ✅ All tool calls are logged for transparency and audit
    - ✅ Default deny - prompts default to "No" for safety
  - Works with both streaming and non-streaming modes
  - Works with both `ask` and `review` commands
  - See `docs/SECURITY.md` for details

- **Code Review Command**: New `review` command for AI-powered code reviews
  - Automatically selects appropriate review prompt based on file type
  - Language-specific prompts for Rust, TypeScript/JavaScript, HTML, and CSS
  - Generic fallback prompt for other file types
  - Optional `-m, --message` flag for focused reviews or specific questions
  - Streaming support with `-s, --stream` flag
  - Example files in `sample-files/` directory for testing
  - Test script: `tests/test-reviews.sh`

- **Enhanced `ask` Command**:
  - Added optional `-m, --message` flag for additional context or instructions
  - Tool calling support (with security approval)

- **Testing Infrastructure**:
  - `tests/test-security.sh` - Interactive security approval demonstrations
  - `tests/test-reviews.sh` - Automated code review testing
  - Comprehensive test documentation in `tests/README.md`

- **Documentation**:
  - `docs/SECURITY.md` - Comprehensive security features guide
  - `docs/PROMPTS.md` - System prompts reference
  - `docs/EXAMPLES.md` - Comprehensive usage examples
  - Updated all documentation with new features

- **Prompts and System Messages**:
  - `src/assets/ask-prompt.md` - Comprehensive system prompt for `ask` command
  - Detailed guidance on when and how to use tools
  - Examples and best practices for tool usage
  - Improves LLM understanding of file-related requests

### Changed

- **Documentation improvements**:
  - **Consolidation**: Removed redundant files (REVIEW_GUIDE.md, SECURITY_APPROVAL.md, FILE_CONTEXT.md)
  - **Consistency**: All docs now use `squid` command instead of `cargo run --` (for post-installation usage)
  - **Organization**: Moved `sample.txt` from `docs/` to `sample-files/` directory
  - **Installation guide**: Enhanced with `cargo install --path .` option and clear usage instructions
  - **Cleanup**: Removed unused `DATABASE_URL` references from code and documentation
  - Kept only essential docs: README, CHANGELOG, EXAMPLES, SECURITY, PROMPTS, QUICKSTART
  - Improved maintainability and user experience

- Reorganized project structure:
  - Moved all test scripts to `tests/` directory
  - All documentation now in `docs/` directory
  - Extracted tool logic into `src/tools.rs` module
  - Better code organization and maintainability

## [0.2.0]

### Added
- **File Context Feature**: Pass files to the CLI for AI analysis
  - New `--file` / `-f` flag for the `ask` command
  - Works with both streaming and non-streaming modes
  - Supports any text-based file format

## [0.1.0] - Initial Release

### Added
- Basic CLI with `ask` command
- OpenAI-compatible API support
- Streaming response support with `--stream` / `-s` flag
- Environment variable configuration via `.env` file
- Support for LM Studio local models and OpenAI API

## Summary

- **v0.3.0**: Streaming by default, tool calling with security approval, code reviews, enhanced documentation
- **v0.2.0**: File context feature
- **v0.1.0**: Initial release with basic ask command

[0.3.0]: https://github.com/yourusername/squid/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/yourusername/squid/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/squid/releases/tag/v0.1.0
