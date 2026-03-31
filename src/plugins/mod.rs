//! Plugin system for extending squid with JavaScript-based tools
//! 
//! This module provides a QuickJS-based plugin system that allows users to create
//! custom tools that can be invoked by the LLM alongside built-in tools.

pub mod context;
pub mod manager;
pub mod metadata;
pub mod registry;
pub mod runtime;
pub mod validator;

#[cfg(test)]
mod tests;

pub use manager::PluginManager;
pub use metadata::PluginMetadata;

use crate::config::Config;
use async_openai::types::chat::ChatCompletionTools;
use log::{debug, warn};
use serde_json::Value;
use std::sync::{Arc, OnceLock};

/// Global plugin manager instance
static PLUGIN_MANAGER: OnceLock<Arc<PluginManager>> = OnceLock::new();

/// Initialize the plugin system
pub fn initialize(config: Arc<Config>) -> Result<(), Box<dyn std::error::Error>> {
    // Check if plugins are enabled
    if !config.plugins.enabled {
        debug!("Plugin system is disabled in config");
        return Ok(());
    }
    
    let manager = PluginManager::new(config);
    manager.initialize()?;
    
    PLUGIN_MANAGER.set(Arc::new(manager)).map_err(|_| {
        "Plugin manager already initialized"
    })?;
    
    Ok(())
}

/// Get the global plugin manager
fn get_manager() -> Option<Arc<PluginManager>> {
    PLUGIN_MANAGER.get().cloned()
}

/// Get all plugin tools for the LLM
pub fn get_plugin_tools() -> Result<Vec<ChatCompletionTools>, Box<dyn std::error::Error>> {
    match get_manager() {
        Some(manager) => manager.get_plugin_tools(),
        None => {
            warn!("Plugin manager not initialized");
            Ok(Vec::new())
        }
    }
}

/// Check if a tool name corresponds to a plugin
pub fn is_plugin_tool(tool_name: &str) -> bool {
    match get_manager() {
        Some(manager) => manager.is_plugin_tool(tool_name),
        None => false,
    }
}

/// Get plugin metadata by tool name
pub fn get_plugin_metadata(tool_name: &str) -> Option<PluginMetadata> {
    get_manager().and_then(|manager| manager.get_plugin_metadata(tool_name))
}

/// Execute a plugin tool
pub async fn execute_plugin_tool(
    tool_name: &str,
    input: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    match get_manager() {
        Some(manager) => manager.execute_plugin_tool(tool_name, input).await,
        None => Err("Plugin manager not initialized".into()),
    }
}

/// Get the number of loaded plugins
pub fn plugin_count() -> usize {
    get_manager().map(|m| m.plugin_count()).unwrap_or(0)
}

/// Reload all plugins (useful for development)
pub fn reload_plugins() -> Result<usize, Box<dyn std::error::Error>> {
    match get_manager() {
        Some(manager) => manager.reload(),
        None => Err("Plugin manager not initialized".into()),
    }
}
