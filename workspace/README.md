# Squid Workspace

This is the default working directory for Squid AI operations. Place your code files here for analysis, or configure a different directory in your `squid.config.json`.

## Default Behavior

- **Local setup**: Files are accessed at `./workspace/` relative to where Squid is installed
- **Docker setup**: Files are mounted to `/workspace/` inside the container
- All file operations, code search, and plugin access are restricted to this directory
- Automatically created if it doesn't exist on startup

## Configuration

You can change the working directory in three ways:

### 1. Config File (squid.config.json)

```json
{
  "working_dir": "./my-project"
}
```

### 2. Environment Variable

```bash
export SQUID_WORKING_DIR=/path/to/your/project
squid serve
```

### 3. Docker Environment Variable

```bash
WORKSPACE_DIR=/path/to/your/project docker compose up
```

## Security

- Plugins cannot see the actual filesystem path
- All operations respect `.squidignore` patterns
- Paths outside the working directory are blocked
