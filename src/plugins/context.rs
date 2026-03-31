use crate::config::Config;
use crate::validate::PathValidator;
use log::{debug, info};
use std::path::Path;
use std::sync::Arc;

/// Context API provided to plugins for safe operations
pub struct PluginContext {
    config: Arc<Config>,
    path_validator: PathValidator,
    plugin_id: String,
    allow_network: bool,
    allow_file_write: bool,
}

impl PluginContext {
    /// Create a new plugin context with specified permissions
    pub fn new(
        config: Arc<Config>,
        plugin_id: String,
        allow_network: bool,
        allow_file_write: bool,
    ) -> Self {
        let ignore_patterns = PathValidator::load_ignore_patterns();
        let path_validator = PathValidator::with_ignore_file(
            if ignore_patterns.is_empty() {
                None
            } else {
                Some(ignore_patterns)
            }
        );
        
        Self {
            config,
            path_validator,
            plugin_id,
            allow_network,
            allow_file_write,
        }
    }
    
    /// Read a file from the filesystem (respects .squidignore)
    pub fn read_file(&self, path: &str) -> Result<String, String> {
        debug!("Plugin '{}' reading file: {}", self.plugin_id, path);
        
        // Validate the path
        let path_obj = Path::new(path);
        self.path_validator
            .validate(path_obj)
            .map_err(|e| format!("Access denied: {}", e))?;
        
        // Read the file
        std::fs::read_to_string(path_obj)
            .map_err(|e| format!("Failed to read file: {}", e))
    }
    
    /// Write content to a file (requires file_write permission)
    pub fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        if !self.allow_file_write {
            return Err("Plugin does not have file_write permission".to_string());
        }
        
        debug!("Plugin '{}' writing file: {}", self.plugin_id, path);
        
        // Validate the path
        let path_obj = Path::new(path);
        self.path_validator
            .validate(path_obj)
            .map_err(|e| format!("Access denied: {}", e))?;
        
        // Write the file
        std::fs::write(path_obj, content)
            .map_err(|e| format!("Failed to write file: {}", e))
    }
    
    /// Make an HTTP GET request (requires network permission)
    pub async fn http_get(&self, url: &str, timeout_ms: Option<u64>) -> Result<String, String> {
        if !self.allow_network {
            return Err("Plugin does not have network permission".to_string());
        }
        
        info!("Plugin '{}' making HTTP GET request to: {}", self.plugin_id, url);
        
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
            return Err(format!("HTTP request failed with status: {}", response.status()));
        }
        
        response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))
    }
    
    /// Log a message from the plugin
    pub fn log(&self, message: &str) {
        info!("[Plugin:{}] {}", self.plugin_id, message);
    }
    
    /// Get the current project directory
    pub fn project_dir(&self) -> String {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| ".".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_read_file_permission() {
        let config = Arc::new(Config::load());
        let context = PluginContext::new(
            config,
            "test-plugin".to_string(),
            false,
            false,
        );
        
        // Try to read Cargo.toml (should work as it's a valid project file)
        let result = context.read_file("Cargo.toml");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_write_file_permission_denied() {
        let config = Arc::new(Config::load());
        let context = PluginContext::new(
            config,
            "test-plugin".to_string(),
            false,
            false, // No write permission
        );
        
        let result = context.write_file("test.txt", "content");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("file_write permission"));
    }
    
    #[test]
    fn test_project_dir() {
        let config = Arc::new(Config::load());
        let context = PluginContext::new(
            config,
            "test-plugin".to_string(),
            false,
            false,
        );
        
        let dir = context.project_dir();
        assert!(!dir.is_empty());
    }
}
