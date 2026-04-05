use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// YAML frontmatter metadata for an agent
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetadata {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_model: Option<String>,
    /// Permissions list - deserialized directly as a list for simplicity
    #[serde(default, deserialize_with = "deserialize_permissions")]
    pub permissions: AgentPermissions,
    /// Whether this agent can use tools (default: true).
    #[serde(default = "default_true")]
    pub use_tools: bool,
    /// Optional list of suggested prompts shown in the web UI for this agent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
}

/// Deserialize permissions from a YAML sequence directly into AgentPermissions.allow
fn deserialize_permissions<'de, D>(deserializer: D) -> Result<AgentPermissions, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let allow = Vec::<String>::deserialize(deserializer)?;
    Ok(AgentPermissions { allow })
}

/// Agent permissions - allow-only model
/// Everything not in the allow list is denied by default.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentPermissions {
    #[serde(default)]
    pub allow: Vec<String>,
}

/// Full agent configuration (metadata + prompt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub description: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
    #[serde(default)]
    pub permissions: AgentPermissions,
    /// Whether this agent can use tools (default: true).
    #[serde(default = "default_true")]
    pub use_tools: bool,
    /// Optional list of suggested prompts shown in the web UI for this agent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
}

impl From<AgentMetadata> for AgentConfig {
    fn from(metadata: AgentMetadata) -> Self {
        Self {
            name: metadata.name,
            enabled: metadata.enabled,
            description: metadata.description,
            model: metadata.model,
            prompt: None,
            pricing_model: metadata.pricing_model,
            context_window: metadata.context_window,
            permissions: metadata.permissions,
            use_tools: metadata.use_tools,
            suggestions: metadata.suggestions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    #[serde(default = "default_agent_id")]
    pub default_agent: String,
}

fn default_true() -> bool {
    true
}

fn default_agent_id() -> String {
    "general-assistant".to_string()
}

/// Parse YAML frontmatter from a markdown file
/// Returns (metadata, prompt_content) or None if parsing fails
pub fn parse_agent_file(content: &str) -> Option<(AgentMetadata, String)> {
    let content = content.trim();

    // Must start with ---
    if !content.starts_with("---") {
        return None;
    }

    // Find the closing ---
    let rest = &content[3..];
    let end_marker = rest.find("\n---")?;

    let yaml_block = &rest[..end_marker];
    let prompt = if end_marker + 4 < rest.len() {
        rest[end_marker + 4..].trim().to_string()
    } else {
        String::new()
    };

    match serde_yaml::from_str::<AgentMetadata>(yaml_block) {
        Ok(metadata) => Some((metadata, prompt)),
        Err(e) => {
            warn!("Failed to parse agent metadata from YAML: {}", e);
            None
        }
    }
}

/// Load a single agent from a markdown file
pub fn load_agent_from_file(path: &Path) -> Option<(String, AgentConfig)> {
    let content = fs::read_to_string(path).ok()?;
    let file_stem = path.file_stem()?.to_string_lossy().to_string();

    if let Some((metadata, prompt)) = parse_agent_file(&content) {
        let mut config: AgentConfig = metadata.into();
        if !prompt.is_empty() {
            config.prompt = Some(prompt);
        }
        Some((file_stem, config))
    } else {
        warn!("Failed to parse agent file: {:?}", path);
        None
    }
}

/// Load all agents from the agents directory
/// Returns a map of agent ID -> AgentConfig
pub fn load_agents_from_dir(dir_path: &Path) -> HashMap<String, AgentConfig> {
    let mut agents = HashMap::new();

    if !dir_path.exists() {
        debug!("Agents directory does not exist: {:?}", dir_path);
        return agents;
    }

    if !dir_path.is_dir() {
        warn!("Agents path is not a directory: {:?}", dir_path);
        return agents;
    }

    debug!("Loading agents from: {:?}", dir_path);

    // Read directory
    let entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read agents directory: {}", e);
            return agents;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Only process .md files
        if path.extension().map_or(true, |ext| ext != "md") {
            continue;
        }

        if let Some((id, config)) = load_agent_from_file(&path) {
            debug!("Loaded agent: {} ({})", id, config.name);
            agents.insert(id, config);
        }
    }

    agents
}

/// Get the agents directory path.
/// Priority:
/// 1. `SQUID_AGENTS_DIR` env var (explicit override)
/// 2. `agents/` folder relative to config file directory
/// 3. `agents/` folder in the current working directory
/// 4. Bundled agents shipped alongside the executable
pub fn get_agents_dir(config_dir: Option<&Path>) -> PathBuf {
    // 1. Check environment variable first
    if let Ok(dir) = std::env::var("SQUID_AGENTS_DIR") {
        let path = PathBuf::from(&dir);
        debug!("Using SQUID_AGENTS_DIR: {:?}", path);
        return path;
    }

    // 2. Default to "agents" folder relative to config directory
    if let Some(config_dir) = config_dir {
        let agents_dir = config_dir.join("agents");
        if agents_dir.exists() {
            debug!("Using agents dir relative to config: {:?}", agents_dir);
            return agents_dir;
        }
    }

    // 3. Check agents/ in current working directory
    let cwd_agents = PathBuf::from("agents");
    if cwd_agents.exists() {
        debug!("Using agents dir from cwd: {:?}", cwd_agents);
        return cwd_agents;
    }

    // 4. Fallback to bundled agents next to the executable
    if let Some(bundled) = get_bundled_agents_dir() {
        debug!("Using bundled agents dir: {:?}", bundled);
        return bundled;
    }

    // Last resort: local "agents" folder (may not exist)
    PathBuf::from("agents")
}

/// Get the bundled agents directory path (relative to the executable).
/// This is used when agents are shipped with the binary (e.g., from crates.io).
pub fn get_bundled_agents_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
        .map(|exe_dir| exe_dir.join("agents"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agent_file_valid() {
        let content = r#"---
name: Test Agent
enabled: true
description: A test agent
model: test-model
permissions:
  - read_file
  - write_file
suggestions:
  - Test suggestion 1
---
You are a test agent prompt.
"#;
        let result = parse_agent_file(content);
        assert!(result.is_some());

        let (metadata, prompt) = result.unwrap();
        assert_eq!(metadata.name, "Test Agent");
        assert!(metadata.enabled);
        assert_eq!(metadata.description, "A test agent");
        assert_eq!(metadata.model, "test-model");
        assert_eq!(metadata.permissions.allow, vec!["read_file", "write_file"]);
        assert_eq!(metadata.suggestions, vec!["Test suggestion 1"]);
        assert_eq!(prompt, "You are a test agent prompt.");
    }

    #[test]
    fn test_parse_agent_file_no_prompt() {
        let content = r#"---
name: No Prompt Agent
enabled: true
description: An agent without a prompt
model: test-model
permissions: []
---
"#;
        let result = parse_agent_file(content);
        assert!(result.is_some());

        let (metadata, prompt) = result.unwrap();
        assert_eq!(metadata.name, "No Prompt Agent");
        assert_eq!(prompt, "");
    }

    #[test]
    fn test_parse_agent_file_use_tools_false() {
        let content = r#"---
name: No Tools Agent
enabled: true
description: An agent without tools
model: test-model
use_tools: false
permissions: []
---
You cannot use tools.
"#;
        let result = parse_agent_file(content);
        assert!(result.is_some());

        let (metadata, _) = result.unwrap();
        assert!(!metadata.use_tools);
    }

    #[test]
    fn test_parse_agent_file_invalid_no_frontmatter() {
        let content = "Just plain text without frontmatter";
        let result = parse_agent_file(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_agent_file_invalid_yaml() {
        let content = r#"---
name: [invalid yaml
---
Prompt
"#;
        let result = parse_agent_file(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_agent_file_empty_permissions() {
        let content = r#"---
name: Empty Perms Agent
enabled: true
description: An agent with no permissions
model: test-model
permissions: []
---
Test prompt.
"#;
        let result = parse_agent_file(content);
        assert!(result.is_some());

        let (metadata, _) = result.unwrap();
        assert!(metadata.permissions.allow.is_empty());
    }

    #[test]
    fn test_parse_agent_file_with_context_window() {
        let content = r#"---
name: Context Agent
enabled: true
description: An agent with custom context window
model: test-model
context_window: 16384
pricing_model: gpt-4o-mini
permissions:
  - read_file
---
Test prompt.
"#;
        let result = parse_agent_file(content);
        assert!(result.is_some());

        let (metadata, _) = result.unwrap();
        assert_eq!(metadata.context_window, Some(16384));
        assert_eq!(metadata.pricing_model, Some("gpt-4o-mini".to_string()));
    }

    #[test]
    fn test_get_agents_dir_from_env() {
        // Note: set_var/remove_var require unsafe in Rust 2024 edition
        // This test just verifies the default behavior when env var is not set
        unsafe { std::env::remove_var("SQUID_AGENTS_DIR") };
        let result = get_agents_dir(None);
        assert_eq!(result, PathBuf::from("agents"));
    }

    #[test]
    fn test_get_agents_dir_from_config() {
        use std::fs;
        let config_dir = PathBuf::from("/tmp/squid_test_config");
        let agents_dir = config_dir.join("agents");
        // Create the dir so it exists for the test
        let _ = fs::create_dir_all(&agents_dir);
        let result = get_agents_dir(Some(&config_dir));
        assert_eq!(result, agents_dir);
        let _ = fs::remove_dir_all(&config_dir);
    }

    #[test]
    fn test_get_agents_dir_default() {
        // Make sure env var is not set
        unsafe { std::env::remove_var("SQUID_AGENTS_DIR") };
        let result = get_agents_dir(None);
        assert_eq!(result, PathBuf::from("agents"));
    }
}
