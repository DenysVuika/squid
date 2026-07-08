# squid-plugins

Reusable JavaScript plugin runtime and discovery crate used by Squid.

It provides:

- Plugin discovery from workspace, global, and optional bundled directories
- JSON schema validation for plugin input/output
- Sandboxed QuickJS execution with memory and timeout controls
- A provider-agnostic tool definition model for host applications

## Installation

Path dependency (inside a workspace):

```toml
[dependencies]
squid-plugins = { path = "crates/squid-plugins" }
```

## Quick Usage

```rust
use squid_plugins::{
    initialize, get_plugin_tools, execute_plugin_tool, PluginSystemConfig,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = PluginSystemConfig {
        enabled: true,
        load_global: true,
        load_workspace: true,
        load_bundled: false,
        working_dir: ".".into(),
        default_timeout_seconds: 30,
        max_memory_mb: 128,
        bundled_plugins_dir: None,
        ignore_file_name: ".squidignore".to_string(),
    };

    initialize(Arc::new(cfg))?;

    let tools = get_plugin_tools()?;
    println!("Loaded {} plugin tools", tools.len());

    let result = execute_plugin_tool(
        "plugin:my-plugin",
        &json!({ "message": "hello" }),
    ).await?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

## Plugin Layout

Each plugin lives in a directory with these files:

- plugin.json
- index.js

Example plugin.json fields:

- id
- title
- description
- version
- api_version (currently 1.0)
- security
- input_schema
- output_schema

## Host Integration Notes

- This crate returns generic PluginToolDefinition values. Hosts can map these to their model provider tool format.
- Security capability mapping (for example, interpreting security.requires) is controlled by the host application.
- The global manager is process-wide and initialized once.

## Safety Model

- QuickJS runtime uses memory and stack limits.
- Plugin execution is bounded by host timeout.
- Context APIs enforce path validation and optional network/file-write permissions.
- Paths are checked against a blacklist, project whitelist, and ignore patterns file.

## License

Apache-2.0. See LICENSE.
