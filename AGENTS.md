# AI Coding Agents Guidelines

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

## File Organization

| Directory       | Purpose                                      |
|-----------------|----------------------------------------------|
| **Root**        | Essential files (README, CHANGELOG, LICENSE) |
| **docs/**       | Minimal additional documentation             |
| **tests/**      | Test scripts (no markdown)                   |
| **src/assets/** | LLM system prompts                           |
| **sample-files/** | Example files for testing                  |

---

## Feature Development Workflow

1. **Implement** the feature in code.
2. **Update prompts** in `src/assets/*.md` if behavior changes.
3. **Update README.md** with usage examples.
4. **Update CHANGELOG.md** (unreleased section).
5. **Optional**: Add **one** doc in `docs/` if the feature is complex.

**Naming**: Use `FEATURE_NAME.md` (no "summary" or "guide" suffixes).

---

## System Prompts (`src/assets/*.md`)

### **Modular Architecture**
- **`persona.md`**: Shared personality (auto-prepended).
- **`tools.md`**: Tool instructions (auto-included).
- **Task-specific**: `ask-prompt.md`, `code-review.md`, `review-*.md`.

**Composition**: `persona.md` + `tools.md` + task-specific prompt.

**Update Rules**:
- **`persona.md`**: Role, tone, and personality.
- **`tools.md`**: Tool availability and usage.
- **Task prompts**: Command-specific instructions.

---

## Testing
- **Location**: `tests/` (executable scripts, well-commented).
- **Docs**: `docs/` (only if necessary).
- **Preference**: Automated tests > manual instructions.

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
2. Update `src/assets/tools.md` with instructions.
3. Update **README.md** and **CHANGELOG.md**.
4. Update security docs if needed (`docs/SECURITY*.md`).

---

## Project-Specific Notes
- **CLI tool** for AI-powered code assistance.
- **Security**: All file operations require user approval.
- **Compatibility**: LM Studio and OpenAI APIs.
- **Streaming**: Must be maintained.

### **Web UI Components**
- **`web/src/components/ai-elements/`**: Reusable UI components - **DO NOT MODIFY** unless fixing bugs.
- **`web/src/components/app/`**: Application-specific components - Can be freely modified.
- **Component customization**: Extend ai-elements behavior in parent components, not by changing ai-elements themselves.
- **Example**: Instead of adding props to `Source` component, handle clicks in the parent `chatbot.tsx`.

---

## Documentation Summary
- **README.md**: Comprehensive enough to minimize additional docs.
- **CHANGELOG.md**: Tracks user-facing changes.
- **src/assets/*.md**: Controls LLM behavior.
- **docs/**: Only for complex features (one doc per feature max).

**Goal**: Just enough documentation to understand and use features—nothing more.
