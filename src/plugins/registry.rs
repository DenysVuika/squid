use crate::plugins::metadata::PluginMetadata;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Plugin registry that manages plugin discovery and caching
pub struct PluginRegistry {
    /// Map of plugin ID to metadata
    plugins: Arc<RwLock<HashMap<String, PluginMetadata>>>,
    
    /// Plugin directories to search
    plugin_dirs: Vec<PathBuf>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_dirs: Vec::new(),
        }
    }
    
    /// Add a plugin directory to search
    pub fn add_directory(&mut self, dir: PathBuf) {
        if dir.exists() && dir.is_dir() {
            self.plugin_dirs.push(dir);
        }
    }
    
    /// Discover and load all plugins from configured directories
    pub fn discover_plugins(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut plugins_map = self.plugins.write().unwrap();
        plugins_map.clear();
        
        let mut total_loaded = 0;
        
        for plugin_dir in &self.plugin_dirs {
            debug!("Scanning plugin directory: {}", plugin_dir.display());
            
            // Check if directory exists
            if !plugin_dir.exists() {
                debug!("Plugin directory does not exist: {}", plugin_dir.display());
                continue;
            }
            
            // Iterate through subdirectories
            let entries = match std::fs::read_dir(plugin_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    warn!("Failed to read plugin directory {}: {}", plugin_dir.display(), e);
                    continue;
                }
            };
            
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Skip if not a directory
                if !path.is_dir() {
                    continue;
                }
                
                // Try to load plugin metadata
                match PluginMetadata::load(&path) {
                    Ok(mut metadata) => {
                        // Validate plugin structure
                        if let Err(e) = metadata.validate_structure() {
                            warn!(
                                "Plugin '{}' validation failed: {}",
                                metadata.id, e
                            );
                            continue;
                        }
                        
                        // Mark if this is a global plugin
                        metadata.is_global = self.is_global_dir(plugin_dir);
                        
                        // Check for ID conflicts (workspace plugins override global)
                        if let Some(existing) = plugins_map.get(&metadata.id)
                            && metadata.is_global && !existing.is_global {
                                // Don't override workspace plugin with global
                                debug!(
                                    "Skipping global plugin '{}' - overridden by workspace plugin",
                                    metadata.id
                                );
                                continue;
                            }
                        
                        info!(
                            "Loaded plugin: {} v{} from {}",
                            metadata.title,
                            metadata.version,
                            path.display()
                        );
                        
                        plugins_map.insert(metadata.id.clone(), metadata);
                        total_loaded += 1;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to load plugin from {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }
        
        info!("Discovered {} plugin(s)", total_loaded);
        
        Ok(total_loaded)
    }
    
    /// Get a plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<PluginMetadata> {
        self.plugins.read().unwrap().get(plugin_id).cloned()
    }
    
    /// Get all plugins
    pub fn get_all_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.read().unwrap().values().cloned().collect()
    }
    
    /// Check if a plugin exists
    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.plugins.read().unwrap().contains_key(plugin_id)
    }
    
    /// Get the number of registered plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.read().unwrap().len()
    }
    
    /// Check if a directory is a global plugin directory
    fn is_global_dir(&self, dir: &Path) -> bool {
        // Check if path contains .squid (typically ~/.squid/plugins)
        dir.to_string_lossy().contains(".squid")
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the global plugins directory path
pub fn get_global_plugins_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".squid").join("plugins"))
}

/// Get the workspace plugins directory path
pub fn get_workspace_plugins_dir() -> PathBuf {
    PathBuf::from("./plugins")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.plugin_count(), 0);
    }
    
    #[test]
    fn test_add_directory() {
        let mut registry = PluginRegistry::new();
        registry.add_directory(PathBuf::from("./test_plugins"));
        assert_eq!(registry.plugin_dirs.len(), 0); // Won't add if doesn't exist
    }
    
    #[test]
    fn test_workspace_dir() {
        let dir = get_workspace_plugins_dir();
        assert_eq!(dir, PathBuf::from("./plugins"));
    }
}
