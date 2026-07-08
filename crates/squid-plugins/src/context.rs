use crate::manager::PluginSystemConfig;
use crate::path_policy::PathValidator;
use log::{debug, info};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Host context exposed to plugins for safe IO and logging.
pub struct PluginContext {
    path_validator: PathValidator,
    plugin_id: String,
    allow_network: bool,
    allow_file_write: bool,
    working_dir: PathBuf,
}

impl PluginContext {
    /// Creates a context instance for one plugin execution.
    pub fn new(
        config: Arc<PluginSystemConfig>,
        plugin_id: String,
        allow_network: bool,
        allow_file_write: bool,
    ) -> Self {
        let ignore_patterns = PathValidator::load_ignore_patterns(&config.ignore_file_name);
        let path_validator = PathValidator::with_ignore_patterns(ignore_patterns);

        let working_dir = if config.working_dir.is_absolute() {
            config.working_dir.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&config.working_dir)
        };

        if !working_dir.exists()
            && let Err(e) = std::fs::create_dir_all(&working_dir)
        {
            debug!(
                "Warning: Failed to create working directory {:?}: {}",
                working_dir, e
            );
        }

        Self {
            path_validator,
            plugin_id,
            allow_network,
            allow_file_write,
            working_dir,
        }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let path_obj = Path::new(path);
        if path_obj.is_absolute() {
            path_obj.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }

    /// Reads a text file after path policy validation.
    pub fn read_file(&self, path: &str) -> Result<String, String> {
        debug!("Plugin '{}' reading file: {}", self.plugin_id, path);

        let path_obj = self.resolve_path(path);

        self.path_validator
            .validate(&path_obj)
            .map_err(|e| format!("Access denied: {}", e))?;

        std::fs::read_to_string(&path_obj).map_err(|e| format!("Failed to read file: {}", e))
    }

    /// Writes a text file when the plugin has `file_write` permission.
    pub fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        if !self.allow_file_write {
            return Err("Plugin does not have file_write permission".to_string());
        }

        debug!("Plugin '{}' writing file: {}", self.plugin_id, path);

        let path_obj = self.resolve_path(path);

        self.path_validator
            .validate(&path_obj)
            .map_err(|e| format!("Access denied: {}", e))?;

        std::fs::write(&path_obj, content).map_err(|e| format!("Failed to write file: {}", e))
    }

    /// Performs a simple HTTP GET when the plugin has `network` permission.
    pub async fn http_get(&self, url: &str, timeout_ms: Option<u64>) -> Result<String, String> {
        if !self.allow_network {
            return Err("Plugin does not have network permission".to_string());
        }

        info!(
            "Plugin '{}' making HTTP GET request to: {}",
            self.plugin_id, url
        );

        let timeout = std::time::Duration::from_millis(timeout_ms.unwrap_or(5000));

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "HTTP request failed with status: {}",
                response.status()
            ));
        }

        response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))
    }

    /// Emits a host log message on behalf of the plugin.
    pub fn log(&self, message: &str) {
        info!("[Plugin:{}] {}", self.plugin_id, message);
    }

    /// Returns a redacted project root marker for plugin scripts.
    pub fn project_dir(&self) -> String {
        ".".to_string()
    }
}
