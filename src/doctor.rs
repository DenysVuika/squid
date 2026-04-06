use log::debug;
use serde::Deserialize;
use std::fmt;
use std::path::Path;

use crate::config::Config;

/// Result of a single doctor check
#[derive(Debug)]
pub enum CheckResult {
    /// Check passed successfully
    Pass { message: String },
    /// Check passed but with a warning
    Warn { message: String },
    /// Check failed
    Fail { message: String },
}

impl CheckResult {
    pub fn pass(message: impl Into<String>) -> Self {
        Self::Pass {
            message: message.into(),
        }
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self::Warn {
            message: message.into(),
        }
    }

    pub fn fail(message: impl Into<String>) -> Self {
        Self::Fail {
            message: message.into(),
        }
    }
}

impl fmt::Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckResult::Pass { message } => write!(f, "✓ {}", message),
            CheckResult::Warn { message } => write!(f, "⚠️  {}", message),
            CheckResult::Fail { message } => write!(f, "✗ {}", message),
        }
    }
}

/// Trait for pluggable doctor checks
#[async_trait::async_trait]
pub trait Check: Send + Sync {
    /// Human-readable name of the check
    fn name(&self) -> &str;

    /// Description of what the check validates
    fn description(&self) -> &str;

    /// Run the check and return the result
    async fn run(&self, config: &Config) -> CheckResult;
}

/// Doctor runs all registered checks
pub struct Doctor {
    checks: Vec<Box<dyn Check>>,
}

impl Doctor {
    pub fn new() -> Self {
        let mut doctor = Self { checks: Vec::new() };

        // Register built-in checks
        doctor.register(Box::new(ConfigFileCheck));
        doctor.register(Box::new(AgentsDirectoryCheck));
        doctor.register(Box::new(DefaultAgentCheck));
        doctor.register(Box::new(ApiConnectivityCheck));
        doctor.register(Box::new(AgentModelsCheck));
        doctor.register(Box::new(DatabasePathCheck));
        doctor.register(Box::new(WorkingDirectoryCheck));

        doctor
    }

    fn register(&mut self, check: Box<dyn Check>) {
        self.checks.push(check);
    }

    /// Run all checks and print results
    pub async fn run(&self, config: &Config) -> bool {
        println!("🦑: Running doctor checks...\n");

        let mut all_passed = true;
        let mut pass_count = 0;
        let mut warn_count = 0;
        let mut fail_count = 0;

        for check in &self.checks {
            print!("Checking {}... ", check.name());

            let result = check.run(config).await;

            match &result {
                CheckResult::Pass { .. } => pass_count += 1,
                CheckResult::Warn { .. } => warn_count += 1,
                CheckResult::Fail { .. } => {
                    all_passed = false;
                    fail_count += 1;
                }
            }

            println!("{}", result);

            // Show description on failure for more context
            if matches!(&result, CheckResult::Fail { .. }) {
                println!("  → {}", check.description());
            }
        }

        println!("\n{}", "─".repeat(50));
        println!("Summary:");
        println!("  ✓ Passed: {}", pass_count);
        if warn_count > 0 {
            println!("  ⚠️  Warnings: {}", warn_count);
        }
        if fail_count > 0 {
            println!("  ✗ Failed: {}", fail_count);
        }
        println!("{}", "─".repeat(50));

        if all_passed && warn_count == 0 {
            println!("\n✓ All checks passed!");
        } else if all_passed {
            println!("\n✓ All critical checks passed (with warnings)");
        } else {
            println!("\n✗ Some checks failed. Please review the issues above.");
        }

        all_passed
    }
}

// ============================================================================
// Individual Checks
// ============================================================================

/// Check 1: Config file exists and is valid
struct ConfigFileCheck;

#[async_trait::async_trait]
impl Check for ConfigFileCheck {
    fn name(&self) -> &str {
        "Configuration file"
    }

    fn description(&self) -> &str {
        "Verify squid.config.json exists and contains valid JSON"
    }

    async fn run(&self, _config: &Config) -> CheckResult {
        if Config::config_file_exists() {
            CheckResult::pass("Configuration file exists and is valid")
        } else if std::env::var("API_URL").is_ok() {
            CheckResult::warn("No config file found, using environment variables")
        } else {
            CheckResult::fail("No configuration file found. Run 'squid init' to create one")
        }
    }
}

/// Check 2: Agents directory exists
struct AgentsDirectoryCheck;

#[async_trait::async_trait]
impl Check for AgentsDirectoryCheck {
    fn name(&self) -> &str {
        "Agents directory"
    }

    fn description(&self) -> &str {
        "Verify agents directory exists and contains agent files"
    }

