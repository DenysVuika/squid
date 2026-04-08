use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::agent::{
    AgentConfig, AgentPermissions, AgentsConfig, get_agents_dir, load_agents_from_dir,
};

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
    "http://127.0.0.1:1234/v1".to_string()
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

/// Plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsConfig {
    /// Enable plugin system
    #[serde(default = "default_plugins_enabled")]
    pub enabled: bool,
    /// Load global plugins from ~/.squid/plugins
    #[serde(default = "default_load_global")]
    pub load_global: bool,
    /// Load workspace plugins from ./plugins
    #[serde(default = "default_load_workspace")]
    pub load_workspace: bool,
    /// Load bundled plugins from executable directory
    #[serde(default = "default_load_bundled")]
    pub load_bundled: bool,
    /// Default timeout for plugin execution in seconds
    #[serde(default = "default_plugin_timeout")]
    pub default_timeout_seconds: u64,
    /// Maximum memory per plugin in MB
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: usize,
}

fn default_plugins_enabled() -> bool {
    true
}

fn default_load_global() -> bool {
    true
}

fn default_load_workspace() -> bool {
    true
}

fn default_load_bundled() -> bool {
    true
}

fn default_plugin_timeout() -> u64 {
    30
}

fn default_max_memory_mb() -> usize {
    128
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enabled: default_plugins_enabled(),
            load_global: default_load_global(),
            load_workspace: default_load_workspace(),
            load_bundled: default_load_bundled(),
            default_timeout_seconds: default_plugin_timeout(),
            max_memory_mb: default_max_memory_mb(),
        }
    }
}

/// Web client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    /// Enable notification sounds in the web client
    #[serde(default = "default_web_sounds")]
    pub sounds: bool,
}

fn default_web_sounds() -> bool {
    true
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            sounds: default_web_sounds(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Allow server to bind to 0.0.0.0 (accessible from local network)
    /// When false, server binds to 127.0.0.1 (localhost only)
    #[serde(default = "default_allow_network")]
    pub allow_network: bool,
}

fn default_allow_network() -> bool {
    false
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            allow_network: default_allow_network(),
        }
    }
}

/// Background jobs configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobsConfig {
    /// Enable background job scheduler
    #[serde(default = "default_jobs_enabled")]
    pub enabled: bool,
    /// Maximum number of concurrent job executions
    #[serde(default = "default_max_concurrent_jobs")]
    pub max_concurrent_jobs: usize,
    /// Maximum CPU percentage a job can use
    #[serde(default = "default_max_cpu_percent")]
    pub max_cpu_percent: i32,
    /// Default number of retries for failed jobs
    #[serde(default = "default_job_retries")]
    pub default_retries: i32,
}

fn default_jobs_enabled() -> bool {
    false
}

fn default_max_concurrent_jobs() -> usize {
    2
}

fn default_max_cpu_percent() -> i32 {
    70
}

fn default_job_retries() -> i32 {
    3
}

impl Default for JobsConfig {
    fn default() -> Self {
        Self {
            enabled: default_jobs_enabled(),
            max_concurrent_jobs: default_max_concurrent_jobs(),
            max_cpu_percent: default_max_cpu_percent(),
            default_retries: default_job_retries(),
        }
    }
}

/// Configuration for squid CLI
///
/// This configuration is typically stored in `squid.config.json` in the project directory.
///
/// **Fields:**
/// - `api_url`: Base URL for the LLM API (e.g., `http://127.0.0.1:1234/v1`)
/// - `api_key`: Optional API key (use `None` for local models)
/// - `context_window`: Maximum context window size in tokens (e.g., `32768` for Qwen2.5-Coder)
/// - `log_level`: Console logging verbosity (`error`, `warn`, `info`, `debug`, `trace`)
/// - `db_log_level`: Database logging verbosity (`error`, `warn`, `info`, `debug`, `trace`)
/// - `version`: Config file version (matches app version when created)
/// - `working_dir`: Working directory for file operations (default: `.`)
///
/// **Best Practices:**
/// - Commit `squid.config.json` to your repository to share project settings with your team
/// - Keep sensitive API keys in `.env` file (which is gitignored)
/// - Use `api_key: None` in config file for local models (LM Studio, Ollama)
/// - For cloud services (OpenAI, etc.), omit `api_key` from config and set it via `.env`
/// - Default `log_level` is `error` (minimal console noise)
/// - Default `db_log_level` is `debug` (capture detailed logs in database)
/// - Use `.squidignore` file for project-wide ignore patterns
///
/// **Configuration Priority:**
/// 1. `squid.config.json` (if exists) - project settings
/// 2. `.env` variables (fallback) - sensitive credentials and overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default = "default_context_window")]
    pub context_window: u32,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_db_log_level")]
    pub db_log_level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default = "default_database_path")]
    pub database_path: String,
    #[serde(default = "default_working_dir")]
    pub working_dir: String,
    #[serde(default)]
    pub rag: RagConfig,
    #[serde(default)]
    pub plugins: PluginsConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub web: WebConfig,
    #[serde(default)]
    pub jobs: JobsConfig,
    /// Default agent ID (agents are loaded from files, not from config)
    #[serde(default = "default_agent_id")]
    pub default_agent: String,

    // Non-serialized fields
    #[serde(skip)]
    pub agents: AgentsConfig,
    #[serde(skip)]
    pub config_dir: Option<PathBuf>,
}

