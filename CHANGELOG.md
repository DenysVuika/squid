# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Init Command Creates .squidignore**: The `squid init` command now automatically creates a `.squidignore` file
  - Creates file with default patterns from `.squidignore.example` template
  - Skips creation if `.squidignore` already exists (won't overwrite)
  - Embedded template using `include_str!` for portability
  - Provides helpful feedback about file creation or existing file usage

- **Path Validation Security**: Multi-layered security for file system operations
  - Whitelist/blacklist validation automatically blocks sensitive system directories
  - Default blacklist includes `/etc`, `/root`, `~/.ssh`, `~/.aws`, `~/.gnupg`, etc.
  - Default whitelist restricts operations to current directory and subdirectories
  - **Validation happens BEFORE user approval prompts** - blocked files never prompt user
  - Clear error messages explain why access was denied
  - New `validate.rs` module with `PathValidator` for secure path handling
  - Windows-specific blacklist for `C:\Windows`, `C:\Program Files`, etc.

- **.squidignore Support**: Project-specific file and directory ignore patterns
  - Works like `.gitignore` - one pattern per line, `#` for comments
  - Glob pattern support: `*.log`, `**/*.rs`, `target/`, `node_modules/**`
  - Filename-only patterns (e.g., `.env`, `*.tmp`) match anywhere in path tree
  - **Checked BEFORE user approval** - ignored files never prompt user
  - Automatic enforcement - files cannot be accessed even if user approves
  - Patterns loaded from `.squidignore` file in project root
  - Included `.squidignore.example` with common patterns
  - Auto-created by `squid init` command for convenience
  - Friendly, personalized error messages when files are blocked

- **Enhanced Security Documentation**: Comprehensive security guide
  - Updated `docs/SECURITY.md` with path validation and ignore patterns documentation
  - Three-layer security model clearly explained (validation → ignore → approval)
  - Example workflows and security scenarios with friendly error messages
  - Pattern syntax reference and best practices
  - Security layers diagram for visual understanding

- **Friendly Error Messages**: Path validation errors are now conversational
  - Instead of technical errors, the assistant explains why it can't access files
  - Returned as `content` (not `error`) so LLM can relay them directly without additional explanation
  - Example: "I cannot access '.env' because it's protected by the project's .squidignore file. This is a security measure to prevent access to sensitive files."
  - Different messages for different validation failures (ignored, blacklisted, not whitelisted)
  - Technical error details logged at debug level (default log level is `error`, keeping output clean)
  - **Note:** LLM still makes tool call requests for blocked files (it doesn't know beforehand which are blocked), but requests are rejected instantly without user interaction, and the friendly message is relayed to the user

- **Improved Streaming Output**: Cleaner formatting for assistant responses
  - Leading newlines are stripped from streaming responses
  - Content now appears directly after `🦑:` emoji without extra blank lines
  - Applies to both initial responses and follow-up responses after tool calls

### Changed

- **Tool Calling Signature**: `call_tool()` now requires `Config` parameter
  - Enables future extensibility for configuration-based features
  - Updated both `ask_llm_streaming()` and `ask_llm()` to pass config to tools
  - Config cloned for async tool execution in spawned tasks

### Security

- **Hardened File System Access**: All file operations now validated against security rules
  - `read_file`, `write_file`, and `grep` tools use path validation
  - Prevents access to system directories without user prompt
  - Prevents access to ignored files without user prompt
  - Three-layer defense: path validation → ignore patterns → user approval

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
