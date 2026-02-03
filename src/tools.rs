use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use console::style;
use inquire::Confirm;
use log::{error, info, warn};
use regex::Regex;
use serde_json::json;
use walkdir::WalkDir;

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
    ]
}

/// Search a single file for pattern matches
fn search_file(
    path: &std::path::Path,
    regex: &Regex,
    results: &mut Vec<serde_json::Value>,
    max_results: usize,
) -> Result<(), String> {
    if results.len() >= max_results {
        return Ok(());
    }

    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    for (line_num, line) in content.lines().enumerate() {
        if results.len() >= max_results {
            break;
        }

        if regex.is_match(line) {
            results.push(json!({
                "file": path.display().to_string(),
                "line": line_num + 1,
                "content": line.trim()
            }));
        }
    }

    Ok(())
}

/// Execute grep search for a pattern in files
fn execute_grep(
    pattern: &str,
    path: &str,
    case_sensitive: bool,
    max_results: usize,
) -> Result<Vec<serde_json::Value>, String> {
    // Compile regex pattern
    let regex = if case_sensitive {
        Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?
    } else {
        Regex::new(&format!("(?i){}", pattern))
            .map_err(|e| format!("Invalid regex pattern: {}", e))?
    };

    let mut results = Vec::new();
    let path_obj = std::path::Path::new(path);

    if !path_obj.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // Determine if we're searching a file or directory
    if path_obj.is_file() {
        search_file(&path_obj, &regex, &mut results, max_results)?;
    } else {
        // Search recursively in directory
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if results.len() >= max_results {
                break;
            }

            if entry.file_type().is_file() {
                // Skip binary files and common non-text files
                if let Some(ext) = entry.path().extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if matches!(
                        ext_str.as_str(),
                        "jpg"
                            | "jpeg"
                            | "png"
                            | "gif"
                            | "bmp"
                            | "ico"
                            | "pdf"
                            | "zip"
                            | "tar"
                            | "gz"
                            | "exe"
                            | "dll"
                            | "so"
                            | "dylib"
                            | "bin"
                            | "dat"
                    ) {
                        continue;
                    }
                }

                if let Err(e) = search_file(entry.path(), &regex, &mut results, max_results) {
                    // Log error but continue searching other files
                    warn!("Error searching {}: {}", entry.path().display(), e);
                }
            }
        }
    }

    Ok(results)
}

/// Execute a tool call with user approval
pub async fn call_tool(name: &str, args: &str) -> serde_json::Value {
    info!("Tool call: {} with args: {}", name, args);

    // Parse arguments first to display them to the user
    let args: serde_json::Value = match args.parse() {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse tool arguments: {}", e);
            return json!({"error": format!("Invalid arguments: {}", e)});
        }
    };

    // Ask for user approval with styled formatting
    let approval_message = match name {
        "read_file" => {
            let path = args["path"].as_str().unwrap_or("unknown");
            format!(
                "🦑 Can I {}?\n  📄 File: {}",
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
                "🦑 Can I {}?\n  📄 File: {}\n  📝 Content preview:\n{}",
                style("write to this file").yellow(),
                style(path).green(),
                style(&preview).dim()
            )
        }
        "grep" => {
            let pattern = args["pattern"].as_str().unwrap_or("unknown");
            let path = args["path"].as_str().unwrap_or("unknown");
            format!(
                "🦑 Can I {}?\n  🔍 Pattern: {}\n  📂 Path: {}",
                style("search for this pattern").yellow(),
                style(pattern).magenta(),
                style(path).green()
            )
        }
        _ => format!("🦑 Can I execute: {}?", style(name).yellow()),
    };

    let approved = Confirm::new(&approval_message)
        .with_default(false)
        .with_help_message(&format!(
            "{} {} to allow, {} to deny",
            style("→").cyan(),
            style("Y").green().bold(),
            style("N").red().bold()
        ))
        .prompt();

    match approved {
        Ok(true) => {
            // User approved, proceed with tool execution
            match name {
                "read_file" => {
                    let path = args["path"].as_str().unwrap_or("");
                    match std::fs::read_to_string(path) {
                        Ok(content) => {
                            info!("Successfully read file: {} ({} bytes)", path, content.len());
                            json!({"content": content})
                        }
                        Err(e) => {
                            warn!("Failed to read file {}: {}", path, e);
                            json!({"error": format!("Failed to read file: {}", e)})
                        }
                    }
                }
                "write_file" => {
                    let path = args["path"].as_str().unwrap_or("");
                    let content = args["content"].as_str().unwrap_or("");

                    match std::fs::write(path, content) {
                        Ok(_) => {
                            info!(
                                "Successfully wrote file: {} ({} bytes)",
                                path,
                                content.len()
                            );
                            json!({"success": true, "message": format!("File written successfully: {}", path)})
                        }
                        Err(e) => {
                            warn!("Failed to write file {}: {}", path, e);
                            json!({"error": format!("Failed to write file: {}", e)})
                        }
                    }
                }
                "grep" => {
                    let pattern = args["pattern"].as_str().unwrap_or("");
                    let path = args["path"].as_str().unwrap_or("");
                    let case_sensitive = args["case_sensitive"].as_bool().unwrap_or(false);
                    let max_results = args["max_results"].as_i64().unwrap_or(50) as usize;

                    match execute_grep(pattern, path, case_sensitive, max_results) {
                        Ok(results) => {
                            info!(
                                "Grep found {} results for pattern '{}' in {}",
                                results.len(),
                                pattern,
                                path
                            );

                            // Format results as readable text for better LLM comprehension
                            if results.is_empty() {
                                json!({"message": format!("No matches found for pattern '{}' in {}", pattern, path)})
                            } else {
                                let mut formatted_results = format!(
                                    "Found {} match{} for pattern '{}' in {}:\n\n",
                                    results.len(),
                                    if results.len() == 1 { "" } else { "es" },
                                    pattern,
                                    path
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
                            warn!("Grep failed for pattern '{}' in {}: {}", pattern, path, e);
                            json!({"error": format!("Grep failed: {}", e)})
                        }
                    }
                }
                _ => {
                    warn!("Unknown tool called: {}", name);
                    json!({"error": format!("Unknown tool: {}", name)})
                }
            }
        }
        Ok(false) => {
            // User declined
            info!("Tool execution skipped by user: {}", name);
            json!({"error": "Tool execution declined by user", "skipped": true})
        }
        Err(e) => {
            // Error in prompt (e.g., non-interactive terminal)
            error!("Failed to get user approval: {}", e);
            json!({"error": format!("Failed to get user approval: {}", e)})
        }
    }
}
