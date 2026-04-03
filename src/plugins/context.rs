use crate::config::Config;
use crate::validate::PathValidator;
use log::{debug, info};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Context API provided to plugins for safe operations
pub struct PluginContext {
    path_validator: PathValidator,
    plugin_id: String,
    allow_network: bool,
    allow_file_write: bool,
    working_dir: PathBuf, // Private - plugins don't have access to this
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

        // Resolve working directory (absolute or relative to cwd)
        let working_dir = PathBuf::from(&config.working_dir);
        let working_dir = if working_dir.is_absolute() {
            working_dir
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&working_dir)
        };

        // Ensure working directory exists
        if !working_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&working_dir) {
                debug!("Warning: Failed to create working directory {:?}: {}", working_dir, e);
            }
        }

        Self {
            path_validator,
            plugin_id,
            allow_network,
            allow_file_write,
            working_dir,
        }
    }

    /// Resolve a path relative to the working directory
    fn resolve_path(&self, path: &str) -> PathBuf {
        let path_obj = Path::new(path);
        if path_obj.is_absolute() {
            path_obj.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }
    
    /// Read a file from the filesystem (respects .squidignore)
    pub fn read_file(&self, path: &str) -> Result<String, String> {
        debug!("Plugin '{}' reading file: {}", self.plugin_id, path);

        // Resolve path relative to working directory
        let path_obj = self.resolve_path(path);

        // Validate the path
        self.path_validator
            .validate(&path_obj)
            .map_err(|e| format!("Access denied: {}", e))?;

        // Read the file
        std::fs::read_to_string(&path_obj)
            .map_err(|e| format!("Failed to read file: {}", e))
    }
    
    /// Write content to a file (requires file_write permission)
    pub fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        if !self.allow_file_write {
            return Err("Plugin does not have file_write permission".to_string());
        }

        debug!("Plugin '{}' writing file: {}", self.plugin_id, path);

        // Resolve path relative to working directory
        let path_obj = self.resolve_path(path);

        // Validate the path
        self.path_validator
            .validate(&path_obj)
            .map_err(|e| format!("Access denied: {}", e))?;

        // Write the file
        std::fs::write(&path_obj, content)
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
    /// Returns "." to indicate plugins should use relative paths
    /// The actual working directory is managed internally by the context
    pub fn project_dir(&self) -> String {
        ".".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_read_file_permission() {
        // Use a config with working_dir set to current directory for tests
        let mut config = Config::load();
        config.working_dir = ".".to_string();
        let config = Arc::new(config);

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
        let mut config = Config::load();
        config.working_dir = ".".to_string();
        let config = Arc::new(config);

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

    #[test]
    fn test_project_dir_always_returns_dot() {
        // Security test: plugins should never see the actual filesystem path
        let mut config = Config::load();

        // Test with different working_dir values
        for working_dir in ["/absolute/path", "./relative/path", "workspace"] {
            config.working_dir = working_dir.to_string();
            let context = PluginContext::new(
                Arc::new(config.clone()),
                "test-plugin".to_string(),
                false,
                false,
            );

            // Should always return "." regardless of actual working_dir
            assert_eq!(context.project_dir(), ".");
        }
    }

    #[test]
    fn test_resolve_path_relative() {
        // Test that relative paths are resolved against working_dir
        let mut config = Config::load();
        config.working_dir = ".".to_string();
        let config = Arc::new(config);

        let context = PluginContext::new(
            config,
            "test-plugin".to_string(),
            false,
            false,
        );

        // Relative path should be joined with working_dir
        let resolved = context.resolve_path("src/main.rs");
        assert!(resolved.to_string_lossy().contains("src"));
    }

    #[test]
    fn test_resolve_path_absolute() {
        // Test that absolute paths are NOT joined with working_dir
        let mut config = Config::load();
        config.working_dir = "./workspace".to_string();
        let config = Arc::new(config);

        let context = PluginContext::new(
            config,
            "test-plugin".to_string(),
            false,
            false,
        );

        // Absolute path should be used as-is (will be validated by PathValidator)
        let absolute_path = if cfg!(windows) {
            "C:\\absolute\\path\\file.txt"
        } else {
            "/absolute/path/file.txt"
        };
        let resolved = context.resolve_path(absolute_path);
        assert_eq!(resolved.to_string_lossy(), absolute_path);
    }

    #[test]
    fn test_system_paths_blocked_by_validator() {
        // Security test: system paths should be blocked by PathValidator
        let mut config = Config::load();
        config.working_dir = ".".to_string();
        let config = Arc::new(config);

        let context = PluginContext::new(
            config,
            "test-plugin".to_string(),
            false,
            false,
        );

        // These system paths should be blocked by PathValidator
        let system_paths = if cfg!(windows) {
            vec![
                "C:\\Windows\\System32\\config\\SAM",
                "C:\\Program Files\\sensitive.dat",
            ]
        } else {
            vec![
                "/etc/passwd",
                "/etc/shadow",
                "/root/.ssh/id_rsa",
            ]
        };

        for path in system_paths {
            let result = context.read_file(path);
            // PathValidator should block these
            assert!(result.is_err(), "System path should be blocked: {}", path);
            let err = result.unwrap_err();
            // Should mention either access denied or permission
            assert!(
                err.contains("Access denied") || err.contains("Permission denied") || err.contains("Failed to read"),
                "Expected security error for {}, got: {}",
                path,
                err
            );
        }
    }

    #[test]
    fn test_working_dir_isolation() {
        // Security test: operations within working_dir should succeed,
        // operations outside should fail
        let mut config = Config::load();
        config.working_dir = ".".to_string();
        let config = Arc::new(config);

        let context = PluginContext::new(
            config,
            "test-plugin".to_string(),
            false,
            false,
        );

        // Reading a file in the current directory should work
        let result = context.read_file("Cargo.toml");
        assert!(result.is_ok(), "Should be able to read files in working_dir");

        // Trying to read from system directories should fail
        // (either path validation fails or file doesn't exist)
        let system_paths = if cfg!(windows) {
            vec!["C:\\Windows\\System32\\config\\SAM"]
        } else {
            vec!["/etc/shadow", "/root/.ssh/id_rsa"]
        };

        for path in system_paths {
            let result = context.read_file(path);
            assert!(result.is_err(), "Should not access system files: {}", path);
        }
    }

    #[tokio::test]
    async fn test_http_permission_required() {
        // Security test: network operations require explicit permission
        let mut config = Config::load();
        config.working_dir = ".".to_string();
        let config = Arc::new(config);

        let context_no_network = PluginContext::new(
            config.clone(),
            "test-plugin-no-net".to_string(),
            false, // No network permission
            false,
        );

        let context_with_network = PluginContext::new(
            config,
            "test-plugin-with-net".to_string(),
            true, // With network permission
            false,
        );

        // Should fail without permission
        let result = context_no_network.http_get("http://example.com", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("network permission"));

        // Should attempt with permission (may fail due to no network, but shouldn't fail on permission)
        let result = context_with_network.http_get("http://example.com", None).await;
        // We don't check if it succeeds (no network in tests), just that permission error doesn't occur
        if let Err(e) = result {
            assert!(!e.contains("network permission"), "Should not fail on permission check, got: {}", e);
        }
    }
}
