use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Plugin metadata loaded from plugin.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin (e.g., "markdown-linter")
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// Description of what the plugin does
    pub description: String,

    /// Plugin version (semantic versioning)
    pub version: String,

    /// API version this plugin is compatible with
    pub api_version: String,

    /// Security requirements and permissions
    pub security: SecurityRequirements,

    /// JSON schema for input validation
    pub input_schema: serde_json::Value,

    /// JSON schema for output validation
    pub output_schema: serde_json::Value,

    /// Path to the plugin directory (populated at runtime)
    #[serde(skip)]
    pub plugin_path: PathBuf,

    /// Whether this is a global or workspace plugin
    #[serde(skip)]
    pub is_global: bool,
}

/// Security requirements declared by the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// List of required permissions (e.g., ["read_file", "write_file"])
    #[serde(default)]
    pub requires: Vec<String>,

    /// Whether the plugin needs network access
    #[serde(default)]
    pub network: bool,

    /// Whether the plugin needs file write access
    #[serde(default)]
    pub file_write: bool,
}

impl PluginMetadata {
    /// Load plugin metadata from plugin.json file
    pub fn load(plugin_dir: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata_path = plugin_dir.join("plugin.json");
        let content = std::fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read plugin.json: {}", e))?;

        let mut metadata: PluginMetadata = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse plugin.json: {}", e))?;

        metadata.plugin_path = plugin_dir.to_path_buf();

        Ok(metadata)
    }

    /// Get the path to the plugin's index.js file
    pub fn index_js_path(&self) -> PathBuf {
        self.plugin_path.join("index.js")
    }

    /// Get the tool name as it will appear to the LLM (e.g., "plugin:markdown-linter")
    pub fn tool_name(&self) -> String {
        format!("plugin:{}", self.id)
    }

    /// Validate that the plugin has all required files
    pub fn validate_structure(&self) -> Result<(), String> {
        // Check if index.js exists
        let index_path = self.index_js_path();
        if !index_path.exists() {
            return Err("Missing index.js in plugin directory".to_string());
        }

        // Validate API version compatibility (currently only support "1.0")
        if self.api_version != "1.0" {
            return Err(format!(
                "Unsupported API version: {}. Only '1.0' is supported.",
                self.api_version
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_name_generation() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            title: "Test Plugin".to_string(),
            description: "A test plugin".to_string(),
            version: "0.1.0".to_string(),
            api_version: "1.0".to_string(),
            security: SecurityRequirements {
                requires: vec![],
                network: false,
                file_write: false,
            },
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            plugin_path: PathBuf::new(),
            is_global: false,
        };

        assert_eq!(metadata.tool_name(), "plugin:test-plugin");
    }
}
