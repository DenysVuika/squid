use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use chrono::{Local, Utc};
use console::style;
use inquire::Select;
use log::{debug, error, info, warn};
use regex::Regex;
use serde_json::json;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;
use walkdir::WalkDir;

use crate::config::Config;
use crate::validate::PathValidator;

/// Get the list of available tools for the LLM
pub fn get_tools() -> Vec<ChatCompletionTools> {
    vec![
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("read_file")
                .description("Read the contents of a file from the filesystem")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to read"
                        }
                    },
                    "required": ["path"]
                }))
                .build()
                .expect("Failed to build read_file function"),
        }),
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("write_file")
                .description("Write content to a file on the filesystem")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path where the file should be written"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }))
                .build()
                .expect("Failed to build write_file function"),
        }),
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("grep")
                .description("Search for a pattern in files using regex. Searches recursively from a given directory or in a specific file.")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "The regex pattern to search for"
                        },
                        "path": {
                            "type": "string",
                            "description": "The file or directory path to search in. If a directory, searches recursively."
                        },
                        "case_sensitive": {
                            "type": "boolean",
                            "description": "Whether the search should be case-sensitive (default: false)"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 50)"
                        }
                    },
                    "required": ["pattern", "path"]
                }))
                .build()
                .expect("Failed to build grep function"),
        }),
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("now")
                .description("Get the current date and time in RFC 3339 format. Only use this when the user specifically asks for it or when current datetime is needed.")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "timezone": {
                            "type": "string",
                            "description": "The timezone to use for the datetime. Options: 'local' for local time (default, use for most queries) or 'utc' for UTC time.",
                            "enum": ["utc", "local"]
                        }
                    },
                    "required": []
                }))
                .build()
                .expect("Failed to build now function"),
        }),
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("bash")
                .description("Execute a bash command. Only use this for safe, non-destructive commands like ls, git status, cat, etc. Dangerous commands (rm, sudo, chmod, dd, curl, wget, kill) are automatically blocked.")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The bash command to execute (e.g., 'ls -la', 'git status', 'cat file.txt')"
                        },
                        "timeout": {
                            "type": "integer",
                            "description": "Maximum execution time in seconds (default: 10, max: 60)",
                            "minimum": 1,
                            "maximum": 60
                        }
                    },
                    "required": ["command"]
                }))
                .build()
                .expect("Failed to build bash function"),
        }),
    ]
}

