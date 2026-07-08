//! Reusable JavaScript plugin runtime and discovery for squid-like hosts.
//!
//! This crate provides:
//! - plugin discovery from workspace/global/bundled directories,
//! - JSON schema validation for plugin IO,
//! - sandboxed QuickJS execution with a small host context API,
//! - a small global manager API for host applications.

pub mod context;
pub mod manager;
pub mod metadata;
mod path_policy;
pub mod registry;
pub mod runtime;
pub mod validator;

#[cfg(test)]
mod tests;

/// Re-exported manager APIs for convenient host integration.
pub use manager::{
    PluginManager, PluginSystemConfig, PluginToolDefinition, execute_plugin_tool,
    get_plugin_metadata, get_plugin_tools, initialize, is_plugin_tool, plugin_count,
    reload_plugins,
};
/// Re-exported plugin metadata type.
pub use metadata::PluginMetadata;
