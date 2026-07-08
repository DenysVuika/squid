use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Parsed metadata from a plugin's `plugin.json` descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Stable plugin identifier.
    pub id: String,
    /// Human-friendly plugin title.
    pub title: String,
    /// Tool description shown to the model.
    pub description: String,
    /// Plugin semantic version.
    pub version: String,
    /// Plugin API compatibility version.
    pub api_version: String,
    /// Declared plugin permissions.
    pub security: SecurityRequirements,
    /// JSON schema for plugin input.
    pub input_schema: serde_json::Value,
    /// JSON schema for plugin output.
    pub output_schema: serde_json::Value,
    /// Populated absolute plugin directory path.
    #[serde(skip)]
    pub plugin_path: PathBuf,
    /// Indicates whether plugin came from a global source.
    #[serde(skip)]
    pub is_global: bool,
}

/// Security flags and permission requirements declared by plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Required host capabilities (checked by the host app).
    #[serde(default)]
    pub requires: Vec<String>,
    /// Enables `httpGet` API when true.
    #[serde(default)]
    pub network: bool,
    /// Enables `writeFile` API when true.
    #[serde(default)]
    pub file_write: bool,
}

impl PluginMetadata {
    /// Loads and parses `plugin.json` in the given directory.
    pub fn load(plugin_dir: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata_path = plugin_dir.join("plugin.json");
        let content = std::fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read plugin.json: {}", e))?;

        let mut metadata: PluginMetadata = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse plugin.json: {}", e))?;

        metadata.plugin_path = plugin_dir.to_path_buf();

        Ok(metadata)
    }

    /// Returns the plugin entrypoint path (`index.js`).
    pub fn index_js_path(&self) -> PathBuf {
        self.plugin_path.join("index.js")
    }

    /// Returns the model-exposed tool name for this plugin.
    pub fn tool_name(&self) -> String {
        format!("plugin:{}", self.id)
    }

    /// Validates required files and supported API version.
    pub fn validate_structure(&self) -> Result<(), String> {
        let index_path = self.index_js_path();
        if !index_path.exists() {
            return Err("Missing index.js in plugin directory".to_string());
        }

        if self.api_version != "1.0" {
            return Err(format!(
                "Unsupported API version: {}. Only '1.0' is supported.",
                self.api_version
            ));
        }

        Ok(())
    }
}
