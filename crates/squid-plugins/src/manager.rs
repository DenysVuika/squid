use crate::context::PluginContext;
use crate::metadata::PluginMetadata;
use crate::registry::{PluginRegistry, get_global_plugins_dir, get_workspace_plugins_dir};
use crate::runtime::PluginRuntime;
use crate::validator::SchemaValidator;
use log::{debug, warn};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

/// Host-facing configuration for the plugin system.
#[derive(Debug, Clone)]
pub struct PluginSystemConfig {
    /// Enables or disables plugin initialization entirely.
    pub enabled: bool,
    /// Loads plugins from `~/.squid/plugins` when true.
    pub load_global: bool,
    /// Loads plugins from `./plugins` when true.
    pub load_workspace: bool,
    /// Loads plugins from `bundled_plugins_dir` when true.
    pub load_bundled: bool,
    /// Working directory used for relative plugin file operations.
    pub working_dir: PathBuf,
    /// Default execution timeout for one plugin call.
    pub default_timeout_seconds: u64,
    /// Per-plugin QuickJS memory cap in MB.
    pub max_memory_mb: usize,
    /// Optional path where bundled plugins are extracted.
    pub bundled_plugins_dir: Option<PathBuf>,
    /// Ignore filename used for path restrictions (for example `.squidignore`).
    pub ignore_file_name: String,
}

impl Default for PluginSystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            load_global: true,
            load_workspace: true,
            load_bundled: true,
            working_dir: PathBuf::from("."),
            default_timeout_seconds: 30,
            max_memory_mb: 128,
            bundled_plugins_dir: None,
            ignore_file_name: ".squidignore".to_string(),
        }
    }
}

/// Provider-agnostic tool definition returned by plugin discovery.
#[derive(Debug, Clone)]
pub struct PluginToolDefinition {
    /// Tool name exposed to the model (for example `plugin:my-plugin`).
    pub name: String,
    /// Human-readable tool description.
    pub description: String,
    /// JSON schema for tool input.
    pub parameters: Value,
}

/// Stateful manager for plugin discovery, metadata lookup, and execution.
pub struct PluginManager {
    registry: Arc<RwLock<PluginRegistry>>,
    config: Arc<PluginSystemConfig>,
}

impl PluginManager {
    /// Creates a new plugin manager.
    pub fn new(config: Arc<PluginSystemConfig>) -> Self {
        Self {
            registry: Arc::new(RwLock::new(PluginRegistry::new())),
            config,
        }
    }

    /// Discovers plugins from configured directories.
    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = self.registry.write().unwrap();

        if self.config.load_global
            && let Some(global_dir) = get_global_plugins_dir()
        {
            debug!("Adding global plugin directory: {}", global_dir.display());
            registry.add_global_directory(global_dir);
        }

        if self.config.load_workspace {
            let workspace_dir = get_workspace_plugins_dir();
            debug!(
                "Adding workspace plugin directory: {}",
                workspace_dir.display()
            );
            registry.add_workspace_directory(workspace_dir);
        }

        if self.config.load_bundled
            && let Some(bundled_dir) = &self.config.bundled_plugins_dir
        {
            debug!("Adding bundled plugin directory: {}", bundled_dir.display());
            registry.add_bundled_directory(bundled_dir.clone());
        }