fn default_agent_id() -> String {
    "general-assistant".to_string()
}

fn default_context_window() -> u32 {
    // Default to 8192 tokens (common for many local models)
    // Users should override this based on their model's actual context window
    8192
}

fn default_log_level() -> String {
    "error".to_string()
}

fn default_db_log_level() -> String {
    "debug".to_string()
}

fn default_database_path() -> String {
    "squid.db".to_string()
}

fn default_working_dir() -> String {
    "./workspace".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:1234/v1".to_string(),
            api_key: None,
            context_window: default_context_window(),
            log_level: default_log_level(),
            db_log_level: default_db_log_level(),
            version: None,
            database_path: default_database_path(),
            working_dir: default_working_dir(),
            rag: RagConfig::default(),
            plugins: PluginsConfig::default(),
            server: ServerConfig::default(),
            web: WebConfig::default(),
            jobs: JobsConfig::default(),
            default_agent: default_agent_id(),
            agents: AgentsConfig::default(),
            config_dir: None,
        }
    }
}

impl Config {
    /// Load configuration from squid.config.json if it exists, otherwise from environment variables
    pub fn load() -> Self {
        // Search for config file in current directory and parent directories
        let config_path =
            Self::find_config_file().unwrap_or_else(|| PathBuf::from("squid.config.json"));

        let mut config = if config_path.exists() {
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

                        config
                    }
                    Err(e) => {
                        debug!("Failed to parse squid.config.json: {}", e);
                        Self::default()
                    }
                },
                Err(e) => {
                    debug!("Failed to read squid.config.json: {}", e);
                    Self::default()
                }
            }
        } else {
            debug!("No squid.config.json found, using defaults");
            Self::default()
        };

        // Environment variables override config file settings
        debug!("Applying environment variable overrides");

        if let Ok(api_url) = std::env::var("API_URL") {
            debug!("Overriding API_URL from environment");
            config.api_url = api_url;
        }

        if let Ok(api_key) = std::env::var("API_KEY") {
            debug!("Overriding API_KEY from environment");
            config.api_key = Some(api_key);
        }

        if let Ok(context_window) = std::env::var("SQUID_CONTEXT_WINDOW")
            && let Ok(window) = context_window.parse()
        {
            debug!("Overriding SQUID_CONTEXT_WINDOW from environment");
            config.context_window = window;
        }

        if let Ok(log_level) = std::env::var("SQUID_LOG_LEVEL") {
            debug!("Overriding SQUID_LOG_LEVEL from environment");
            config.log_level = log_level;
        }

        if let Ok(db_log_level) = std::env::var("SQUID_DB_LOG_LEVEL") {
            debug!("Overriding SQUID_DB_LOG_LEVEL from environment");
            config.db_log_level = db_log_level;
        }

        if let Ok(db_path) = std::env::var("SQUID_DATABASE_PATH") {
            debug!("Overriding SQUID_DATABASE_PATH from environment");
            config.database_path = db_path;
        }

        if let Ok(working_dir) = std::env::var("SQUID_WORKING_DIR") {
            debug!("Overriding SQUID_WORKING_DIR from environment");
            config.working_dir = working_dir;
        }

        // RAG configuration overrides
        if let Ok(rag_enabled) = std::env::var("SQUID_RAG_ENABLED")
            && let Ok(enabled) = rag_enabled.parse()
        {
            debug!("Overriding SQUID_RAG_ENABLED from environment");
            config.rag.enabled = enabled;
        }

        if let Ok(embedding_model) = std::env::var("SQUID_EMBEDDING_MODEL") {
            debug!("Overriding SQUID_EMBEDDING_MODEL from environment");
            config.rag.embedding_model = embedding_model;
        }

        if let Ok(embedding_url) = std::env::var("SQUID_EMBEDDING_URL") {
            debug!("Overriding SQUID_EMBEDDING_URL from environment");
            config.rag.embedding_url = embedding_url;
        }

        if let Ok(chunk_size) = std::env::var("SQUID_RAG_CHUNK_SIZE")
            && let Ok(size) = chunk_size.parse()
        {
            debug!("Overriding SQUID_RAG_CHUNK_SIZE from environment");
            config.rag.chunk_size = size;
        }

        if let Ok(chunk_overlap) = std::env::var("SQUID_RAG_CHUNK_OVERLAP")
            && let Ok(overlap) = chunk_overlap.parse()
        {
            debug!("Overriding SQUID_RAG_CHUNK_OVERLAP from environment");
            config.rag.chunk_overlap = overlap;
        }

        if let Ok(top_k) = std::env::var("SQUID_RAG_TOP_K")
            && let Ok(k) = top_k.parse()
        {
            debug!("Overriding SQUID_RAG_TOP_K from environment");
            config.rag.top_k = k;
        }

        if let Ok(docs_path) = std::env::var("SQUID_RAG_DOCUMENTS_PATH") {
            debug!("Overriding SQUID_RAG_DOCUMENTS_PATH from environment");
            config.rag.documents_path = docs_path;
        }

        // Server configuration overrides
        if let Ok(allow_network) = std::env::var("SQUID_SERVER_ALLOW_NETWORK")
            && let Ok(enabled) = allow_network.parse()
        {
            debug!("Overriding SQUID_SERVER_ALLOW_NETWORK from environment");
            config.server.allow_network = enabled;
        }

        // Web client configuration overrides
        if let Ok(sounds) = std::env::var("SQUID_WEB_SOUNDS")
            && let Ok(enabled) = sounds.parse()
        {
            debug!("Overriding SQUID_WEB_SOUNDS from environment");
            config.web.sounds = enabled;
        }

        // Plugin configuration overrides
        if let Ok(load_bundled) = std::env::var("SQUID_PLUGINS_LOAD_BUNDLED")
            && let Ok(enabled) = load_bundled.parse()
        {
            debug!("Overriding SQUID_PLUGINS_LOAD_BUNDLED from environment");
            config.plugins.load_bundled = enabled;
        }

        // Background jobs configuration overrides
        if let Ok(jobs_enabled) = std::env::var("SQUID_JOBS_ENABLED")
            && let Ok(enabled) = jobs_enabled.parse()
        {
            debug!("Overriding SQUID_JOBS_ENABLED from environment");
            config.jobs.enabled = enabled;
        }

        if let Ok(max_concurrent) = std::env::var("SQUID_MAX_CONCURRENT_JOBS")
            && let Ok(count) = max_concurrent.parse()
        {
            debug!("Overriding SQUID_MAX_CONCURRENT_JOBS from environment");
            config.jobs.max_concurrent_jobs = count;
        }

        if let Ok(max_cpu) = std::env::var("SQUID_JOBS_MAX_CPU_PERCENT")
            && let Ok(cpu) = max_cpu.parse()
        {
            debug!("Overriding SQUID_JOBS_MAX_CPU_PERCENT from environment");
            config.jobs.max_cpu_percent = cpu;
        }

        if let Ok(retries) = std::env::var("SQUID_JOBS_DEFAULT_RETRIES")
            && let Ok(retry_count) = retries.parse()
        {
            debug!("Overriding SQUID_JOBS_DEFAULT_RETRIES from environment");
            config.jobs.default_retries = retry_count;
        }

        // Store config directory for agents loading
        config.config_dir = config_path.parent().map(|p| p.to_path_buf());

        config
    }

    /// Load agents from the agents directory
    pub fn load_agents(&mut self) {
        let agents_dir = get_agents_dir(self.config_dir.as_deref());
        let agents = load_agents_from_dir(&agents_dir);

        if agents.is_empty() {
            warn!(
                "No agents loaded from {:?}. Create .md files with YAML frontmatter.",
                agents_dir
            );
        } else {
            info!("Loaded {} agents from {:?}", agents.len(), agents_dir);
        }

        self.agents = AgentsConfig {
            agents,
            default_agent: self.default_agent.clone(),
        };
    }

    /// Check if a configuration file exists in the current directory or parent directories
    pub fn config_file_exists() -> bool {
        Self::find_config_file().is_some()
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
    pub fn save_to_dir(&self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

    /// Get agent configuration by ID
    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentConfig> {
        self.agents.agents.get(agent_id)
    }

    /// Get agent permissions by ID
    pub fn get_agent_permissions(&self, agent_id: &str) -> Option<&AgentPermissions> {
        self.get_agent(agent_id).map(|a| &a.permissions)
    }

    /// Get the default agent configuration
    pub fn get_default_agent(&self) -> Option<&AgentConfig> {
        self.get_agent(&self.agents.default_agent)
    }

    /// Add a tool to an agent's allow list
    /// Note: This modifies the in-memory config only.
    /// To persist changes, update the agent's .md file directly.
    pub fn allow_tool_for_agent(
        &mut self,
        agent_id: &str,
        tool_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tool_str = tool_name.to_string();

        let agent = self
            .agents
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?;

        // Add to allow list if not already present
        if !agent.permissions.allow.contains(&tool_str) {
            agent.permissions.allow.push(tool_str);
        }

        Ok(())
    }

    /// Remove a tool from an agent's allow list
    /// Note: This modifies the in-memory config only.
    /// To persist changes, update the agent's .md file directly.
    pub fn deny_tool_for_agent(
        &mut self,
        agent_id: &str,
        tool_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tool_str = tool_name.to_string();

        let agent = self
            .agents
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?;

        // Remove from allow list if present
        agent.permissions.allow.retain(|t| t != &tool_str);

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
        assert_eq!(config.api_key, None);
        assert_eq!(config.context_window, 8192);
        assert_eq!(config.log_level, "error");
        assert_eq!(config.version, None);
        assert_eq!(config.database_path, "squid.db");
        assert_eq!(config.working_dir, "./workspace");
        assert_eq!(config.rag.enabled, true);
        assert_eq!(
            config.rag.embedding_model,
            "text-embedding-nomic-embed-text-v1.5"
        );
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
