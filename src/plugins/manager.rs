use crate::config::Config;
use crate::plugins::context::PluginContext;
use crate::plugins::metadata::PluginMetadata;
use crate::plugins::registry::{get_global_plugins_dir, get_workspace_plugins_dir, PluginRegistry};
use crate::plugins::runtime::PluginRuntime;
use crate::plugins::validator::SchemaValidator;
use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use log::{debug, warn};
use serde_json::{json, Value};
use std::sync::{Arc, RwLock};

/// Plugin manager that handles plugin lifecycle
pub struct PluginManager {
    registry: Arc<RwLock<PluginRegistry>>,
    config: Arc<Config>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            registry: Arc::new(RwLock::new(PluginRegistry::new())),
            config,
        }
    }
    
    /// Initialize the plugin system
    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = self.registry.write().unwrap();
        
        // Add plugin directories based on config
        // 1. Global plugins
        if self.config.plugins.load_global
            && let Some(global_dir) = get_global_plugins_dir() {
                debug!("Adding global plugin directory: {}", global_dir.display());
                registry.add_directory(global_dir);
            }
        
        // 2. Workspace plugins
        if self.config.plugins.load_workspace {
            let workspace_dir = get_workspace_plugins_dir();
            debug!("Adding workspace plugin directory: {}", workspace_dir.display());
            registry.add_directory(workspace_dir);
        }
        
        // Discover all plugins
        let count = registry.discover_plugins()?;
        debug!("Discovered {} plugin(s)", count);
        
        Ok(())
    }
    
    /// Get all plugins as LLM tools
    pub fn get_plugin_tools(&self) -> Result<Vec<ChatCompletionTools>, Box<dyn std::error::Error>> {
        let registry = self.registry.read().unwrap();
        let plugins = registry.get_all_plugins();
        
        let mut tools = Vec::new();
        
        for plugin in plugins {
            let tool = ChatCompletionTools::Function(ChatCompletionTool {
                function: FunctionObjectArgs::default()
                    .name(plugin.tool_name())
                    .description(&plugin.description)
                    .parameters(plugin.input_schema.clone())
                    .build()?,
            });
            
            tools.push(tool);
        }
        
        Ok(tools)
    }
    
    /// Check if a tool name corresponds to a plugin
    pub fn is_plugin_tool(&self, tool_name: &str) -> bool {
        if !tool_name.starts_with("plugin:") {
            return false;
        }
        
        let plugin_id = &tool_name[7..]; // Remove "plugin:" prefix
        let registry = self.registry.read().unwrap();
        registry.has_plugin(plugin_id)
    }
    
    /// Get plugin metadata by tool name
    pub fn get_plugin_metadata(&self, tool_name: &str) -> Option<PluginMetadata> {
        if !tool_name.starts_with("plugin:") {
            return None;
        }
        
        let plugin_id = &tool_name[7..];
        let registry = self.registry.read().unwrap();
        registry.get_plugin(plugin_id)
    }
    
    /// Execute a plugin tool
    pub async fn execute_plugin_tool(
        &self,
        tool_name: &str,
        input: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        // Extract plugin ID
        let plugin_id = tool_name
            .strip_prefix("plugin:")
            .ok_or("Invalid plugin tool name")?;
        
        // Get plugin metadata
        let metadata = {
            let registry = self.registry.read().unwrap();
            registry
                .get_plugin(plugin_id)
                .ok_or(format!("Plugin '{}' not found", plugin_id))?
        };
        
        debug!("Executing plugin: {} v{}", metadata.title, metadata.version);
        
        // Validate input against schema
        let input_validator = SchemaValidator::new(&metadata.input_schema)?;
        if let Err(errors) = input_validator.validate(input) {
            return Ok(json!({
                "error": "Input validation failed",
                "details": errors
            }));
        }
        
        // Execute plugin in a synchronous blocking task to avoid Send/Sync issues
        // QuickJS Runtime and Context are not Send, so they must stay on a single thread
        let metadata_clone = metadata.clone();
        let input_clone = input.clone();
        let config_clone = Arc::clone(&self.config);
        let timeout_duration = std::time::Duration::from_secs(config_clone.plugins.default_timeout_seconds);
        
        let task = tokio::task::spawn_blocking(move || -> Result<Value, String> {
            // Load plugin code
            let js_code = std::fs::read_to_string(metadata_clone.index_js_path())
                .map_err(|e| format!("Failed to read plugin code: {}", e))?;
            
            // Create runtime with configured memory limit
            let memory_mb = config_clone.plugins.max_memory_mb;
            let mut runtime = PluginRuntime::with_memory_limit(memory_mb)
                .map_err(|e| format!("Failed to create runtime: {}", e))?;
            
            runtime.load_plugin(&js_code)
                .map_err(|e| format!("Failed to load plugin: {}", e))?;
            
            // Create and set plugin context
            let context = Arc::new(PluginContext::new(
                config_clone,
                metadata_clone.id.clone(),
                metadata_clone.security.network,
                metadata_clone.security.file_write,
            ));
            runtime.set_context(context);
            
            // Execute plugin (synchronous)
            runtime.execute_sync(input_clone)
                .map_err(|e| format!("Plugin execution failed: {}", e))
        });
        
        let result = tokio::time::timeout(timeout_duration, task)
            .await
            .map_err(|_| format!("Plugin '{}' exceeded timeout of {}s", plugin_id, timeout_duration.as_secs()))?
            .map_err(|e| format!("Plugin task panicked: {}", e))
            .and_then(|r| r)?;
        
        // Validate output against schema
        let output_validator = SchemaValidator::new(&metadata.output_schema)?;
        if let Err(errors) = output_validator.validate(&result) {
            warn!(
                "Plugin '{}' output validation failed: {:?}",
                metadata.id, errors
            );
            warn!(
                "Plugin '{}' returned: {}",
                metadata.id, serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
            );
            return Ok(json!({
                "error": "Output validation failed",
                "details": errors,
                "raw_output": result
            }));
        }
        
        Ok(result)
    }
    
    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        self.registry.read().unwrap().plugin_count()
    }
    
    /// Reload all plugins (useful for development)
    pub fn reload(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // Rediscover plugins
        let mut registry = self.registry.write().unwrap();
        registry.discover_plugins()
    }
}
