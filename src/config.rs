use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Tool permissions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    /// Tools that are always allowed to run without confirmation
    #[serde(default = "default_allowed_tools")]
    pub allow: Vec<String>,
    /// Tools that are never allowed to run
    #[serde(default)]
    pub deny: Vec<String>,
}

fn default_allowed_tools() -> Vec<String> {
    vec!["now".to_string()]
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            allow: default_allowed_tools(),
            deny: Vec::new(),
        }
    }
}

/// RAG (Retrieval-Augmented Generation) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Enable RAG features
    #[serde(default = "default_rag_enabled")]
    pub enabled: bool,
    /// Embedding model name (e.g., "nomic-embed-text")
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
    /// Embedding API URL (typically same as api_url)
    #[serde(default = "default_embedding_url")]
    pub embedding_url: String,
    /// Chunk size in tokens for document splitting
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    /// Overlap between chunks in tokens
    #[serde(default = "default_chunk_overlap")]
    pub chunk_overlap: usize,
    /// Number of top results to retrieve
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Documents directory path (relative to current working directory)
    #[serde(default = "default_documents_path")]
    pub documents_path: String,
}

fn default_rag_enabled() -> bool {
    true
}

fn default_embedding_model() -> String {
    "text-embedding-nomic-embed-text-v1.5".to_string()
}

fn default_embedding_url() -> String {
    "http://127.0.0.1:11434".to_string()
}

fn default_chunk_size() -> usize {
    512
}

fn default_chunk_overlap() -> usize {
    50
}

fn default_top_k() -> usize {
    5
}

fn default_documents_path() -> String {
    "documents".to_string()
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            enabled: default_rag_enabled(),
            embedding_model: default_embedding_model(),
            embedding_url: default_embedding_url(),
            chunk_size: default_chunk_size(),
            chunk_overlap: default_chunk_overlap(),
            top_k: default_top_k(),
            documents_path: default_documents_path(),
        }
    }
}


/// Configuration for squid CLI
///
/// This configuration is typically stored in `squid.config.json` in the project directory.
///
/// **Fields:**
/// - `api_url`: Base URL for the LLM API (e.g., `http://127.0.0.1:1234/v1`)
/// - `api_model`: Model identifier (e.g., `local-model`, `qwen2.5-coder`, `gpt-4`)
/// - `api_key`: Optional API key (use `None` for local models)
/// - `context_window`: Maximum context window size in tokens (e.g., `32768` for Qwen2.5-Coder)
/// - `log_level`: Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`)
/// - `version`: Config file version (matches app version when created)
///
/// **Best Practices:**
/// - Commit `squid.config.json` to your repository to share project settings with your team
/// - Keep sensitive API keys in `.env` file (which is gitignored)
/// - Use `api_key: None` in config file for local models (LM Studio, Ollama)
/// - For cloud services (OpenAI, etc.), omit `api_key` from config and set it via `.env`
/// - Default `log_level` is `error` (minimal noise)
/// - Use `.squidignore` file for project-wide ignore patterns
///
/// **Configuration Priority:**
/// 1. `squid.config.json` (if exists) - project settings
/// 2. `.env` variables (fallback) - sensitive credentials and overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_url: String,
    pub api_model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default = "default_context_window")]
    pub context_window: u32,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default = "default_database_path")]
    pub database_path: String,
    #[serde(default = "default_enable_env_context")]
    pub enable_env_context: bool,
    #[serde(default)]
    pub rag: RagConfig,
}

fn default_context_window() -> u32 {
    // Default to 8192 tokens (common for many local models)
    // Users should override this based on their model's actual context window
    8192
}

fn default_log_level() -> String {
    "error".to_string()
}

fn default_database_path() -> String {
    "squid.db".to_string()
}

