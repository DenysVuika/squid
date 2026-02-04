# Instructions for AI Agents

This file contains guidelines for AI coding assistants working on this project. Following the [agents.md](https://agents.md/) specification.

## Documentation Philosophy

### Minimal Documentation Set

**IMPORTANT:** Keep documentation **absolutely minimal** and focused. Only generate/update these files:

1. **README.md** - Main project documentation (always keep updated)
2. **CHANGELOG.md** - Version history and release notes (always keep updated)
3. **src/assets/*.md** - System prompts for the LLM (critical for behavior)
4. **docs/** - Additional documentation **only when absolutely necessary** (keep to bare minimum)

**DO NOT CREATE:**
- ❌ Summary documents (SUMMARY.md, COMPLETION.md, CHANGES.md, etc.)
- ❌ Multiple documentation files for the same feature
- ❌ "Quick test" or "quick start" files if README.md suffices
- ❌ Implementation details documents unless truly needed for complex features
- ❌ Redundant documentation that repeats README.md or CHANGELOG.md

### Critical Update Areas

When making changes, **ALWAYS** consider updates to:

✅ **README.md** - Update features, usage examples, and documentation links
✅ **CHANGELOG.md** - Add to unreleased section or appropriate version
✅ **src/assets/*.md** - Update system prompts if tool behavior changes

### Documentation Anti-Patterns

- ❌ Don't create excessive documentation files
- ❌ Don't duplicate information across multiple files
- ❌ Don't create documentation for the sake of documentation
- ❌ Don't leave orphaned or outdated docs
- ❌ **Don't create summary documents** - information should be in README/CHANGELOG
- ❌ **Don't create multiple guides for one feature** - one concise doc is enough
- ❌ **Don't document the obvious** - code should be self-documenting where possible
- ❌ **Don't generate folder/file structure diagrams** - they're hard to maintain and go out of sync
- ❌ **Don't include ASCII trees** unless absolutely critical for understanding

## File Organization

- **Root** - Only essential files (README, CHANGELOG, LICENSE, etc.)
- **docs/** - All markdown documentation goes here
- **tests/** - Test scripts and test-related code only (no MD files)
- **src/assets/** - System prompts and LLM instructions
- **sample-files/** - Example files for testing code review features

## When Adding New Features

1. **Code first** - Implement the feature
2. **Update prompts** - If it's a tool or command, update `src/assets/*.md`
3. **Update README.md** - Add feature to list and usage examples (this should be enough for most features)
4. **Update CHANGELOG.md** - Add to unreleased section
5. **Optional docs** - Only create ONE additional doc in docs/ if the feature is complex and README.md isn't sufficient
   - Name it clearly (e.g., `FEATURE_NAME.md`, not `FEATURE_SUMMARY.md` or `FEATURE_GUIDE.md`)
   - Keep it concise and focused on understanding the feature, not implementation details

## System Prompts (src/assets/*.md)

These files control LLM behavior and are **critical**:

### Modular Prompt Architecture

Squid uses a **modular composition** system (as of v0.4.0):

- `persona.md` - **Shared personality** definition (auto-prepended to all prompts)
- `ask-prompt.md` - Tool usage instructions for the `ask` command
- `code-review.md` - Generic code review criteria
- `review-*.md` - Language-specific review prompts

At runtime, prompts are composed as: `persona.md` + task-specific prompt

**Benefits:**
- Single source of truth for AI personality
- Consistent tone across all commands
- Easier maintenance (update persona once)
- Task prompts focus only on instructions

**When updating:**
- **persona.md** - Defines who the assistant is (role, personality, tone)
- **Task prompts** - Define what to do (instructions, guidelines, examples)
- Be explicit and clear
- Use examples
- Test with the actual LLM
- Consider edge cases
- Update tool descriptions when adding new tools

## Testing

- Test scripts go in `tests/` directory
- Test documentation goes in `docs/` directory
- Keep test scripts executable and well-commented
- Automated tests preferred over manual instructions

## Version Control

- Follow [Semantic Versioning](https://semver.org/)
- Update CHANGELOG.md for all notable changes
- Group related changes in CHANGELOG under appropriate sections:
  - Added, Changed, Deprecated, Removed, Fixed, Security

## Tool Development

When adding new tools to `src/tools.rs`:

1. Add tool definition to `get_tools()`
2. Add approval prompt in `call_tool()`
3. Add execution logic in `call_tool()` match statement
4. **Update `src/assets/ask-prompt.md`** with tool documentation (instructions only)
5. Consider if `persona.md` needs updates (usually not - it's about personality, not tools)
6. Update README.md with examples
7. Update CHANGELOG.md
8. Update security docs if relevant (docs/SECURITY*.md)

## Project-Specific Notes

- This is a CLI tool for AI-powered code assistance
- Security is important - all file operations require user approval
- Tool calling is a core feature
- LM Studio and OpenAI compatibility required
- Streaming support must be maintained

## Documentation Philosophy Summary

**Minimal is better.** The goal is to have just enough documentation to understand and use the features, nothing more.

- **README.md** should be comprehensive enough that users rarely need other docs
- **CHANGELOG.md** tracks what changed and when
- **src/assets/*.md** controls LLM behavior
- **docs/** is for truly complex features that can't fit concisely in README.md
- **One feature = one doc maximum** (if any doc beyond README is even needed)
- **No summaries, no completion docs, no redundant guides**

## Questions?

See existing files for patterns and examples:
- README.md for documentation style
- CHANGELOG.md for change tracking format
- src/assets/ask-prompt.md for prompt engineering patterns