        registry.discover_plugins()?;
        Ok(())
    }

    /// Returns all discovered plugins as generic tool definitions.
    pub fn get_plugin_tools(
        &self,
    ) -> Result<Vec<PluginToolDefinition>, Box<dyn std::error::Error>> {
        let registry = self.registry.read().unwrap();
        let plugins = registry.get_all_plugins();

        Ok(plugins
            .into_iter()
            .map(|plugin| PluginToolDefinition {
                name: plugin.tool_name(),
                description: plugin.description,
                parameters: plugin.input_schema,
            })
            .collect())
    }

    /// Returns true when a tool name points to a discovered plugin.
    pub fn is_plugin_tool(&self, tool_name: &str) -> bool {
        if !tool_name.starts_with("plugin:") {
            return false;
        }

        let plugin_id = &tool_name[7..];
        let registry = self.registry.read().unwrap();
        registry.has_plugin(plugin_id)
    }

    /// Gets plugin metadata by tool name.
    pub fn get_plugin_metadata(&self, tool_name: &str) -> Option<PluginMetadata> {
        if !tool_name.starts_with("plugin:") {
            return None;
        }

        let plugin_id = &tool_name[7..];
        let registry = self.registry.read().unwrap();
        registry.get_plugin(plugin_id)
    }

    /// Executes a plugin after validating input and output schemas.
    pub async fn execute_plugin_tool(
        &self,
        tool_name: &str,
        input: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let plugin_id = tool_name
            .strip_prefix("plugin:")
            .ok_or("Invalid plugin tool name")?;

        let metadata = {
            let registry = self.registry.read().unwrap();
            registry
                .get_plugin(plugin_id)
                .ok_or(format!("Plugin '{}' not found", plugin_id))?
        };

        let input_validator = SchemaValidator::new(&metadata.input_schema)?;
        if let Err(errors) = input_validator.validate(input) {
            return Ok(json!({
                "error": "Input validation failed",
                "details": errors
            }));
        }

        let metadata_clone = metadata.clone();
        let input_clone = input.clone();
        let config_clone = Arc::clone(&self.config);
        let timeout_duration = std::time::Duration::from_secs(config_clone.default_timeout_seconds);

        let task = tokio::task::spawn_blocking(move || -> Result<Value, String> {
            let js_code = std::fs::read_to_string(metadata_clone.index_js_path())
                .map_err(|e| format!("Failed to read plugin code: {}", e))?;

            let mut runtime = PluginRuntime::with_memory_limit(config_clone.max_memory_mb)
                .map_err(|e| format!("Failed to create runtime: {}", e))?;

            runtime
                .load_plugin(&js_code)
                .map_err(|e| format!("Failed to load plugin: {}", e))?;

            let context = Arc::new(PluginContext::new(
                config_clone,
                metadata_clone.id.clone(),
                metadata_clone.security.network,
                metadata_clone.security.file_write,
            ));
            runtime.set_context(context);

            runtime
                .execute_sync(input_clone)
                .map_err(|e| format!("Plugin execution failed: {}", e))
        });

        let result = tokio::time::timeout(timeout_duration, task)
            .await
            .map_err(|_| {
                format!(
                    "Plugin '{}' exceeded timeout of {}s",
                    plugin_id,
                    timeout_duration.as_secs()
                )
            })?
            .map_err(|e| format!("Plugin task panicked: {}", e))
            .and_then(|r| r)?;

        let output_validator = SchemaValidator::new(&metadata.output_schema)?;
        if let Err(errors) = output_validator.validate(&result) {
            warn!(
                "Plugin '{}' output validation failed: {:?}",
                metadata.id, errors
            );
            return Ok(json!({
                "error": "Output validation failed",
                "details": errors,
                "raw_output": result
            }));
        }

        Ok(result)
    }

    /// Returns discovered plugin count.
    pub fn plugin_count(&self) -> usize {
        self.registry.read().unwrap().plugin_count()
    }

    /// Re-runs discovery and returns newly loaded plugin count.
    pub fn reload(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut registry = self.registry.write().unwrap();
        registry.discover_plugins()
    }
}

static PLUGIN_MANAGER: OnceLock<Arc<PluginManager>> = OnceLock::new();

/// Initializes the global plugin manager instance.
pub fn initialize(config: Arc<PluginSystemConfig>) -> Result<(), Box<dyn std::error::Error>> {
    if !config.enabled {
        debug!("Plugin system is disabled in config");
        return Ok(());
    }

    let manager = PluginManager::new(config);
    manager.initialize()?;

    PLUGIN_MANAGER
        .set(Arc::new(manager))
        .map_err(|_| "Plugin manager already initialized")?;

    Ok(())
}

fn get_manager() -> Option<Arc<PluginManager>> {
    PLUGIN_MANAGER.get().cloned()
}

/// Returns all discovered plugin tools from the global manager.
pub fn get_plugin_tools() -> Result<Vec<PluginToolDefinition>, Box<dyn std::error::Error>> {
    match get_manager() {
        Some(manager) => manager.get_plugin_tools(),
        None => Ok(Vec::new()),
    }
}

/// Returns true when the given name maps to a discovered plugin.
pub fn is_plugin_tool(tool_name: &str) -> bool {
    match get_manager() {
        Some(manager) => manager.is_plugin_tool(tool_name),
        None => false,
    }
}

/// Returns plugin metadata by tool name.
pub fn get_plugin_metadata(tool_name: &str) -> Option<PluginMetadata> {
    get_manager().and_then(|manager| manager.get_plugin_metadata(tool_name))
}

/// Executes a plugin via the global manager.
pub async fn execute_plugin_tool(
    tool_name: &str,
    input: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    match get_manager() {
        Some(manager) => manager.execute_plugin_tool(tool_name, input).await,
        None => Err("Plugin manager not initialized".into()),
    }
}

/// Returns discovered plugin count from the global manager.
pub fn plugin_count() -> usize {
    get_manager().map(|m| m.plugin_count()).unwrap_or(0)
}

/// Reloads plugin discovery via the global manager.
pub fn reload_plugins() -> Result<usize, Box<dyn std::error::Error>> {
    match get_manager() {
        Some(manager) => manager.reload(),
        None => Err("Plugin manager not initialized".into()),
    }
}
