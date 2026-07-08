use crate::config::Config;
use async_openai::types::chat::ChatCompletionTools;
use async_openai::types::chat::{ChatCompletionTool, FunctionObjectArgs};
use log::warn;
use serde_json::Value;
use std::sync::Arc;

pub use squid_plugins::PluginMetadata;
pub use squid_plugins::PluginSystemConfig;

fn to_plugin_system_config(config: Arc<Config>) -> PluginSystemConfig {
    PluginSystemConfig {
        enabled: config.plugins.enabled,
        load_global: config.plugins.load_global,
        load_workspace: config.plugins.load_workspace,
        load_bundled: config.plugins.load_bundled,
        working_dir: std::path::PathBuf::from(&config.working_dir),
        default_timeout_seconds: config.plugins.default_timeout_seconds,
        max_memory_mb: config.plugins.max_memory_mb,
        bundled_plugins_dir: crate::bundled::get_bundled_plugins_dir(),
        ignore_file_name: ".squidignore".to_string(),
    }
}

pub fn initialize(config: Arc<Config>) -> Result<(), Box<dyn std::error::Error>> {
    squid_plugins::initialize(Arc::new(to_plugin_system_config(config)))
}

pub fn get_plugin_tools() -> Result<Vec<ChatCompletionTools>, Box<dyn std::error::Error>> {
    let plugin_tools = squid_plugins::get_plugin_tools()?;

    let mut tools = Vec::with_capacity(plugin_tools.len());
    for plugin in plugin_tools {
        let tool = ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name(plugin.name)
                .description(plugin.description)
                .parameters(plugin.parameters)
                .build()?,
        });
        tools.push(tool);
    }

    Ok(tools)
}

pub fn is_plugin_tool(tool_name: &str) -> bool {
    squid_plugins::is_plugin_tool(tool_name)
}

pub fn get_plugin_metadata(tool_name: &str) -> Option<PluginMetadata> {
    squid_plugins::get_plugin_metadata(tool_name)
}

pub async fn execute_plugin_tool(
    tool_name: &str,
    input: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    squid_plugins::execute_plugin_tool(tool_name, input).await
}

pub fn plugin_count() -> usize {
    squid_plugins::plugin_count()
}

pub fn reload_plugins() -> Result<usize, Box<dyn std::error::Error>> {
    let result = squid_plugins::reload_plugins();
    if result.is_err() {
        warn!("Plugin manager not initialized");
    }
    result
}