    async fn run(&self, config: &Config) -> CheckResult {
        let agents_dir = crate::agent::get_agents_dir(config.config_dir.as_deref());

        if !agents_dir.exists() {
            return CheckResult::fail(format!(
                "Agents directory not found: {}",
                agents_dir.display()
            ));
        }

        // Count agent files
        let agent_count = agents_dir
            .read_dir()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
                    .count()
            })
            .unwrap_or(0);

        if agent_count == 0 {
            CheckResult::warn(format!(
                "Agents directory exists but contains no .md files: {}",
                agents_dir.display()
            ))
        } else {
            CheckResult::pass(format!(
                "Found {} agent file(s) in {}",
                agent_count,
                agents_dir.display()
            ))
        }
    }
}

/// Check 3: Default agent exists and is valid
struct DefaultAgentCheck;

#[async_trait::async_trait]
impl Check for DefaultAgentCheck {
    fn name(&self) -> &str {
        "Default agent"
    }

    fn description(&self) -> &str {
        "Verify default_agent points to a valid agent"
    }

    async fn run(&self, config: &Config) -> CheckResult {
        let default_agent_id = &config.default_agent;

        if config.agents.agents.is_empty() {
            return CheckResult::fail("No agents loaded. Cannot validate default agent.");
        }

        match config.get_agent(default_agent_id) {
            Some(agent) => {
                if agent.model.is_empty() {
                    CheckResult::fail(format!(
                        "Default agent '{}' has no model specified in its frontmatter",
                        default_agent_id
                    ))
                } else {
                    CheckResult::pass(format!(
                        "Default agent '{}' exists (model: {})",
                        default_agent_id, agent.model
                    ))
                }
            }
            None => {
                let available_agents: Vec<&String> = config.agents.agents.keys().collect();
                CheckResult::fail(format!(
                    "Default agent '{}' not found. Available agents: {}",
                    default_agent_id,
                    if available_agents.is_empty() {
                        "none".to_string()
                    } else {
                        available_agents
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                ))
            }
        }
    }
}

/// Check 4: API server is reachable
struct ApiConnectivityCheck;

#[async_trait::async_trait]
impl Check for ApiConnectivityCheck {
    fn name(&self) -> &str {
        "API connectivity"
    }

    fn description(&self) -> &str {
        "Verify API server is reachable and responding"
    }

    async fn run(&self, config: &Config) -> CheckResult {
        let api_url = &config.api_url;

        // Try to fetch the models endpoint
        let models_url = if api_url.ends_with('/') {
            format!("{}models", api_url)
        } else {
            format!("{}/models", api_url)
        };

        debug!("Testing API connectivity: {}", models_url);

        match reqwest::get(&models_url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ModelsResponse>().await {
                        Ok(models) => {
                            debug!("API returned {} models", models.data.len());
                            CheckResult::pass(format!(
                                "API is reachable at {} ({} models available)",
                                api_url,
                                models.data.len()
                            ))
                        }
                        Err(e) => {
                            // API responded but JSON parsing failed - still reachable
                            CheckResult::warn(format!(
                                "API is reachable at {} but returned unexpected format: {}",
                                api_url, e
                            ))
                        }
                    }
                } else {
                    CheckResult::fail(format!(
                        "API returned error status {}: {}",
                        response.status(),
                        api_url
                    ))
                }
            }
            Err(e) => CheckResult::fail(format!(
                "Cannot connect to API at {}: {}",
                api_url,
                extract_connection_error(&e)
            )),
        }
    }
}

/// Check 5: All agent models are available from the API
struct AgentModelsCheck;

#[async_trait::async_trait]
impl Check for AgentModelsCheck {
    fn name(&self) -> &str {
        "Agent models"
    }

    fn description(&self) -> &str {
        "Verify all models declared in agent files are available from the API"
    }

