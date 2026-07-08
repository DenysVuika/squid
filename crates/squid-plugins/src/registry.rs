use crate::metadata::PluginMetadata;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Thread-safe plugin metadata registry with source precedence rules.
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, PluginMetadata>>>,
    plugin_dirs: Vec<PathBuf>,
    global_dirs: Vec<PathBuf>,
    bundled_dirs: Vec<PathBuf>,
}

impl PluginRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_dirs: Vec::new(),
            global_dirs: Vec::new(),
            bundled_dirs: Vec::new(),
        }
    }

    /// Adds a global plugin directory.
    pub fn add_global_directory(&mut self, dir: PathBuf) {
        if dir.exists() && dir.is_dir() {
            self.plugin_dirs.push(dir.clone());
            self.global_dirs.push(dir);
        }
    }

    /// Adds a workspace plugin directory.
    pub fn add_workspace_directory(&mut self, dir: PathBuf) {
        if dir.exists() && dir.is_dir() {
            self.plugin_dirs.push(dir);
        }
    }

    /// Adds a bundled plugin directory.
    pub fn add_bundled_directory(&mut self, dir: PathBuf) {
        if dir.exists() && dir.is_dir() {
            self.plugin_dirs.push(dir.clone());
            self.bundled_dirs.push(dir);
        }
    }

    /// Discovers plugins and applies precedence (workspace > global > bundled).
    pub fn discover_plugins(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut plugins_map = self.plugins.write().unwrap();
        plugins_map.clear();

        let mut total_loaded = 0;

        for plugin_dir in &self.plugin_dirs {
            debug!("Scanning plugin directory: {}", plugin_dir.display());

            if !plugin_dir.exists() {
                continue;
            }

            let entries = match std::fs::read_dir(plugin_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    warn!(
                        "Failed to read plugin directory {}: {}",
                        plugin_dir.display(),
                        e
                    );
                    continue;
                }
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                match PluginMetadata::load(&path) {
                    Ok(mut metadata) => {
                        if let Err(e) = metadata.validate_structure() {
                            warn!("Plugin '{}' validation failed: {}", metadata.id, e);
                            continue;
                        }

                        metadata.is_global = self.is_global_dir(plugin_dir);
                        let is_bundled = self.is_bundled_dir(plugin_dir);

                        if let Some(existing) = plugins_map.get(&metadata.id) {
                            if !metadata.is_global && !is_bundled {
                            } else if metadata.is_global && !existing.is_global {
                                continue;
                            } else if is_bundled && (!existing.is_global || metadata.is_global) {
                                continue;
                            }
                        }

                        let source = if metadata.is_global {
                            "global"
                        } else if is_bundled {
                            "bundled"
                        } else {
                            "workspace"
                        };

                        info!(
                            "Loaded plugin: {} v{} from {} ({})",
                            metadata.title,
                            metadata.version,
                            path.display(),
                            source
                        );

                        plugins_map.insert(metadata.id.clone(), metadata);
                        total_loaded += 1;
                    }
                    Err(e) => {
                        warn!("Failed to load plugin from {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(total_loaded)
    }

    /// Looks up one plugin by ID.
    pub fn get_plugin(&self, plugin_id: &str) -> Option<PluginMetadata> {
        self.plugins.read().unwrap().get(plugin_id).cloned()
    }

    /// Returns all discovered plugins.
    pub fn get_all_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.read().unwrap().values().cloned().collect()
    }

    /// Checks whether a plugin ID exists.
    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.plugins.read().unwrap().contains_key(plugin_id)
    }

    /// Returns discovered plugin count.
    pub fn plugin_count(&self) -> usize {
        self.plugins.read().unwrap().len()
    }

    fn is_global_dir(&self, dir: &std::path::Path) -> bool {
        self.global_dirs.iter().any(|global| dir == global)
    }

    fn is_bundled_dir(&self, dir: &std::path::Path) -> bool {
        self.bundled_dirs.iter().any(|bundled| dir == bundled)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_global_plugins_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".squid").join("plugins"))
}

/// Returns the default workspace plugin directory.
pub fn get_workspace_plugins_dir() -> PathBuf {
    PathBuf::from("./plugins")
}