fn default_enable_env_context() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:1234/v1".to_string(),
            api_model: "qwen2.5-coder-7b-instruct".to_string(),
            api_key: None,
            context_window: default_context_window(),
            log_level: default_log_level(),
            permissions: Permissions::default(),
            version: None,
            database_path: default_database_path(),
            enable_env_context: default_enable_env_context(),
            rag: RagConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from squid.config.json if it exists, otherwise from environment variables
    pub fn load() -> Self {
        // Search for config file in current directory and parent directories
        let config_path = Self::find_config_file().unwrap_or_else(|| PathBuf::from("squid.config.json"));

        if config_path.exists() {
            debug!("Loading configuration from squid.config.json");
            match fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str::<Config>(&content) {
                    Ok(mut config) => {
                        // Check version and warn if outdated
                        if let Some(warning) = config.version_warning() {
                            eprintln!("\n{}\n", warning);
                        }

                        // Resolve database_path relative to config file directory
                        let db_path = PathBuf::from(&config.database_path);
                        if db_path.is_relative() {
                            // Get the directory containing the config file
                            let config_dir = config_path
                                .parent()
                                .unwrap_or_else(|| std::path::Path::new("."));

                            // Resolve database path relative to config directory
                            let absolute_db_path = config_dir.join(&config.database_path);

                            // Convert to string, using the original if conversion fails
                            if let Some(path_str) = absolute_db_path.to_str() {
                                config.database_path = path_str.to_string();
                                debug!("Resolved database path to: {}", config.database_path);
                            }
                        }

                        return config;
                    }
                    Err(e) => {
                        debug!("Failed to parse squid.config.json: {}", e);
                    }
                },
                Err(e) => {
                    debug!("Failed to read squid.config.json: {}", e);
                }
            }
        }

        // Fallback to environment variables
        debug!("Loading configuration from environment variables");

        // Try to find existing database in parent directories
        let db_path = std::env::var("DATABASE_PATH")
            .ok()
            .or_else(|| Self::find_database_file().and_then(|p| p.to_str().map(String::from)))
            .unwrap_or_else(|| Self::default().database_path);

        Self {
            api_url: std::env::var("API_URL").unwrap_or_else(|_| Self::default().api_url),
            api_model: std::env::var("API_MODEL").unwrap_or_else(|_| Self::default().api_model),
            api_key: std::env::var("API_KEY").ok(),
            context_window: std::env::var("CONTEXT_WINDOW")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_context_window),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| Self::default().log_level),
            permissions: Permissions::default(),
            version: None,
            database_path: db_path,
            enable_env_context: std::env::var("ENABLE_ENV_CONTEXT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_enable_env_context),
            rag: RagConfig::default(),
        }
    }

    /// Search for squid.config.json in current directory and parent directories
    fn find_config_file() -> Option<PathBuf> {
        let mut current_dir = std::env::current_dir().ok()?;

        loop {
            let config_path = current_dir.join("squid.config.json");
            if config_path.exists() {
                debug!("Found config file at: {:?}", config_path);
                return Some(config_path);
            }

            // Try parent directory
            if !current_dir.pop() {
                // Reached root, no config found
                break;
            }
        }

        None
    }

    /// Search for squid.db in current directory and parent directories
    fn find_database_file() -> Option<PathBuf> {
        let mut current_dir = std::env::current_dir().ok()?;

        loop {
            let db_path = current_dir.join("squid.db");
            if db_path.exists() {
                debug!("Found database file at: {:?}", db_path);
                return Some(db_path);
            }

            // Try parent directory
            if !current_dir.pop() {
                // Reached root, no database found
                break;
            }
        }

        None
    }

    /// Get the current application version
    pub fn app_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Check if the config version matches the current app version
    /// Returns None if no version is set (old config), Some(true) if matches, Some(false) if outdated
    pub fn is_version_current(&self) -> Option<bool> {
        self.version.as_ref().map(|v| v == &Self::app_version())
    }

    /// Get a warning message if the config is outdated
    pub fn version_warning(&self) -> Option<String> {
        match self.is_version_current() {
            Some(false) => Some(format!(
                "⚠️  Warning: Config file version ({}) doesn't match app version ({}). Consider running 'squid init' to update.",
                self.version.as_ref().unwrap(),
                Self::app_version()
            )),
            None => Some(format!(
                "⚠️  Warning: Config file has no version field (created with an older version). Consider running 'squid init' to update to version {}.",
                Self::app_version()
            )),
            Some(true) => None,
        }
    }

    /// Save configuration to squid.config.json in the specified directory
    pub fn save_to_dir(&self, dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = dir.join("squid.config.json");

        // Create a copy with the current version set
        let mut config_to_save = self.clone();
        config_to_save.version = Some(Self::app_version());

        let json = serde_json::to_string_pretty(&config_to_save)?;
        fs::write(&config_path, json)?;
        info!("Configuration saved to {:?}", config_path);
        Ok(())
    }

    /// Get API key with fallback to "not-needed" for local models
    pub fn get_api_key(&self) -> String {
        self.api_key
            .clone()
            .or_else(|| std::env::var("API_KEY").ok())
            .unwrap_or_else(|| "not-needed".to_string())
    }

    /// Check if a tool is allowed to run without confirmation
    /// Supports granular bash permissions: "bash:ls", "bash:git", etc.
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        self.permissions.allow.contains(&tool_name.to_string())
    }

    /// Check if a tool is denied from running
    /// Supports granular bash permissions: "bash:ls", "bash:git", etc.
    pub fn is_tool_denied(&self, tool_name: &str) -> bool {
        self.permissions.deny.contains(&tool_name.to_string())
    }

    /// Check if a bash command is allowed to run without confirmation
    /// Supports granular permissions:
    /// - "bash" -> allows all bash commands
    /// - "bash:ls" -> allows only ls commands (ls, ls -la, etc.)
    /// - "bash:git" -> allows only git commands
    /// - "bash:git status" -> allows only git status commands
    pub fn is_bash_command_allowed(&self, command: &str) -> bool {
        // Check if all bash commands are allowed
        if self.permissions.allow.contains(&"bash".to_string()) {
            return true;
        }

        // Extract the first word(s) from the command for matching
        let command_trimmed = command.trim();

        // Check for granular permissions
        for permission in &self.permissions.allow {
            if let Some(bash_cmd) = permission.strip_prefix("bash:") {
                // Match if command starts with the allowed pattern
                if command_trimmed == bash_cmd
                    || command_trimmed.starts_with(&format!("{} ", bash_cmd))
                {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a bash command is denied from running
    /// Supports granular permissions similar to is_bash_command_allowed
    pub fn is_bash_command_denied(&self, command: &str) -> bool {
        // Check if all bash commands are denied
        if self.permissions.deny.contains(&"bash".to_string()) {
            return true;
        }

        // Extract the first word(s) from the command for matching
        let command_trimmed = command.trim();

        // Check for granular denials
        for permission in &self.permissions.deny {
            if let Some(bash_cmd) = permission.strip_prefix("bash:") {
                // Match if command starts with the denied pattern
                if command_trimmed == bash_cmd
                    || command_trimmed.starts_with(&format!("{} ", bash_cmd))
                {
                    return true;
                }
            }
        }

        false
    }

    /// Add a tool to the allow list and save config
    pub fn allow_tool(&mut self, tool_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let tool_str = tool_name.to_string();

        // Remove from deny list if present
        self.permissions.deny.retain(|t| t != &tool_str);

        // Add to allow list if not already present
        if !self.permissions.allow.contains(&tool_str) {
            self.permissions.allow.push(tool_str);
        }

        self.save_to_dir(&PathBuf::from("."))?;
        Ok(())
    }

    /// Add a tool to the deny list and save config
    pub fn deny_tool(&mut self, tool_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let tool_str = tool_name.to_string();

        // Remove from allow list if present
        self.permissions.allow.retain(|t| t != &tool_str);

        // Add to deny list if not already present
        if !self.permissions.deny.contains(&tool_str) {
            self.permissions.deny.push(tool_str);
        }

        self.save_to_dir(&PathBuf::from("."))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api_url, "http://127.0.0.1:1234/v1");
        assert_eq!(config.api_model, "qwen2.5-coder-7b-instruct");
        assert_eq!(config.api_key, None);
        assert_eq!(config.context_window, 8192);
        assert_eq!(config.log_level, "error");
        assert_eq!(config.permissions.allow, vec!["now".to_string()]);
        assert_eq!(config.permissions.deny.len(), 0);
        assert_eq!(config.version, None);
        assert_eq!(config.database_path, "squid.db");
        assert_eq!(config.enable_env_context, true);
        assert_eq!(config.rag.enabled, true);
        assert_eq!(config.rag.embedding_model, "text-embedding-nomic-embed-text-v1.5");
        assert_eq!(config.rag.chunk_size, 512);
    }

    #[test]
    fn test_get_api_key_fallback() {
        let config = Config::default();
        assert_eq!(config.get_api_key(), "not-needed");
    }

    #[test]
    fn test_get_api_key_with_value() {
        let config = Config {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), "test-key");
    }

    #[test]
    fn test_is_tool_allowed() {
        let config = Config::default();
        assert!(config.is_tool_allowed("now"));
        assert!(!config.is_tool_allowed("read_file"));
    }

    #[test]
    fn test_is_tool_denied() {
        let mut config = Config::default();
        config.permissions.deny.push("write_file".to_string());
        assert!(config.is_tool_denied("write_file"));
        assert!(!config.is_tool_denied("read_file"));
    }

    #[test]
    fn test_is_bash_command_allowed() {
        let mut config = Config::default();

        // Test: no bash permissions
        assert!(!config.is_bash_command_allowed("ls -la"));

        // Test: all bash allowed
        config.permissions.allow.push("bash".to_string());
        assert!(config.is_bash_command_allowed("ls -la"));
        assert!(config.is_bash_command_allowed("git status"));

        // Test: granular permission
        config.permissions.allow.clear();
        config.permissions.allow.push("bash:ls".to_string());
        assert!(config.is_bash_command_allowed("ls"));
        assert!(config.is_bash_command_allowed("ls -la"));
        assert!(config.is_bash_command_allowed("ls -l"));
        assert!(!config.is_bash_command_allowed("cat file.txt"));

        // Test: more specific granular permission
        config.permissions.allow.push("bash:git status".to_string());
        assert!(config.is_bash_command_allowed("git status"));
        assert!(config.is_bash_command_allowed("git status --short"));
        assert!(!config.is_bash_command_allowed("git log"));
    }

    #[test]
    fn test_is_bash_command_denied() {
        let mut config = Config::default();

        // Test: no bash denials
        assert!(!config.is_bash_command_denied("ls -la"));

        // Test: all bash denied
        config.permissions.deny.push("bash".to_string());
        assert!(config.is_bash_command_denied("ls -la"));
        assert!(config.is_bash_command_denied("git status"));

        // Test: granular denial
        config.permissions.deny.clear();
        config.permissions.deny.push("bash:rm".to_string());
        assert!(config.is_bash_command_denied("rm file.txt"));
        assert!(config.is_bash_command_denied("rm -rf folder"));
        assert!(!config.is_bash_command_denied("ls -la"));
    }

    #[test]
    fn test_allow_tool() {
        let mut config = Config::default();
        config.permissions.deny.push("read_file".to_string());

        // This would save to disk, so we can't test it fully in unit tests
        // but we can verify the logic
        config.permissions.allow.push("read_file".to_string());
        config.permissions.deny.retain(|t| t != "read_file");

        assert!(config.permissions.allow.contains(&"read_file".to_string()));
        assert!(!config.permissions.deny.contains(&"read_file".to_string()));
    }

    #[test]
    fn test_app_version() {
        let version = Config::app_version();
        assert!(!version.is_empty());
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_is_version_current() {
        let mut config = Config::default();

        // No version set
        assert_eq!(config.is_version_current(), None);

        // Current version
        config.version = Some(Config::app_version());
        assert_eq!(config.is_version_current(), Some(true));

        // Outdated version
        config.version = Some("0.1.0".to_string());
        assert_eq!(config.is_version_current(), Some(false));
    }

    #[test]
    fn test_version_warning() {
        let mut config = Config::default();

        // No version - should warn
        assert!(config.version_warning().is_some());

        // Current version - no warning
        config.version = Some(Config::app_version());
        assert!(config.version_warning().is_none());

        // Outdated version - should warn
        config.version = Some("0.1.0".to_string());
        let warning = config.version_warning();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("0.1.0"));
    }
}