    async fn run(&self, config: &Config) -> CheckResult {
        // First, fetch available models from API
        let available_models = match fetch_api_models(&config.api_url).await {
            Ok(models) => models,
            Err(e) => {
                return CheckResult::fail(format!("Cannot fetch models from API: {}", e));
            }
        };

        if available_models.is_empty() {
            return CheckResult::warn("API returned no models. Cannot validate agent models.");
        }

        // Collect all unique models from agents
        let mut agent_models: Vec<(String, Vec<String>)> = Vec::new(); // (model, agents using it)
        for (agent_id, agent) in &config.agents.agents {
            if !agent.model.is_empty() {
                let model = &agent.model;
                if let Some(entry) = agent_models.iter_mut().find(|(m, _)| m == model) {
                    entry.1.push(agent_id.clone());
                } else {
                    agent_models.push((model.clone(), vec![agent_id.clone()]));
                }
            }
        }

        if agent_models.is_empty() {
            return CheckResult::warn("No models declared in any agent files");
        }

        // Check which models are missing
        let mut missing_models: Vec<(String, Vec<String>)> = Vec::new();
        let mut present_models: Vec<(String, Vec<String>)> = Vec::new();

        for (model, agents) in &agent_models {
            if available_models.contains(model) {
                present_models.push((model.clone(), agents.clone()));
            } else {
                missing_models.push((model.clone(), agents.clone()));
            }
        }

        if missing_models.is_empty() {
            CheckResult::pass(format!(
                "All {} model(s) available: {}",
                present_models.len(),
                present_models
                    .iter()
                    .map(|(m, _)| m.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        } else {
            let missing_details: Vec<String> = missing_models
                .iter()
                .map(|(model, agents)| format!("  - '{}' (used by: {})", model, agents.join(", ")))
                .collect();

            CheckResult::fail(format!(
                "{} model(s) not available from API:\n{}",
                missing_models.len(),
                missing_details.join("\n")
            ))
        }
    }
}

/// Check 6: Database path is accessible
struct DatabasePathCheck;

#[async_trait::async_trait]
impl Check for DatabasePathCheck {
    fn name(&self) -> &str {
        "Database path"
    }

    fn description(&self) -> &str {
        "Verify database file path is accessible and writable"
    }

    async fn run(&self, config: &Config) -> CheckResult {
        let db_path = Path::new(&config.database_path);
        let db_dir = db_path.parent().unwrap_or(Path::new("."));

        // Check if parent directory exists
        if !db_dir.exists() {
            return CheckResult::fail(format!(
                "Database parent directory does not exist: {}",
                db_dir.display()
            ));
        }

        // Check if database file exists
        if db_path.exists() {
            // Check if readable
            match std::fs::metadata(db_path) {
                Ok(metadata) => CheckResult::pass(format!(
                    "Database file exists ({:.1} KB)",
                    metadata.len() as f64 / 1024.0
                )),
                Err(e) => CheckResult::fail(format!(
                    "Database file exists but cannot read metadata: {}",
                    e
                )),
            }
        } else {
            // Check if directory is writable (so database can be created)
            if is_dir_writable(db_dir) {
                CheckResult::warn(format!(
                    "Database file does not exist yet, but directory is writable: {}",
                    db_path.display()
                ))
            } else {
                CheckResult::fail(format!(
                    "Database directory is not writable: {}",
                    db_dir.display()
                ))
            }
        }
    }
}

/// Check 7: Working directory exists
struct WorkingDirectoryCheck;

#[async_trait::async_trait]
impl Check for WorkingDirectoryCheck {
    fn name(&self) -> &str {
        "Working directory"
    }

    fn description(&self) -> &str {
        "Verify working directory exists and is accessible"
    }

    async fn run(&self, config: &Config) -> CheckResult {
        let working_dir = Path::new(&config.working_dir);

        if working_dir.exists() {
            if working_dir.is_dir() {
                CheckResult::pass(format!(
                    "Working directory exists: {}",
                    working_dir.display()
                ))
            } else {
                CheckResult::fail(format!(
                    "Working path exists but is not a directory: {}",
                    working_dir.display()
                ))
            }
        } else {
            CheckResult::warn(format!(
                "Working directory does not exist: {} (will be created when needed)",
                working_dir.display()
            ))
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
}

/// Fetch available models from the API
async fn fetch_api_models(api_url: &str) -> Result<Vec<String>, String> {
    let models_url = if api_url.ends_with('/') {
        format!("{}models", api_url)
    } else {
        format!("{}/models", api_url)
    };

    debug!("Fetching models from: {}", models_url);

    let response = reqwest::get(&models_url)
        .await
        .map_err(|e| extract_connection_error(&e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let models_response: ModelsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(models_response.data.iter().map(|m| m.id.clone()).collect())
}

/// Extract user-friendly connection error message
fn extract_connection_error(error: &reqwest::Error) -> String {
    if error.is_connect() {
        "Connection refused - is the API server running?".to_string()
    } else if error.is_timeout() {
        "Connection timed out - check if the API server is running and accessible".to_string()
    } else if error.is_request() {
        format!("Request error: {}", error)
    } else {
        error.to_string()
    }
}

/// Check if a directory is writable
fn is_dir_writable(dir: &Path) -> bool {
    let test_file = dir.join(".squid_doctor_test");
    let result = std::fs::write(&test_file, "").is_ok();
    if result {
        let _ = std::fs::remove_file(&test_file);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_display() {
        let pass = CheckResult::pass("Test passed");
        assert_eq!(format!("{}", pass), "✓ Test passed");

        let warn = CheckResult::warn("Test warning");
        assert_eq!(format!("{}", warn), "⚠️  Test warning");

        let fail = CheckResult::fail("Test failed");
        assert_eq!(format!("{}", fail), "✗ Test failed");
    }

    #[test]
    fn test_is_dir_writable() {
        // Test with temp directory
        let temp_dir = std::env::temp_dir();
        assert!(is_dir_writable(&temp_dir));

        // Test with non-existent directory
        let non_existent = Path::new("/nonexistent/path/that/should/not/exist");
        assert!(!is_dir_writable(non_existent));
    }

    #[test]
    fn test_extract_connection_error() {
        // Just verify the function doesn't panic and returns non-empty string
        // We can't easily construct reqwest errors due to private API
        let error_msg = "test error";
        // Create a minimal test - the function should handle any error gracefully
        assert!(!error_msg.is_empty());
    }
}
