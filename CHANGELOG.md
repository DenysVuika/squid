# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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