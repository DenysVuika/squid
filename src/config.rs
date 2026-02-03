use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration for squid CLI
///
/// This configuration is typically stored in `squid.config.json` in the project directory.
///
/// **Fields:**
/// - `api_url`: Base URL for the LLM API (e.g., `http://127.0.0.1:1234/v1`)
/// - `api_model`: Model identifier (e.g., `local-model`, `qwen2.5-coder`, `gpt-4`)
/// - `api_key`: Optional API key (use `None` for local models)
/// - `log_level`: Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`)
///
/// **Best Practices:**
/// - Commit `squid.config.json` to your repository to share project settings with your team
/// - Keep sensitive API keys in `.env` file (which is gitignored)
/// - Use `api_key: None` in config file for local models (LM Studio, Ollama)
/// - For cloud services (OpenAI, etc.), omit `api_key` from config and set it via `.env`
/// - Default `log_level` is `error` (minimal noise)
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
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "error".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:1234/v1".to_string(),
            api_model: "local-model".to_string(),
            api_key: None,
            log_level: default_log_level(),
        }
    }
}

impl Config {
    /// Load configuration from squid.config.json if it exists, otherwise from environment variables
    pub fn load() -> Self {
        let config_path = PathBuf::from("squid.config.json");

        if config_path.exists() {
            debug!("Loading configuration from squid.config.json");
            match fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str::<Config>(&content) {
                    Ok(config) => {
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
        Self {
            api_url: std::env::var("API_URL").unwrap_or_else(|_| Self::default().api_url),
            api_model: std::env::var("API_MODEL").unwrap_or_else(|_| Self::default().api_model),
            api_key: std::env::var("API_KEY").ok(),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| Self::default().log_level),
        }
    }

    /// Save configuration to squid.config.json in the specified directory
    pub fn save_to_dir(&self, dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = dir.join("squid.config.json");
        let json = serde_json::to_string_pretty(self)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api_url, "http://127.0.0.1:1234/v1");
        assert_eq!(config.api_model, "local-model");
        assert_eq!(config.api_key, None);
        assert_eq!(config.log_level, "error");
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
}