// Helper function to search in a single file
fn search_file(
    path: &std::path::Path,
    regex: &Regex,
    max_results: usize,
    results: &mut Vec<serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    if results.len() >= max_results {
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;

    for (line_num, line) in content.lines().enumerate() {
        if regex.is_match(line) {
            results.push(json!({
                "file": path.display().to_string(),
                "line": line_num + 1,
                "content": line
            }));

            if results.len() >= max_results {
                break;
            }
        }
    }

    Ok(())
}

// Execute grep search
async fn execute_bash(command: &str, timeout_secs: u64) -> Result<String, String> {
    let output = timeout(
        Duration::from_secs(timeout_secs),
        tokio::task::spawn_blocking({
            let command = command.to_string();
            move || {
                Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
            }
        }),
    )
    .await
    .map_err(|_| format!("Command timed out after {} seconds", timeout_secs))?
    .map_err(|e| format!("Failed to spawn command: {}", e))?
    .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Command failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

fn execute_grep(
    pattern: &str,
    path: &str,
    case_sensitive: bool,
    max_results: usize,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let regex = if case_sensitive {
        Regex::new(pattern)?
    } else {
        Regex::new(&format!("(?i){}", pattern))?
    };

    let mut results = Vec::new();
    let search_path = std::path::Path::new(path);

    // Initialize path validator with ignore patterns
    let ignore_patterns = PathValidator::load_ignore_patterns();
    let validator = PathValidator::with_ignore_file(if ignore_patterns.is_empty() {
        None
    } else {
        Some(ignore_patterns)
    });

    if search_path.is_file() {
        // Search in a single file
        search_file(search_path, &regex, max_results, &mut results)?;
    } else if search_path.is_dir() {
        // Search recursively in directory
        for entry in WalkDir::new(search_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            // Skip if not a file
            if !entry_path.is_file() {
                continue;
            }

            // Skip if path validation fails (respects .squidignore)
            if validator.validate(entry_path).is_err() {
                debug!(
                    "Skipping ignored path during grep: {}",
                    entry_path.display()
                );
                continue;
            }

            // Skip binary files and common non-text files by extension
            if let Some(ext) = entry_path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if matches!(
                    ext_str.as_str(),
                    "jpg"
                        | "jpeg"
                        | "png"
                        | "gif"
                        | "bmp"
                        | "ico"
                        | "webp"
                        | "pdf"
                        | "zip"
                        | "tar"
                        | "gz"
                        | "rar"
                        | "7z"
                        | "exe"
                        | "dll"
                        | "so"
                        | "dylib"
                        | "bin"
                        | "dat"
                        | "mp4"
                        | "mov"
                        | "avi"
                        | "mkv"
                        | "iso"
                        | "db"
                        | "sqlite"
                        | "sqlite3"
                ) {
                    debug!("Skipping binary file during grep: {}", entry_path.display());
                    continue;
                }
            }

            // Skip empty files
            if let Ok(metadata) = entry_path.metadata() {
                if metadata.len() == 0 {
                    continue;
                }
            }

            // Try to search the file
            if let Err(e) = search_file(entry_path, &regex, max_results, &mut results) {
                debug!("Skipping file {} due to error: {}", entry_path.display(), e);
                continue;
            }

            if results.len() >= max_results {
                break;
            }
        }
    }

    Ok(results)
}

/// Permission choices for tool execution
#[derive(Debug, Clone, Copy, PartialEq)]
enum PermissionChoice {
    Yes,
    No,
    Always,
    Never,
}

impl std::fmt::Display for PermissionChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionChoice::Yes => write!(f, "Yes (this time)"),
            PermissionChoice::No => write!(f, "No (skip)"),
            PermissionChoice::Always => write!(f, "Always (add to allow list)"),
            PermissionChoice::Never => write!(f, "Never (add to deny list)"),
        }
    }
}

