# AI Coding Agents Guidelines

## Project Overview

**squid** is a CLI-first AI coding assistant with a web UI frontend. It connects to OpenAI-compatible LLM APIs (LM Studio, Ollama, OpenAI, Mistral, etc.) and provides tool-calling capabilities for file operations, code search, and safe bash execution.

### Architecture

```
CLI (clap) → Agent → LLM (async-openai) → Tool Calls → Tools (tools.rs)
                                                    ↓
Server (actix-web) ← SSE Streaming ← Response ←─────┘
       ↓
  Web UI (Vite/React/TypeScript)
       ↓
  SQLite (rusqlite + sqlite-vec for RAG)
```

**Key technologies:**
- **Backend**: Rust 2024 edition, actix-web server, async streaming via SSE
- **LLM integration**: `async-openai` crate, OpenAI-compatible API
- **Database**: SQLite with vector embeddings (`rusqlite` + `sqlite-vec`)
- **Templating**: Tera for agent prompt variable substitution
- **Plugins**: JavaScript sandbox via `rquickjs` (QuickJS runtime)
- **Frontend**: React + TypeScript, Vite dev server, shadcn/ui components

---

## File Organization

| Directory       | Purpose                                      |
|-----------------|----------------------------------------------|
| **Root**        | Essential files (README, CHANGELOG, LICENSE) |
| **src/**        | Rust backend source code                     |
| **src/assets/** | LLM system prompts (persona, review prompts) |
| **agents/**     | Agent definitions (YAML frontmatter + prompt)|
| **plugins/**    | JavaScript plugin definitions                |
| **web/src/**    | React/TypeScript frontend                    |
| **docs/**       | Minimal additional documentation             |
| **tests/**      | Test scripts (executable, well-commented)    |
| **sample-files/** | Example files for testing                  |
| **static/**     | Built web UI assets (auto-generated)         |
| **migrations/** | Database migrations                          |

---

## Documentation Philosophy

### **Minimal Documentation**
Only maintain these files:
- **README.md** (main project documentation)
- **CHANGELOG.md** (version history)
- **src/assets/*.md** (LLM system prompts)
- **docs/** (only for complex features, kept to a minimum)

**Avoid:**
- Summary files (SUMMARY.md, COMPLETION.md)
- Redundant or duplicate documentation
- Implementation details unless critical
- ASCII trees or folder diagrams

---

## Feature Development Workflow

1. **Implement** the feature in code.
2. **Update prompts** in `src/assets/*.md` if behavior changes.
3. **Update README.md** with usage examples.
4. **Update CHANGELOG.md** (unreleased section).
5. **Optional**: Add **one** doc in `docs/` if the feature is complex.

**Naming**: Use `FEATURE_NAME.md` (no "summary" or "guide" suffixes).

---

## Built-in Tools

The LLM has access to 6 built-in tools defined in `src/tools.rs`:

| Tool | Description | Security |
|------|-------------|----------|
| `read_file` | Read file contents | Path validation, `.squidignore` |
| `write_file` | Write to files with preview | Path validation, user approval |
| `grep` | Regex search across files | Respects `.squidignore`, skips binaries |
| `now` | Get current date/time | None needed |
| `bash` | Execute shell commands | Dangerous commands always blocked |
| `demo_tool` | Testing approval workflows | None |

**Security model:**
- Allow-list only — anything not explicitly permitted is denied
- Dangerous bash (`rm`, `sudo`, `chmod`, `dd`, `curl`, `wget`, `kill`) is **always** blocked
- Path validation via `PathValidator` respects `.squidignore` patterns
- All file operations require user approval in the Web UI
- Plugins use capability-based permissions separate from built-in tools

**Adding a tool:** Update `get_tools()` and `call_tool()` in `src/tools.rs`, then update `persona.md` and `README.md`.

---

## System Prompts (`src/assets/*.md`)

### **Modular Architecture**
- **`persona.md`**: Shared personality and tool usage guidelines (available as `{{persona}}` template variable).
- **Task-specific**: `ask-prompt.md`, `code-review.md`, `review-*.md`.

**Composition**:
- Built-in commands: `persona.md` + task-specific prompt
- Agent custom prompts: Use `{{persona}}` variable to include base personality

**Update Rules**:
- **`persona.md`**: Role, tone, personality, and tool usage guidelines.
- **Task prompts**: Command-specific instructions.
- **Agent prompts**: Include `{{persona}}` at the start to preserve core behavior.

**Example Agent Prompt**:
```json
{
  "prompt": "{{persona}}\n\nYou are an expert code reviewer working on {{os}} ({{arch}}). Focus on security and performance."
}
```

### **Template Variables**
Agent prompts support Tera template syntax. Available variables: `{{persona}}`, `{{now}}`, `{{os}}`, `{{arch}}`, `{{date}}`, `{{time}}`, `{{year}}`, `{{timestamp}}`, `{{timezone}}`, `{{os_version}}`, `{{kernel_version}}`, `{{os_family}}`.
See `src/template.rs` for implementation.

---

## Web UI Components

- **`web/src/components/ai-elements/`**: Reusable UI components — **DO NOT MODIFY** unless fixing bugs.
- **`web/src/components/app/`**: Application-specific components — can be freely modified.
- **`web/src/components/ui/`**: shadcn-style UI primitives (button, dialog, accordion, etc.).
- **Component customization**: Extend ai-elements behavior in parent components, not by changing ai-elements themselves.
- **Example**: Instead of adding props to `Source` component, handle clicks in the parent `chatbot.tsx`.

**State management**: Zustand stores in `web/src/stores/` (agent, chat, config, session — each with tests).

**API client**: `web/src/lib/chat-api.ts` — TypeScript client for the REST API.

---

## Testing

- **Location**: `tests/` (executable shell scripts, well-commented).
- **Web UI**: Component tests alongside stores in `web/src/stores/*.test.ts`.
- **Preference**: Automated tests over manual instructions.
- **Test scripts**: Use `./tests/test-*.sh` pattern, run with `bash tests/test-<feature>.sh`.
- **Plugin testing**: Use `squid chat` to interactively test plugins.

---

## Version Control

- Follow [Semantic Versioning](https://semver.org/).
- **CHANGELOG.md**: User-focused entries (no technical jargon).

**Good Example**:
✅ "Enhanced Tool Availability: Tools are now available in code review commands."

**Bad Example**:
❌ "Refactored prompt composition to use `include_str!` macro."

---

## Tool Development

1. Add tool to `src/tools.rs` (`get_tools()`, `call_tool()`).
2. Update **README.md** and **CHANGELOG.md**.
3. Update security docs if needed (`docs/SECURITY*.md`).

**Note**: Tool definitions are sent via OpenAI API (name, description, parameters). General tool usage guidelines are in `persona.md`.

---

## Key Conventions

- **Error handling**: `thiserror` for library errors, `anyhow` for application errors
- **Async**: Tokio runtime, all I/O is async, streaming via SSE (`async-stream`)
- **Serialization**: `serde` + `serde_json` / `serde_yaml` for config
- **CLI**: `clap` derive macros for command definitions
- **Logging**: Structured logging with levels (error, warn, info, debug, trace)
- **Database**: `rusqlite` with bundled SQLite, migrations in `migrations/`
- **Embeddings**: `sqlite-vec` for vector search (RAG feature)
- **File watching**: `notify` crate for hot-reload detection

---

## Documentation Summary

- **README.md**: Comprehensive enough to minimize additional docs.
- **CHANGELOG.md**: Tracks user-facing changes.
- **src/assets/*.md**: Controls LLM behavior.
- **docs/**: Only for complex features (one doc per feature max).

**Goal**: Just enough documentation to understand and use features—nothing more.
