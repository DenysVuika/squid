# File Context Feature - Architecture Documentation

## Overview

The file context feature allows users to provide a file to the `squid` CLI, which will be read and included in the prompt sent to the LLM. This enables the AI to answer questions based on the file's content.

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                    User Executes Command                    │
│  squid ask --file document.txt "What is this about?"        │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                  CLI Argument Parsing                       │
│  - question: "What is this about?"                          │
│  - file: Some("document.txt")                               │
│  - stream: false                                            │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    File Reading                             │
│  std::fs::read_to_string("document.txt")                    │
│  Returns: Ok(String) or Err                                 │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                 Prompt Construction                         │
│                                                             │
│  if file_content.is_some():                                 │
│    "Here is the content of the file:                        │
│                                                             │
│     ```                                                     │
│     [FILE CONTENT HERE]                                     │
│     ```                                                     │
│                                                             │
│     Question: [USER QUESTION]"                              │
│                                                             │
│  else:                                                      │
│    "[USER QUESTION]"                                        │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                  LLM Request Creation                       │
│  Messages:                                                  │
│  1. System: "You are a helpful assistant..."                │
│  2. User: [Constructed prompt with file content]            │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│              Send to LLM (LM Studio/OpenAI)                 │
│  POST to API_URL/chat/completions                           │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                  Response Processing                        │
│  - Streaming: Display tokens as received                    │
│  - Non-streaming: Display complete response                 │
└─────────────────────────────────────────────────────────────┘
```

## Code Architecture

### Key Components

#### 1. CLI Definition (`Commands` enum)

```rust
Ask {
    question: String,
    #[arg(short, long)]
    stream: bool,
    #[arg(short, long)]
    file: Option<PathBuf>,
}
```

The `file` parameter is optional, allowing the feature to work with or without file context.

#### 2. File Reading (in `main()`)

```rust
let file_content = if let Some(file_path) = file {
    match std::fs::read_to_string(file_path) {
        Ok(content) => Some(content),
        Err(e) => {
            error!("Failed to read file: {}", e);
            return;
        }
    }
} else {
    None
};
```

- Reads the entire file into a `String`
- Error handling prevents crashes on file read failures
- Returns early if file cannot be read

#### 3. Prompt Construction (in `ask_llm` and `ask_llm_streaming`)

```rust
let user_message = if let Some(content) = file_content {
    format!(
        "Here is the content of the file:\n\n```\n{}\n```\n\nQuestion: {}",
        content, question
    )
} else {
    question.to_string()
};
```

- Wraps file content in markdown code blocks for better formatting
- Clearly separates file content from the question
- Falls back to just the question if no file is provided

#### 4. System Message Enhancement

```rust
ChatCompletionRequestSystemMessageArgs::default()
    .content("You are a helpful assistant. When provided with file content, analyze it carefully and answer questions based on that content.")
```

- Enhanced system prompt instructs the LLM to focus on file content
- Ensures better responses when file context is provided

## Data Flow

### Without File Context

```
User Question → LLM → Response
```

### With File Context

```
User Question + File Content → Combined Prompt → LLM → Response
```

## Error Handling

### File Read Errors

- **File not found**: Returns early with error message
- **Permission denied**: Returns early with error message
- **Invalid UTF-8**: Returns early with error message

All errors are logged using the `error!` macro from the `log` crate.

## Performance Considerations

### Memory Usage

- Entire file is loaded into memory
- For large files (>10MB), consider:
  - Chunking strategies
  - Streaming file reads
  - File size validation

### API Limits

- Most LLMs have token limits (context window)
- GPT-4: ~128k tokens (~96k words)
- GPT-3.5: ~16k tokens (~12k words)
- Local models: Varies (often 2k-8k tokens)

**Recommendation**: Add file size/token validation before sending to API

## Future Enhancements

### Potential Improvements

1. **Multiple File Support**
   ```bash
   squid ask --file file1.txt --file file2.txt "Compare these files"
   ```

2. **File Type Detection**
   - Auto-format based on extension (.rs, .py, .md, etc.)
   - Syntax highlighting hints in prompt

3. **Token Counting**
   - Warn user if file + question exceeds model's context window
   - Truncate intelligently if needed

4. **File Size Limits**
   ```rust
   const MAX_FILE_SIZE: u64 = 1_000_000; // 1MB
   ```

5. **Binary File Handling**
   - Detect binary files and reject them
   - Or provide base64 encoding for image analysis (with vision models)

6. **Directory Support**
   ```bash
   squid ask --dir ./src "Analyze this codebase"
   ```

7. **Caching**
   - Cache file content if asking multiple questions about same file
   - Reuse parsed content

8. **Preprocessing**
   - Remove comments
   - Minify code
   - Extract only relevant sections

## Security Considerations

### File Access

- Currently reads any file the user has permission to read
- Consider adding:
  - Path validation (prevent path traversal)
  - Allowed directory whitelist
  - File type restrictions

### Data Privacy

- File contents are sent to the LLM API
- Users should be aware:
  - Local models (LM Studio): Data stays local
  - Remote APIs (OpenAI): Data sent to third party

### Recommendations

1. Add `--confirm` flag for sensitive files
2. Display file size before sending
3. Warn when using remote APIs
4. Add `.squidignore` file support

## Testing

### Manual Testing

```bash
# Test with sample file
squid ask -f sample.txt "What is this about?"

# Test with non-existent file
squid ask -f nonexistent.txt "Test"

# Test with code file
squid ask -f src/main.rs "Explain this code"

# Test with streaming
squid ask -f README.md -s "Summarize"

# Test without file (original functionality)
squid ask "What is Rust?"
```

### Edge Cases

- Empty file
- Very large file (>1GB)
- Binary file
- File with special characters
- File path with spaces
- Relative vs absolute paths
- Symlinks

## Integration

### Compatible Models

- ✅ OpenAI GPT-3.5/4
- ✅ LM Studio (local models)
- ✅ Ollama (via OpenAI compatibility)
- ✅ Any OpenAI-compatible API

### Dependencies

- `std::fs::read_to_string`: File reading
- `std::path::PathBuf`: Path handling
- `clap`: CLI argument parsing
- `async-openai`: API client

## Examples

See `EXAMPLES.md` for comprehensive usage examples.

## Support

For issues or questions:
1. Check `README.md` for configuration
2. Review `EXAMPLES.md` for usage patterns
3. Enable debug logging: `RUST_LOG=debug squid ask ...`
