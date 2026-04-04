# Plugin Development Guide

Squid supports a powerful JavaScript-based plugin system that allows you to extend its capabilities with custom tools. Plugins can be invoked by the LLM alongside built-in tools like `read_file`, `write_file`, and `grep`.

## Table of Contents

- [Quick Start](#quick-start)
- [Plugin Structure](#plugin-structure)
- [Plugin API](#plugin-api)
- [Security Model](#security-model)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Quick Start

### 1. Create a Plugin Directory

Plugins can be stored in three locations:

- **Workspace plugins**: `./plugins/` (project-specific)
- **Global plugins**: `~/.squid/plugins/` (shared across projects)
- **Bundled plugins**: Shipped with the executable (when installed from crates)

Workspace plugins override global plugins, which override bundled plugins with the same ID.

**Development note**: During development (`cargo build`), the `plugins/` directory is automatically copied to `target/debug/plugins/` so the executable can find bundled plugins alongside global and workspace plugins.

```bash
mkdir -p plugins/my-plugin
cd plugins/my-plugin
```

### 2. Create `plugin.json`

```json
{
  "id": "my-plugin",
  "title": "My First Plugin",
  "description": "A simple example plugin",
  "version": "0.1.0",
  "api_version": "1.0",
  "security": {
    "requires": ["read_file"],
    "network": false,
    "file_write": false
  },
  "input_schema": {
    "type": "object",
    "properties": {
      "message": {
        "type": "string",
        "description": "Message to process"
      }
    },
    "required": ["message"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "result": {
        "type": "string",
        "description": "Processed message"
      }
    },
    "required": ["result"]
  }
}
```

### 3. Create `index.js`

```javascript
function execute(context, input) {
    try {
        context.log(`Processing message: ${input.message}`);
        
        // Your plugin logic here
        const result = input.message.toUpperCase();
        
        return { result };
    } catch (error) {
        return {
            error: error.message
        };
    }
}

globalThis.execute = execute;
```

### 4. Enable Plugin for Agent

Edit `squid.config.json`:

```json
{
  "agents": {
    "general-assistant": {
      "permissions": {
        "allow": [
          "read_file",
          "plugin:my-plugin",
          "plugin:*"
        ]
      }
    }
  }
}
```

### 5. Test Your Plugin

Start the server and ask the LLM to use your plugin:

```bash
squid server
```

Then in the web UI: "Please use the my-plugin tool to process the message 'hello world'"

## Plugin Structure

Every plugin consists of two required files:

```text
plugins/
└── my-plugin/
    ├── plugin.json      # Metadata and schemas
    └── index.js         # Implementation
```

### plugin.json

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | ✓ | Unique identifier (lowercase, hyphens) |
| `title` | string | ✓ | Human-readable name |
| `description` | string | ✓ | What the plugin does |
| `version` | string | ✓ | Semantic version (e.g., "0.1.0") |
| `api_version` | string | ✓ | Plugin API version (currently "1.0") |
| `security` | object | ✓ | Security requirements |
| `input_schema` | object | ✓ | JSON Schema for input validation |
| `output_schema` | object | ✓ | JSON Schema for output validation |

### Security Object

```json
{
  "security": {
    "requires": ["read_file", "write_file"],
    "network": false,
    "file_write": false
  }
}
```

- **`requires`**: Array of built-in tools the plugin needs (e.g., `read_file`, `grep`, `bash:ls`)
- **`network`**: Whether plugin needs HTTP access
- **`file_write`**: Whether plugin needs file write access

## Plugin API

### Context Object

Your `execute` function receives a `context` object with these APIs:

#### `context.readFile(path: string): string`

Read a file from the filesystem (respects `.squidignore`).

```javascript
const content = context.readFile("./README.md");
console.log("File size:", content.length);
```

**Throws**: Error if file doesn't exist or path is not allowed.

#### `context.log(message: string): void`

Log a message that will be saved to the server's database logs (appears in server logs and can be queried with `squid logs`).

```javascript
context.log("Processing started");
context.log("Found 5 issues");
```

**Note**: `console.log()` also works and is routed through the same logging system.

#### `context.writeFile(path: string, content: string): boolean`

Write content to a file (requires `file_write: true` permission).

```javascript
const success = context.writeFile("./output.txt", "Hello, World!");
if (success) {
    console.log("File written successfully");
}
```

**Throws**: Error if permission denied or path is not allowed.

#### `context.httpGet(url: string, timeout?: number): string`

Make an HTTP GET request (requires `network: true` permission).

```javascript
const html = context.httpGet("https://example.com", 10000);
console.log("Response length:", html.length);
```

**Parameters**:
- `url`: The URL to fetch
- `timeout`: Optional timeout in milliseconds (default: 5000)

**Throws**: Error if permission denied, network error, or non-200 status code.

#### `context.config.projectDir: string`

Get the current project directory.

```javascript
const dir = context.config.projectDir;
```

### Input and Output

Your plugin receives structured input and must return structured output that matches your schemas:

```javascript
function execute(context, input) {
    // input is validated against input_schema
    const result = processData(input);
    
    // return value is validated against output_schema
    return { result };
}
```

### Error Handling

Always wrap your logic in try-catch:

```javascript
function execute(context, input) {
    try {
        // Your logic
        return { success: true, data: result };
    } catch (error) {
        return {
            error: error.message,
            success: false
        };
    }
}
```

## Security Model

Squid uses a **hybrid security model**:

1. **Plugins declare** what they need (`security.requires`)
2. **Agents control** what plugins can run (`permissions.allow/deny`)
3. **User approves** plugin execution (via Web UI)

### Permission Levels

```json
{
  "permissions": {
    "allow": [
      "plugin:*",                  // Allow all plugins
      "plugin:markdown-linter",    // Allow specific plugin
      "read_file"                  // Required by plugins
    ],
    "deny": [
      "plugin:dangerous-plugin"    // Block specific plugin
    ]
  }
}
```

### Sandbox Restrictions

Plugins run in a sandboxed QuickJS environment:

- ❌ No `eval()` or `Function()` constructor
- ❌ No direct filesystem access (use context APIs)
- ❌ No network access unless `security.network = true`
- ✓ Memory limit: 128MB (configurable)
- ✓ Timeout: 30 seconds (configurable)

## Examples

### Example 1: Markdown Linter

Analyzes markdown files for style issues.

**plugin.json:**

```json
{
  "id": "markdown-linter",
  "title": "Markdown Linter",
  "description": "Lints markdown files for style issues",
  "version": "0.1.0",
  "api_version": "1.0",
  "security": {
    "requires": ["read_file"],
    "network": false,
    "file_write": false
  },
  "input_schema": {
    "type": "object",
    "properties": {
      "path": { "type": "string" },
      "max_line_length": { "type": "number", "default": 120 }
    },
    "required": ["path"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "issues": { "type": "array" },
      "stats": { "type": "object" }
    },
    "required": ["issues", "stats"]
  }
}
```

**index.js:**

```javascript
function execute(context, input) {
    const content = context.readFile(input.path);
    const maxLen = input.max_line_length || 120;
    
    const issues = [];
    const lines = content.split('\n');
    
    lines.forEach((line, i) => {
        if (line.length > maxLen) {
            issues.push(`Line ${i + 1}: Exceeds max length`);
        }
    });
    
    return {
        issues,
        stats: {
            lines: lines.length,
            headings: (content.match(/^#+\s+/gm) || []).length
        }
    };
}

globalThis.execute = execute;
```

### Example 2: Code Formatter

Formats code snippets.

See `plugins/code-formatter/` for complete implementation.

### Example 3: HTTP Fetcher

Fetches content from URLs (requires network permission).

See `plugins/http-fetcher/` for complete implementation.

## Best Practices

### 1. Validate Input

Always validate input even though schemas provide basic validation:

```javascript
function execute(context, input) {
    if (!input.path || typeof input.path !== 'string') {
        return { error: "Invalid path" };
    }
    // ...
}
```

### 2. Provide Useful Errors

Return structured error information:

```javascript
return {
    error: "File not found",
    details: {
        path: input.path,
        suggestion: "Check if the file exists"
    }
};
```

### 3. Use Logging

Log important operations for debugging and monitoring. Logs are saved to the database:

```javascript
context.log(`Processing file: ${input.path}`);
context.log(`Found ${issues.length} issues`);

// console.log() also works and goes to the same place
console.log("Debug info:", { count: 42 });
```

View plugin logs with:

```bash
squid logs --level info | grep Plugin
```

### 4. Keep It Fast

Plugins should complete within the timeout (default: 30s):

```javascript
// Good: Process in chunks
for (let i = 0; i < items.length; i += 100) {
    processChunk(items.slice(i, i + 100));
}

// Bad: Long synchronous operation
for (let i = 0; i < 1000000; i++) {
    heavyOperation(i);
}
```

### 5. Follow Naming Conventions

- Plugin ID: lowercase with hyphens (`markdown-linter`)
- Tool name: automatically prefixed (`plugin:markdown-linter`)
- Files: exactly `plugin.json` and `index.js`

## Troubleshooting

### Plugin Not Loading

Check server logs for errors:

```bash
squid server --log-level debug
```

Common issues:

- Invalid JSON in `plugin.json`
- Missing `execute` function in `index.js`
- Invalid API version (must be "1.0")

### Permission Denied

Ensure the agent has permission:

```json
{
  "permissions": {
    "allow": [
      "plugin:your-plugin",
      "read_file"  // If plugin requires it
    ]
  }
}
```

### Validation Errors

Check that your input/output matches the schemas:

```javascript
// Input must match input_schema
return {
    issues: [],  // Must be array
    stats: {     // Must be object with required fields
        lines: 0,
        headings: 0
    }
};
```

### Timeout Issues

Increase timeout in config:

```json
{
  "plugins": {
    "default_timeout_seconds": 60
  }
}
```

## Configuration Reference

### squid.config.json

```json
{
  "plugins": {
    "enabled": true,
    "load_global": true,
    "load_workspace": true,
    "load_bundled": true,
    "default_timeout_seconds": 30,
    "max_memory_mb": 128
  }
}
```

- **`enabled`**: Enable/disable plugin system
- **`load_global`**: Load plugins from `~/.squid/plugins/`
- **`load_workspace`**: Load plugins from `./plugins/`
- **`load_bundled`**: Load bundled plugins shipped with the executable (default: true)
- **`default_timeout_seconds`**: Maximum execution time
- **`max_memory_mb`**: Memory limit per plugin

### Environment Variables

Plugin configuration can be overridden via environment variables:

- **`SQUID_PLUGINS_LOAD_BUNDLED`**: Set to `false` to disable loading bundled plugins (e.g., `SQUID_PLUGINS_LOAD_BUNDLED=false`)

## Advanced Topics

### Async/Promise Limitations

**Important**: The current plugin system uses synchronous execution only. JavaScript `async`/`await` and Promises are **not supported** due to limitations in the QuickJS runtime integration (see [rquickjs#401](https://github.com/DelSkayn/rquickjs/issues/401)).

All plugin `execute` functions must be **synchronous**:

```javascript
// ✓ Correct: Synchronous
function execute(context, input) {
    const result = processData(input);
    return { result };
}

// ✗ Incorrect: Async not supported
async function execute(context, input) {
    const result = await fetchData();  // Will not work!
    return { result };
}
```

If your plugin needs async operations (HTTP requests, file I/O), use the context APIs which are implemented in Rust and handle async internally:

```javascript
function execute(context, input) {
    // Use context.httpGet() instead of fetch() - handled by Rust
    // Use context.readFile() instead of async file reads
    const content = context.readFile(input.path);  // Synchronous from JS perspective
    return { content };
}
```

**Future**: Async support may be added if the `rquickjs` event loop integration improves.

### Multiple Files

For complex plugins, split logic into modules (future):

```javascript
// Currently: single index.js file
// Future: support for require() or imports
```

### Plugin Marketplace

Future feature: Share plugins via a central marketplace.

## Getting Help

- **Issues**: https://github.com/DenysVuika/squid/issues
- **Discussions**: https://github.com/DenysVuika/squid/discussions
- **Examples**: See `plugins/` directory for working examples

## Contributing

We welcome plugin contributions! To share your plugin:

1. Test it thoroughly
2. Document it well
3. Submit a PR to add it to the examples
4. Consider publishing to the marketplace (coming soon)

---

Happy plugin building! 🦑🔌