pub async fn call_tool(name: &str, args: &str, config: &Config) -> serde_json::Value {
    info!("Tool call: {} with args: {}", name, args);

    // Parse arguments first to get command for bash tool
    let args: serde_json::Value = match args.parse() {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse tool arguments: {}", e);
            return json!({"error": format!("Invalid arguments: {}", e)});
        }
    };

    // MANDATORY SECURITY CHECK: Block dangerous bash commands BEFORE any permission checks
    // This cannot be bypassed by configuration or user approval
    if name == "bash" {
        let command = args["command"].as_str().unwrap_or("");

        let dangerous_patterns = [
            "rm -rf", "rm -f", "sudo ", "chmod ", "dd ", "mkfs", "fdisk", "> /dev/", "curl",
            "wget", "kill ", "pkill", "killall",
        ];

        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                warn!("Blocked dangerous bash command: {}", command);
                return json!({
                    "error": format!(
                        "Command blocked for security reasons. The command contains a dangerous pattern: '{}'. Commands like rm, sudo, chmod, dd, curl, wget, and kill operations are not allowed.",
                        pattern
                    ),
                    "skipped": true
                });
            }
        }
    }

    // Check if tool is denied (with granular bash command support)
    if name == "bash" {
        let command = args["command"].as_str().unwrap_or("");
        if config.is_bash_command_denied(command) {
            warn!(
                "Bash command '{}' is in the deny list, blocking execution",
                command
            );
            return json!({"error": format!("Bash command '{}' is denied by configuration", command), "skipped": true});
        }
    } else if config.is_tool_denied(name) {
        warn!("Tool '{}' is in the deny list, blocking execution", name);
        return json!({"error": format!("Tool '{}' is denied by configuration", name), "skipped": true});
    }

    // Initialize path validator with ignore patterns from .squidignore
    let ignore_patterns = PathValidator::load_ignore_patterns();
    let validator = PathValidator::with_ignore_file(if ignore_patterns.is_empty() {
        None
    } else {
        Some(ignore_patterns)
    });

    // Validate paths BEFORE asking for user approval
    let validated_path = match name {
        "read_file" | "write_file" | "grep" => {
            let path = args["path"].as_str().unwrap_or("");
            match validator.validate(std::path::Path::new(path)) {
                Ok(p) => Some(p),
                Err(e) => {
                    debug!("Path validation failed for {}: {}", name, e);
                    let friendly_message = match e {
                        crate::validate::PathValidationError::PathIgnored(_) => {
                            format!(
                                "I cannot access '{}' because it's protected by the project's .squidignore file. This is a security measure to prevent access to sensitive files.",
                                path
                            )
                        }
                        crate::validate::PathValidationError::PathNotAllowed(ref msg)
                            if msg.contains("blacklisted") =>
                        {
                            format!(
                                "I cannot access '{}' because it's a protected system file or directory. Access to this location is blocked for security reasons.",
                                path
                            )
                        }
                        crate::validate::PathValidationError::PathNotAllowed(ref msg)
                            if msg.contains("not whitelisted") =>
                        {
                            format!(
                                "I cannot access '{}' because it's outside the current project directory. I can only access files within the current workspace for security reasons.",
                                path
                            )
                        }
                        _ => {
                            format!("I cannot access '{}' due to security restrictions.", path)
                        }
                    };
                    return json!({"content": friendly_message});
                }
            }
        }
        _ => None,
    };

    // Check if tool is auto-allowed (with granular bash command support)
    let auto_allowed = if name == "bash" {
        let command = args["command"].as_str().unwrap_or("");
        config.is_bash_command_allowed(command)
    } else {
        config.is_tool_allowed(name)
    };

    // Ask for user approval if not auto-allowed
    let permission = if auto_allowed {
        info!("Tool '{}' is in the allow list, auto-approving", name);
        PermissionChoice::Yes
    } else {
        // Build approval message with styled formatting
        let approval_message = match name {
            "read_file" => {
                let path = args["path"].as_str().unwrap_or("unknown");
                format!(
                    "Can I {}?\n  📄 File: {}",
                    style("read this file").yellow(),
                    style(path).green()
                )
            }
            "write_file" => {
                let path = args["path"].as_str().unwrap_or("unknown");
                let content = args["content"].as_str().unwrap_or("");
                let preview = if content.len() > 100 {
                    format!("{}... ({} bytes total)", &content[..100], content.len())
                } else {
                    content.to_string()
                };
                format!(
                    "Can I {}?\n  📄 File: {}\n  📝 Content preview:\n{}",
                    style("write to this file").yellow(),
                    style(path).green(),
                    style(&preview).dim()
                )
            }
            "grep" => {
                let pattern = args["pattern"].as_str().unwrap_or("unknown");
                let path = args["path"].as_str().unwrap_or("unknown");
                format!(
                    "Can I {}?\n  🔍 Pattern: {}\n  📂 Path: {}",
                    style("search for this pattern").yellow(),
                    style(pattern).magenta(),
                    style(path).green()
                )
            }
            "now" => {
                let timezone = args["timezone"].as_str().unwrap_or("utc");
                format!(
                    "Can I {}?\n  🕐 Timezone: {}",
                    style("get the current date and time").yellow(),
                    style(timezone).cyan()
                )
            }
            "bash" => {
                let command = args["command"].as_str().unwrap_or("unknown");
                let timeout_secs = args["timeout"].as_u64().unwrap_or(10);
                format!(
                    "Can I {}?\n  💻 Command: {}\n  ⏱️  Timeout: {} seconds",
                    style("execute this bash command").yellow(),
                    style(command).magenta(),
                    style(timeout_secs).cyan()
                )
            }
            _ => format!("Can I execute: {}?", style(name).yellow()),
        };

        let options = vec![
            PermissionChoice::Yes,
            PermissionChoice::No,
            PermissionChoice::Always,
            PermissionChoice::Never,
        ];

        match Select::new(&approval_message, options)
            .with_help_message(&format!(
                "{} Use arrow keys to navigate, {} to select",
                style("→").cyan(),
                style("Enter").green().bold()
            ))
            .prompt()
        {
            Ok(choice) => {
                // Handle "Always" and "Never" choices by updating config
                match choice {
                    PermissionChoice::Always => {
                        // For bash tool, save granular permission with command
                        let tool_to_save = if name == "bash" {
                            let command = args["command"].as_str().unwrap_or("");
                            // Extract base command (first word or two words for git/npm/etc)
                            let base_cmd = if command.starts_with("git ") {
                                command
                                    .split_whitespace()
                                    .take(2)
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            } else {
                                command
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or(command)
                                    .to_string()
                            };
                            format!("bash:{}", base_cmd)
                        } else {
                            name.to_string()
                        };

                        info!("Adding '{}' to allow list", tool_to_save);
                        // Load current config, modify it, and save
                        let mut updated_config = Config::load();
                        if let Err(e) = updated_config.allow_tool(&tool_to_save) {
                            error!("Failed to update config with allow list: {}", e);
                            eprintln!("{} Failed to save permission: {}", style("✗").red(), e);
                        } else {
                            eprintln!(
                                "{} Permission '{}' added to allow list in squid.config.json",
                                style("✓").green(),
                                style(&tool_to_save).cyan()
                            );
                        }
                    }
                    PermissionChoice::Never => {
                        // For bash tool, save granular permission with command
                        let tool_to_save = if name == "bash" {
                            let command = args["command"].as_str().unwrap_or("");
                            // Extract base command (first word or two words for git/npm/etc)
                            let base_cmd = if command.starts_with("git ") {
                                command
                                    .split_whitespace()
                                    .take(2)
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            } else {
                                command
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or(command)
                                    .to_string()
                            };
                            format!("bash:{}", base_cmd)
                        } else {
                            name.to_string()
                        };

                        info!("Adding '{}' to deny list", tool_to_save);
                        // Load current config, modify it, and save
                        let mut updated_config = Config::load();
                        if let Err(e) = updated_config.deny_tool(&tool_to_save) {
                            error!("Failed to update config with deny list: {}", e);
                            eprintln!("{} Failed to save permission: {}", style("✗").red(), e);
                        } else {
                            eprintln!(
                                "{} Permission '{}' added to deny list in squid.config.json",
                                style("✓").green(),
                                style(&tool_to_save).cyan()
                            );
                        }
                    }
                    _ => {}
                }
                choice
            }
            Err(e) => {
                error!("Failed to get user approval: {}", e);
                return json!({"error": format!("Failed to get user approval: {}", e)});
            }
        }
    };

    // Execute tool based on permission
    match permission {
        PermissionChoice::Yes | PermissionChoice::Always => {
            // User approved, proceed with tool execution
            match name {
                "read_file" => {
                    let validated_path = validated_path.unwrap();

                    match std::fs::read_to_string(&validated_path) {
                        Ok(content) => {
                            info!(
                                "Successfully read file: {} ({} bytes)",
                                validated_path.display(),
                                content.len()
                            );
                            json!({"content": content})
                        }
                        Err(e) => {
                            warn!("Failed to read file {}: {}", validated_path.display(), e);
                            json!({"error": format!("Failed to read file: {}", e)})
                        }
                    }
                }
                "write_file" => {
                    let validated_path = validated_path.unwrap();
                    let content = args["content"].as_str().unwrap_or("");

                    match std::fs::write(&validated_path, content) {
                        Ok(_) => {
                            info!(
                                "Successfully wrote file: {} ({} bytes)",
                                validated_path.display(),
                                content.len()
                            );
                            json!({"success": true, "message": format!("File written successfully: {}", validated_path.display())})
                        }
                        Err(e) => {
                            warn!("Failed to write file {}: {}", validated_path.display(), e);
                            json!({"error": format!("Failed to write file: {}", e)})
                        }
                    }
                }
                "grep" => {
                    let validated_path = validated_path.unwrap();
                    let pattern = args["pattern"].as_str().unwrap_or("");
                    let case_sensitive = args["case_sensitive"].as_bool().unwrap_or(false);
                    let max_results = args["max_results"].as_i64().unwrap_or(50) as usize;

                    match execute_grep(
                        pattern,
                        validated_path.to_str().unwrap_or(""),
                        case_sensitive,
                        max_results,
                    ) {
                        Ok(results) => {
                            info!(
                                "Grep found {} results for pattern '{}' in {}",
                                results.len(),
                                pattern,
                                validated_path.display()
                            );

                            // Format results as readable text for better LLM comprehension
                            if results.is_empty() {
                                json!({"message": format!("No matches found for pattern '{}' in {}", pattern, validated_path.display())})
                            } else {
                                let mut formatted_results = format!(
                                    "Found {} match{} for pattern '{}' in {}:\n\n",
                                    results.len(),
                                    if results.len() == 1 { "" } else { "es" },
                                    pattern,
                                    validated_path.display()
                                );

                                for result in &results {
                                    let file = result["file"].as_str().unwrap_or("?");
                                    let line = result["line"].as_i64().unwrap_or(0);
                                    let content = result["content"].as_str().unwrap_or("");

                                    formatted_results.push_str(&format!(
                                        "  - {}:{} — {}\n",
                                        file,
                                        line,
                                        content.trim()
                                    ));
                                }

                                info!(
                                    "Grep result preview: First match at {}:{}",
                                    results[0]["file"].as_str().unwrap_or("?"),
                                    results[0]["line"].as_i64().unwrap_or(0)
                                );

                                json!({"content": formatted_results})
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Grep failed for pattern '{}' in {}: {}",
                                pattern,
                                validated_path.display(),
                                e
                            );
                            json!({"error": format!("Grep failed: {}", e)})
                        }
                    }
                }
                "now" => {
                    let timezone = args["timezone"].as_str().unwrap_or("local");

                    let datetime_str = match timezone {
                        "utc" => Utc::now().to_rfc3339(),
                        _ => Local::now().to_rfc3339(), // Default to local
                    };

                    info!(
                        "Returning current datetime: {} ({})",
                        datetime_str, timezone
                    );
                    json!({"content": format!("The current datetime is {}.", datetime_str)})
                }
                "bash" => {
                    let command = args["command"].as_str().unwrap_or("");
                    let timeout_secs = args["timeout"].as_u64().unwrap_or(10);

                    // Note: Dangerous command check already performed at the top of call_tool()
                    // Execute the command
                    match execute_bash(command, timeout_secs).await {
                        Ok(output) => {
                            info!("Bash command executed successfully: {}", command);
                            json!({"content": format!("Command executed successfully:\n\n{}", output)})
                        }
                        Err(e) => {
                            warn!("Bash command failed: {}: {}", command, e);
                            json!({"error": format!("Command failed: {}", e)})
                        }
                    }
                }
                _ => {
                    warn!("Unknown tool called: {}", name);
                    json!({"error": format!("Unknown tool: {}", name)})
                }
            }
        }
        PermissionChoice::No | PermissionChoice::Never => {
            // User declined
            info!("Tool execution declined by user: {}", name);
            json!({"error": "Tool execution declined by user", "skipped": true})
        }
    }
}
