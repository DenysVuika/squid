# Template Variables

Agent prompts support variable substitution using the [Tera template engine](https://keats.github.io/tera/). Variables are automatically available in agent prompts (defined in `agents/*.md` files) and system prompts.

## Available Variables

| Variable | Example | Description |
|----------|---------|-------------|
| `{{persona}}` | — | Base AI personality and tool usage guidelines from `src/assets/persona.md` |
| `{{now}}` | `2026-03-28T12:34:56+00:00` | Current timestamp in ISO 8601 format |
| `{{date}}` | `2026-03-28` | Current date |
| `{{time}}` | `12:34:56` | Current time |
| `{{year}}` | `2026` | Current year |
| `{{month}}` | `03` | Current month |
| `{{day}}` | `28` | Current day |
| `{{timestamp}}` | `1711629296` | Unix timestamp |
| `{{timezone}}` | `UTC` | Timezone name |
| `{{timezone_offset}}` | `+0000` | Timezone offset |
| `{{os}}` | `macOS`, `Linux` | Operating system name |
| `{{os_version}}` | `14.4` | OS version |
| `{{kernel_version}}` | `23.4.0` | Kernel version |
| `{{arch}}` | `aarch64`, `x86_64` | System architecture |
| `{{os_family}}` | `unix`, `windows` | OS family |

## Usage Examples

### Standard Agent (with persona)

Include `{{persona}}` at the start of custom agent prompts to preserve base personality and tool usage guidelines:

```yaml
---
name: Code Reviewer
model: qwen2.5-coder
permissions:
  - read_file
  - grep
---
{{persona}}

You are an expert code reviewer on {{os}} ({{arch}}) at {{now}}. Focus on security and performance.
```

### Fully Custom Agent (without persona)

For specialized agents with completely custom behavior, omit `{{persona}}`:

```yaml
---
name: Captain Squidbeard
model: qwen2.5-coder
permissions:
  - now
---
Ye be Captain Squidbeard 🏴‍☠️, a cunning pirate squid sailin' the seven seas of code!
```

This creates agents with no inherited guidelines — useful for demos, experiments, or highly specialized personalities. Pair with `use_tools: false` for persona agents that should never invoke tools; the Tools button will be hidden automatically in the Web UI.

![Custom Prompt](docs/assets/custom-prompts-pirate.png)

## Advanced Templates

Tera supports conditionals, loops, and filters. See the [Tera documentation](https://keats.github.io/tera/) for advanced features.

Example with conditional logic:

```
{% if os == "macOS" %}
You are running on macOS with {{arch}} architecture.
{% else %}
You are running on {{os}}.
{% endif %}
```
