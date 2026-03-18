# Squid Workspace

Place your code files here for analysis with Squid.

## Usage

Files in this directory are accessible to Squid at `/data/workspace/` inside the container.

Example:
```bash
docker compose exec squid /app/squid review --file /data/workspace/myproject/src/main.rs
```
