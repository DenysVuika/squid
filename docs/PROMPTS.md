# System Prompts Reference

This document describes all the system prompts used in the `squid` CLI tool.

## Overview

Squid uses specialized system prompts to guide the LLM's behavior for different commands. These prompts are stored in `src/assets/` and are compiled into the binary.

## Available Prompts

### 1. Ask Command Prompt (`ask-prompt.md`)

**Used by:** `ask` command (default)  
**Location:** `src/assets/ask-prompt.md`  
**Purpose:** General-purpose assistant with intelligent tool usage

**Key features:**
- Guides LLM on when to use `read_file` and `write_file` tools
- Provides clear examples of tool usage scenarios
- Encourages proactive file reading based on user questions
- Helps LLM understand natural language file requests

**Examples of when this prompt is active:**
```bash
cargo run -- ask "Read Cargo.toml and list dependencies"
cargo run -- ask "What's in the README file?"
cargo run -- ask "Create a notes.txt with my tasks"
```

---

### 2. Code Review Prompts

#### Generic Code Review (`code-review.md`)

**Used by:** `review` command (fallback for unknown file types)  
**Location:** `src/assets/code-review.md`  
**Purpose:** General code quality review

**Focus areas:**
- Code quality and best practices
- Potential bugs and issues
- Performance considerations
- Maintainability and readability

#### Rust Review (`review-rust.md`)

**Used by:** `review` command for `.rs` files  
**Location:** `src/assets/review-rust.md`  
**Purpose:** Rust-specific code review

**Focus areas:**
- Ownership and borrowing
- Memory safety
- Idiomatic Rust patterns
- Error handling
- Performance (unnecessary clones, allocations)
- Safety (unwrap, panic, unsafe)

**Example:**
```bash
cargo run -- review src/main.rs
```

#### TypeScript/JavaScript Review (`review-typescript.md`)

**Used by:** `review` command for `.ts`, `.js`, `.tsx`, `.jsx` files  
**Location:** `src/assets/review-typescript.md`  
**Purpose:** TypeScript and JavaScript code review

**Focus areas:**
- Type safety (`any` usage, type assertions)
- Modern ES6+ features
- Async/await patterns
- Security (XSS, injection vulnerabilities)
- Memory leaks (event listeners, closures)
- Performance optimizations

**Example:**
```bash
cargo run -- review src/App.tsx
```

#### HTML Review (`review-html.md`)

**Used by:** `review` command for `.html`, `.htm` files  
**Location:** `src/assets/review-html.md`  
**Purpose:** HTML semantic and accessibility review

**Focus areas:**
- Semantic HTML elements
- Accessibility (ARIA, alt text, labels)
- SEO best practices
- Performance (inline scripts/styles)
- Document structure
- Forms and interactive elements

**Example:**
```bash
cargo run -- review index.html
```

#### CSS Review (`review-css.md`)

**Used by:** `review` command for `.css`, `.scss`, `.sass` files  
**Location:** `src/assets/review-css.md`  
**Purpose:** CSS performance and maintainability review

**Focus areas:**
- Performance (selectors, animations)
- Responsive design
- Maintainability (naming, organization)
- Browser compatibility
- Accessibility (focus styles, contrast)
- Modern CSS features

**Example:**
```bash
cargo run -- review styles/main.css
```

## How Prompts Are Selected

### Ask Command

The `ask` command always uses `ask-prompt.md` unless a custom system prompt is provided.

```rust
// In src/main.rs
const ASK_PROMPT: &str = include_str!("./assets/ask-prompt.md");
```

### Review Command

The `review` command automatically selects the appropriate prompt based on file extension:

```rust
fn get_review_prompt_for_file(file_path: &Path) -> &'static str {
    match extension {
        "rs" => CODE_REVIEW_RUST_PROMPT,
        "ts" | "js" | "tsx" | "jsx" => CODE_REVIEW_TYPESCRIPT_PROMPT,
        "html" | "htm" => CODE_REVIEW_HTML_PROMPT,
        "css" | "scss" | "sass" => CODE_REVIEW_CSS_PROMPT,
        _ => CODE_REVIEW_PROMPT,  // Generic fallback
    }
}
```

## Customizing Prompts

To customize a prompt:

1. Edit the corresponding `.md` file in `src/assets/`
2. Rebuild the project: `cargo build --release`
3. The new prompt is now compiled into the binary

**Example:** Customize Rust reviews
```bash
# Edit the prompt
vim src/assets/review-rust.md

# Rebuild
cargo build --release

# Test
./target/release/squid review sample-files/example.rs
```

## Prompt Best Practices

When editing prompts, follow these guidelines:

### 1. Be Specific
- Clearly define the LLM's role and capabilities
- List specific areas to focus on
- Provide concrete examples

### 2. Use Examples
- Show example user requests and expected behavior
- Demonstrate tool usage patterns
- Include edge cases

### 3. Structure Clearly
- Use headings and sections
- Bullet points for lists
- Separate different concerns

### 4. Guide Tool Usage
- Explicitly state when to use tools
- Provide clear criteria for tool invocation
- Include examples of tool-triggering phrases

### 5. Set Expectations
- Define output format
- Specify level of detail
- Clarify response style

## Adding New Prompts

To add a new specialized prompt:

1. **Create the prompt file:**
   ```bash
   touch src/assets/review-python.md
   ```

2. **Define the constant in `main.rs`:**
   ```rust
   const CODE_REVIEW_PYTHON_PROMPT: &str = include_str!("./assets/review-python.md");
   ```

3. **Add to the selection logic:**
   ```rust
   fn get_review_prompt_for_file(file_path: &Path) -> &'static str {
       match extension {
           "py" => CODE_REVIEW_PYTHON_PROMPT,  // New!
           // ... existing matches
       }
   }
   ```

4. **Write the prompt content** (see existing prompts for structure)

5. **Test thoroughly** with example files

## Prompt Templates

### Basic Structure

```markdown
# Role Definition
You are a [specific role] assistant...

## Available Tools (if applicable)
1. **tool_name** - Description
   - When to use
   - Examples

## When to [Perform Action]
- Criterion 1
- Criterion 2
- Examples

## Focus Areas
- Area 1: Description
- Area 2: Description

## Response Format
- How to structure responses
- Level of detail
- Tone and style

## Examples
**User**: "Example question"
**You**: [Expected behavior]
```

## Viewing Prompts

To view the current prompts:

```bash
# View all prompts
ls -la src/assets/*.md

# View a specific prompt
cat src/assets/ask-prompt.md

# View with formatting
bat src/assets/review-rust.md  # if you have bat installed
```

## Troubleshooting

### LLM not using tools as expected

**Problem:** LLM doesn't read files when asked  
**Solution:** Check `ask-prompt.md` includes clear tool usage examples

### Irrelevant review feedback

**Problem:** Code reviews focus on wrong areas  
**Solution:** Update the specific review prompt to emphasize desired focus areas

### Verbose or terse responses

**Problem:** Responses too long or too short  
**Solution:** Adjust "Response Style" section in the prompt

## Version History

- **v0.3.0**: Added `ask-prompt.md` for intelligent tool usage
- **v0.3.0**: Initial specialized review prompts (Rust, TypeScript, HTML, CSS)

## Related Documentation

- **[Security Features](SECURITY.md)** - Tool approval and security
- **[Examples](EXAMPLES.md)** - Comprehensive usage examples and workflows

---

**Note:** All prompts are compiled into the binary at build time. Changes require rebuilding the project.