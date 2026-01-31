use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use inquire::Confirm;
use log::{error, info, warn};
use serde_json::json;

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
    ]
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

    // Ask for user approval
    let approval_message = match name {
        "read_file" => {
            let path = args["path"].as_str().unwrap_or("unknown");
            format!("Allow reading file: {}?", path)
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
                "Allow writing to file: {}?\nContent preview:\n{}",
                path, preview
            )
        }
        _ => format!("Allow executing tool: {}?", name),
    };

    let approved = Confirm::new(&approval_message)
        .with_default(false)
        .with_help_message("Press Y to allow, N to skip")
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
