# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **File Context Feature**: Added ability to pass files to the CLI for AI analysis
  - New `--file` / `-f` flag for the `ask` command
  - Automatically reads file content and includes it in the prompt
  - Works with both streaming and non-streaming modes
  - Supports any text-based file format (code, docs, config, data, etc.)
  - Comprehensive error handling for file read failures
  - Enhanced system prompt to better handle file-based questions
  
- **Documentation**:
  - `QUICKSTART.md` - Quick start guide for new users
  - `EXAMPLES.md` - Comprehensive usage examples
  - `FILE_CONTEXT.md` - Technical architecture documentation
  - `docs/sample.txt` - Sample file for testing the feature
  - Updated `README.md` with file context examples

### Changed
- Enhanced `ask_llm()` function to accept optional file content parameter
- Enhanced `ask_llm_streaming()` function to accept optional file content parameter
- Improved system message to instruct LLM on handling file content
- Updated CLI argument structure to support file paths

### Technical Details
- File content is read using `std::fs::read_to_string()`
- Content is wrapped in markdown code blocks for better formatting
- Graceful error handling prevents crashes on file read failures
- Compatible with LM Studio, OpenAI, and any OpenAI-compatible API

## [0.1.0] - Initial Release

### Added
- Basic CLI structure with `init`, `run`, and `ask` commands
- OpenAI-compatible API support
- Streaming response support with `--stream` / `-s` flag
- Environment variable configuration via `.env` file
- Support for LM Studio local models
- Support for OpenAI remote API
- Logging system with debug levels
- Async/await architecture using Tokio

### Features
- Ask questions to LLM without file context
- Real-time streaming responses
- Configurable API endpoint, model, and key
- Comprehensive error handling
- Cross-platform support (macOS, Linux, Windows)

### Dependencies
- `async-openai` v0.24.1 - OpenAI API client
- `clap` v4.5.56 - CLI argument parsing
- `tokio` - Async runtime
- `dotenvy` - Environment variable loading
- `log` / `env_logger` - Logging framework
- `futures` - Stream processing

[Unreleased]: https://github.com/yourusername/squid/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/squid/releases/tag/v0.1.0