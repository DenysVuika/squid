# System Prompts Reference

Squid uses specialized system prompts stored in `src/assets/` to guide the LLM's behavior for different commands.

## Prompt Architecture

Squid uses **modular prompt composition**:

```
[persona.md] + [task-specific prompt] = Complete System Prompt
```

- **`persona.md`** — Shared AI assistant personality (professional, direct, honest)
- **Task-specific** — `ask-prompt.md`, `code-review.md`, `review-*.md`

The persona is automatically prepended to all task-specific prompts at runtime.

## Available Prompts

### Ask Command (`ask-prompt.md`)

Default for `squid ask`. Guides the LLM to proactively use tools (read files, search code, run safe commands) when helpful.

```bash
squid ask "What is Rust?"                        # Uses ask-prompt.md
squid ask -p custom-prompt.md "question"         # Override with custom prompt
squid ask -f main.rs -p expert.md "Review this"  # Custom + file context
```

### Code Review Prompts

The `review` command automatically selects the appropriate prompt based on file extension:

| File Types | Prompt File | Focus |
|-----------|-------------|-------|
| `.rs` | `review-rust.md` | Ownership, memory safety, idioms |
| `.ts`, `.js`, `.tsx`, `.jsx` | `review-typescript.md` | Type safety, async patterns, XSS |
| `.html`, `.htm` | `review-html.md` | Semantics, accessibility, SEO |
| `.css`, `.scss`, `.sass` | `review-css.md` | Performance, responsive, maintainability |
| `.py` | `review-py.md` | Python best practices |
| `.go` | `review-go.md` | Go idioms, concurrency |
| `.java` | `review-java.md` | Java patterns, null safety |
| `.sql` | `review-sql.md` | Query optimization, injection |
| `.sh` | `review-sh.md` | Shell scripting, safety |
| `.json` | `review-json.md` | Schema, structure |
| `.yaml`, `.yml` | `review-yaml.md` | Syntax, structure |
| `.md` | `review-md.md` | Markdown style, links |
| `Dockerfile`, `*.dockerfile` | `review-docker.md` | Container best practices |
| `Makefile` | `review-makefile.md` | Build patterns |
| Other | `code-review.md` | Generic code quality |

```bash
squid review src/main.rs          # Auto-selects review-rust.md
squid review App.tsx              # Auto-selects review-typescript.md
squid review index.html           # Auto-selects review-html.md
```

## Custom Prompts

Use the `-p`/`--prompt` flag with `squid ask` to override the default system prompt:

```bash
# Create a custom prompt
cat > security-expert.md << 'EOF'
You are a security expert. Focus on vulnerabilities and best practices.
EOF

# Use it
squid ask -p security-expert.md "Review this auth code"
squid ask -f auth.rs -p security-expert.md "Find security issues"
```

To permanently modify built-in prompts, edit the files in `src/assets/` and rebuild (`cargo build --release`).

## Viewing Prompts

```bash
ls src/assets/*.md          # List all prompts
cat src/assets/persona.md   # View base persona
```

## Version History

- **v0.4.0**: Added `-p`/`--prompt` flag, modular persona architecture
- **v0.3.0**: Initial specialized review prompts
