use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionMessageToolCall, ChatCompletionMessageToolCalls,
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestToolMessage,
        ChatCompletionRequestUserMessage, ChatCompletionStreamOptions,
        CreateChatCompletionRequestArgs, FinishReason,
    },
};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info, warn};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

mod config;
mod logger;
mod tools;
mod validate;

const PERSONA: &str = include_str!("./assets/persona.md");
const TOOLS: &str = include_str!("./assets/tools.md");
const ASK_PROMPT: &str = include_str!("./assets/ask-prompt.md");
const CODE_REVIEW_PROMPT: &str = include_str!("./assets/code-review.md");
const CODE_REVIEW_RUST_PROMPT: &str = include_str!("./assets/review-rust.md");
const CODE_REVIEW_TYPESCRIPT_PROMPT: &str = include_str!("./assets/review-typescript.md");
const CODE_REVIEW_HTML_PROMPT: &str = include_str!("./assets/review-html.md");
const CODE_REVIEW_CSS_PROMPT: &str = include_str!("./assets/review-css.md");
const CODE_REVIEW_PYTHON_PROMPT: &str = include_str!("./assets/review-py.md");
const CODE_REVIEW_SQL_PROMPT: &str = include_str!("./assets/review-sql.md");
const CODE_REVIEW_SHELL_PROMPT: &str = include_str!("./assets/review-sh.md");
const SQUIDIGNORE_TEMPLATE: &str = include_str!("../.squidignore.example");

fn combine_prompts(task_prompt: &str) -> String {
    format!("{}\n\n{}\n\n{}", PERSONA, TOOLS, task_prompt)
}

fn get_review_prompt_for_file(file_path: &Path) -> &'static str {
    if let Some(extension) = file_path.extension() {
        match extension.to_str() {
            Some("rs") => CODE_REVIEW_RUST_PROMPT,
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => {
                CODE_REVIEW_TYPESCRIPT_PROMPT
            }
            Some("html") | Some("htm") => CODE_REVIEW_HTML_PROMPT,
            Some("css") | Some("scss") | Some("sass") | Some("less") => CODE_REVIEW_CSS_PROMPT,
            Some("py") | Some("pyw") | Some("pyi") => CODE_REVIEW_PYTHON_PROMPT,
            Some("sql") | Some("ddl") | Some("dml") => CODE_REVIEW_SQL_PROMPT,
            Some("sh") | Some("bash") | Some("zsh") | Some("fish") => CODE_REVIEW_SHELL_PROMPT,
            _ => CODE_REVIEW_PROMPT,
        }
    } else {
        CODE_REVIEW_PROMPT
    }
}

async fn ask_llm_streaming(
    question: &str,
    file_content: Option<&str>,
    file_path: Option<&str>,
    system_prompt: Option<&str>,
    app_config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Using API URL: {}", app_config.api_url);
    debug!("Using API Model: {}", app_config.api_model);

    let config = OpenAIConfig::new()
        .with_api_base(&app_config.api_url)
        .with_api_key(app_config.get_api_key());

    let client = Client::with_config(config);

    let user_message = if let Some(content) = file_content {
        let file_info = if let Some(path) = file_path {
            format!("the file '{}'", path)
        } else {
            "the file".to_string()
        };
        format!(
            "Here is the content of {}:\n\n```\n{}\n```\n\nQuestion: {}",
            file_info, content, question
        )
    } else {
        question.to_string()
    };

    let default_prompt = combine_prompts(ASK_PROMPT);
    let system_message = system_prompt.unwrap_or(&default_prompt);

    let initial_messages = vec![
        ChatCompletionRequestSystemMessage {
            content: system_message.to_string().into(),
            ..Default::default()
        }
        .into(),
        ChatCompletionRequestUserMessage {
            content: user_message.into(),
            ..Default::default()
        }
        .into(),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model(&app_config.api_model)
        .messages(initial_messages.clone())
        .tools(tools::get_tools())
        .stream_options(ChatCompletionStreamOptions {
            include_usage: Some(true),
            include_obfuscation: None,
        })
        .build()?;

    debug!("Sending streaming request...");

    // Show spinner while waiting for the first response
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Waiting for squid...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut stream = client.chat().create_stream(request).await?;
    let mut tool_calls: Vec<ChatCompletionMessageToolCall> = Vec::new();
    let mut execution_handles = Vec::new();
    let mut lock = io::stdout().lock();
    let mut first_content = true;
    let mut spinner_active = true;

    while let Some(result) = stream.next().await {
        let response = result?;

        // Log token usage statistics from streaming response (only present in final chunk)
        if let Some(usage) = &response.usage {
            writeln!(lock)?; // Add newline before logging token stats
            info!(
                "Token usage - Prompt: {}, Completion: {}, Total: {}",
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
            );

            if let Some(prompt_details) = &usage.prompt_tokens_details {
                if let Some(cached) = prompt_details.cached_tokens {
                    info!("Cached tokens: {}", cached);
                }
            }
        }

        for choice in response.choices {
            if let Some(content) = &choice.delta.content {
                // Clear spinner and write prompt on first content
                if spinner_active {
                    spinner.finish_and_clear();
                    writeln!(lock)?;
                    write!(lock, "🦑: ")?;
                    spinner_active = false;
                }

                let content_to_write = if first_content {
                    first_content = false;
                    content.trim_start()
                } else {
                    content.as_str()
                };
                write!(lock, "{}", content_to_write)?;
            }

            if let Some(tool_call_chunks) = choice.delta.tool_calls {
                for chunk in tool_call_chunks {
                    let index = chunk.index as usize;

                    while tool_calls.len() <= index {
                        tool_calls.push(ChatCompletionMessageToolCall {
                            id: String::new(),
                            function: Default::default(),
                        });
                    }

                    let tool_call = &mut tool_calls[index];
                    if let Some(id) = chunk.id {
                        tool_call.id = id;
                    }
                    if let Some(function_chunk) = chunk.function {
                        if let Some(name) = function_chunk.name {
                            tool_call.function.name = name;
                        }
                        if let Some(arguments) = function_chunk.arguments {
                            tool_call.function.arguments.push_str(&arguments);
                        }
                    }
                }
            }

            if matches!(choice.finish_reason, Some(FinishReason::ToolCalls)) {
                // Clear spinner if still active (tool calls without content)
                if spinner_active {
                    spinner.finish_and_clear();
                    writeln!(lock)?;
                    write!(lock, "🦑: ")?;
                    spinner_active = false;
                }

                for tool_call in tool_calls.iter() {
                    let name = tool_call.function.name.clone();
                    let args = tool_call.function.arguments.clone();
                    let tool_call_id = tool_call.id.clone();

                    let config_clone = app_config.clone();
                    let handle = tokio::spawn(async move {
                        let result: serde_json::Value =
                            tools::call_tool(&name, &args, &config_clone).await;
                        (tool_call_id, result)
                    });
                    execution_handles.push(handle);
                }
            }
        }
        lock.flush()?;
    }

    if !execution_handles.is_empty() {
        let mut tool_responses = Vec::new();
        for handle in execution_handles {
            let (tool_call_id, response) = handle.await?;
            tool_responses.push((tool_call_id, response));
        }

        let mut messages: Vec<ChatCompletionRequestMessage> = initial_messages;

        let assistant_tool_calls: Vec<ChatCompletionMessageToolCalls> =
            tool_calls.iter().map(|tc| tc.clone().into()).collect();

        messages.push(
            ChatCompletionRequestAssistantMessage {
                content: None,
                tool_calls: Some(assistant_tool_calls),
                ..Default::default()
            }
            .into(),
        );

        for (tool_call_id, response) in tool_responses {
            messages.push(
                ChatCompletionRequestToolMessage {
                    content: response.to_string().into(),
                    tool_call_id,
                    ..Default::default()
                }
                .into(),
            );
        }

        let follow_up_request = CreateChatCompletionRequestArgs::default()
            .model(&app_config.api_model)
            .messages(messages)
            .stream_options(ChatCompletionStreamOptions {
                include_usage: Some(true),
                include_obfuscation: None,
            })
            .build()?;

        let mut follow_up_stream = client.chat().create_stream(follow_up_request).await?;
        let mut first_followup_content = true;

        while let Some(result) = follow_up_stream.next().await {
            let response = result?;

            // Log token usage statistics from follow-up streaming response (only present in final chunk)
            if let Some(usage) = &response.usage {
                writeln!(lock)?; // Add newline before logging token stats
                info!(
                    "Follow-up token usage - Prompt: {}, Completion: {}, Total: {}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                );

                if let Some(prompt_details) = &usage.prompt_tokens_details {
                    if let Some(cached) = prompt_details.cached_tokens {
                        info!("Follow-up cached tokens: {}", cached);
                    }
                }
            }

            for choice in response.choices {
                if let Some(content) = &choice.delta.content {
                    let content_to_write = if first_followup_content {
                        first_followup_content = false;
                        content.trim_start()
                    } else {
                        content.as_str()
                    };
                    write!(lock, "{}", content_to_write)?;
                }
            }
            lock.flush()?;
        }
    }

    writeln!(lock)?;
    Ok(())
}

async fn ask_llm(
    question: &str,
    file_content: Option<&str>,
    file_path: Option<&str>,
    system_prompt: Option<&str>,
    app_config: &config::Config,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Using API URL: {}", app_config.api_url);
    debug!("Using API Model: {}", app_config.api_model);

    let config = OpenAIConfig::new()
        .with_api_base(&app_config.api_url)
        .with_api_key(app_config.get_api_key());

    let client = Client::with_config(config);

    let user_message = if let Some(content) = file_content {
        let file_info = if let Some(path) = file_path {
            format!("the file '{}'", path)
        } else {
            "the file".to_string()
        };
        format!(
            "Here is the content of {}:\n\n```\n{}\n```\n\nQuestion: {}",
            file_info, content, question
        )
    } else {
        question.to_string()
    };

    let default_prompt = combine_prompts(ASK_PROMPT);
    let system_message = system_prompt.unwrap_or(&default_prompt);

    let initial_messages = vec![
        ChatCompletionRequestSystemMessage {
            content: system_message.to_string().into(),
            ..Default::default()
        }
        .into(),
        ChatCompletionRequestUserMessage {
            content: user_message.into(),
            ..Default::default()
        }
        .into(),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model(&app_config.api_model)
        .messages(initial_messages.clone())
        .tools(tools::get_tools())
        .build()?;

    debug!("Sending request...");

    let response = client.chat().create(request).await?;

    // Log token usage statistics
    if let Some(usage) = &response.usage {
        info!(
            "Token usage - Prompt: {}, Completion: {}, Total: {}",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );

        if let Some(prompt_details) = &usage.prompt_tokens_details {
            if let Some(cached) = prompt_details.cached_tokens {
                info!("Cached tokens: {}", cached);
            }
        }
    }
    let response_message = response
        .choices
        .first()
        .ok_or("No response from LLM")?
        .message
        .clone();

    if let Some(tool_calls) = response_message.tool_calls {
        let mut handles = Vec::new();
        for tool_call in &tool_calls {
            if let ChatCompletionMessageToolCalls::Function(tc) = tool_call {
                let name = tc.function.name.clone();
                let args = tc.function.arguments.clone();
                let tool_call_clone = tool_call.clone();

                let config_clone = app_config.clone();
                let handle = tokio::spawn(async move {
                    let result: serde_json::Value =
                        tools::call_tool(&name, &args, &config_clone).await;
                    (tool_call_clone, result)
                });
                handles.push(handle);
            }
        }

        let mut function_responses = Vec::new();
        for handle in handles {
            let (tool_call, response_content): (ChatCompletionMessageToolCalls, serde_json::Value) =
                handle.await?;
            function_responses.push((tool_call, response_content));
        }

        let mut messages: Vec<ChatCompletionRequestMessage> = initial_messages;

        let assistant_tool_calls: Vec<ChatCompletionMessageToolCalls> = function_responses
            .iter()
            .map(|(tool_call, _)| tool_call.clone())
            .collect();

        messages.push(
            ChatCompletionRequestAssistantMessage {
                content: None,
                tool_calls: Some(assistant_tool_calls),
                ..Default::default()
            }
            .into(),
        );

        for (tool_call, response_content) in function_responses {
            if let ChatCompletionMessageToolCalls::Function(tc) = &tool_call {
                messages.push(
                    ChatCompletionRequestToolMessage {
                        content: response_content.to_string().into(),
                        tool_call_id: tc.id.clone(),
                        ..Default::default()
                    }
                    .into(),
                );
            }
        }

        let follow_up_request = CreateChatCompletionRequestArgs::default()
            .model(&app_config.api_model)
            .messages(messages)
            .build()?;

        let final_response = client.chat().create(follow_up_request).await?;

        // Log token usage statistics for follow-up request
        if let Some(usage) = &final_response.usage {
            info!(
                "Follow-up token usage - Prompt: {}, Completion: {}, Total: {}",
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
            );

            if let Some(prompt_details) = &usage.prompt_tokens_details {
                if let Some(cached) = prompt_details.cached_tokens {
                    info!("Follow-up cached tokens: {}", cached);
                }
            }
        }

        let answer = final_response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or("No response from LLM")?;

        return Ok(answer.to_string());
    }

    let answer = response_message.content.ok_or("No response from LLM")?;

    Ok(answer)
}

#[derive(Parser)]
#[command(name = "squid")]
#[command(about = "squid 🦑: An AI-powered command-line tool for code reviews and suggestions.", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        /// Directory to initialize (defaults to current directory)
        #[arg(default_value = ".")]
        dir: PathBuf,
        /// API URL (skips interactive prompt if provided)
        #[arg(long)]
        url: Option<String>,
        /// API Model (skips interactive prompt if provided)
        #[arg(long)]
        model: Option<String>,
        /// API Key (skips interactive prompt if provided)
        #[arg(long)]
        api_key: Option<String>,
        /// Log Level (skips interactive prompt if provided)
        #[arg(long)]
        log_level: Option<String>,
    },
    /// Ask a question to the LLM
    Ask {
        /// The question to ask
        question: String,
        /// Optional additional context or instructions
        #[arg(short, long)]
        message: Option<String>,
        /// Disable streaming (return complete response at once)
        #[arg(long)]
        no_stream: bool,
        /// Optional file to provide context
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Optional custom system prompt file
        #[arg(short, long)]
        prompt: Option<PathBuf>,
    },
    /// Review code from a file
    Review {
        /// Path to the file to review
        file: PathBuf,
        /// Optional additional message or specific question about the code
        #[arg(short, long)]
        message: Option<String>,
        /// Disable streaming (return complete response at once)
        #[arg(long)]
        no_stream: bool,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();

    // Load config early to initialize logger with correct log level
    // For init command, we'll use defaults since config doesn't exist yet
    let app_config = if matches!(cli.command, Commands::Init { .. }) {
        config::Config::default()
    } else {
        config::Config::load()
    };

    logger::init(Some(&app_config.log_level));

    match &cli.command {
        Commands::Init {
            dir,
            url,
            model,
            api_key,
            log_level,
        } => {
            info!("Initializing squid configuration in {:?}...", dir);

            // Create directory if it doesn't exist
            if !dir.exists() {
                if let Err(e) = std::fs::create_dir_all(dir) {
                    error!("Failed to create directory {:?}: {}", dir, e);
                    return;
                }
            }

            // Try to load existing config, otherwise use defaults
            let config_path = dir.join("squid.config.json");
            let config_existed = config_path.exists();
            let existing_config = if config_existed {
                println!("Found existing configuration, using current values as defaults...\n");
                match std::fs::read_to_string(&config_path) {
                    Ok(content) => match serde_json::from_str::<config::Config>(&content) {
                        Ok(cfg) => Some(cfg),
                        Err(e) => {
                            info!("Failed to parse existing config: {}", e);
                            None
                        }
                    },
                    Err(e) => {
                        info!("Failed to read existing config: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            let default_config = existing_config.unwrap_or_else(|| config::Config::default());

            // Use CLI args if provided, otherwise prompt interactively
            let final_url = if let Some(u) = url {
                u.clone()
            } else {
                match inquire::Text::new("API URL:")
                    .with_default(&default_config.api_url)
                    .with_help_message(
                        "The base URL for the API (e.g., http://127.0.0.1:1234/v1 for LM Studio)",
                    )
                    .prompt()
                {
                    Ok(u) => u,
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            let final_model = if let Some(m) = model {
                m.clone()
            } else {
                match inquire::Text::new("API Model:")
                    .with_default(&default_config.api_model)
                    .with_help_message("The model identifier to use")
                    .prompt()
                {
                    Ok(m) => m,
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            let final_api_key = if api_key.is_some() {
                api_key.clone()
            } else {
                match inquire::Text::new("API Key (optional, press Enter to skip):")
                    .with_help_message("API key if required (leave empty for local models)")
                    .prompt_skippable()
                {
                    Ok(key) => key.filter(|k| !k.is_empty()),
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            let final_log_level = if let Some(level) = log_level {
                level.clone()
            } else {
                // Find the index of the current log level for the cursor position
                let levels = vec!["error", "warn", "info", "debug", "trace"];
                let cursor_pos = levels
                    .iter()
                    .position(|&l| l == default_config.log_level)
                    .unwrap_or(2);

                match inquire::Select::new("Log Level:", levels)
                    .with_help_message("Logging verbosity (info is recommended)")
                    .with_starting_cursor(cursor_pos)
                    .prompt()
                {
                    Ok(level) => level.to_string(),
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            // Smart merge permissions: keep user's custom permissions + add new defaults
            let (merged_permissions, old_permissions) = if config_existed {
                let old_perms = default_config.permissions.clone();
                let allow_set: std::collections::HashSet<String> =
                    default_config.permissions.allow.iter().cloned().collect();
                let deny_set: std::collections::HashSet<String> =
                    default_config.permissions.deny.iter().cloned().collect();

                // Add new default permissions that user doesn't have
                let new_defaults = config::Permissions::default();
                let mut merged_allow = allow_set.clone();
                for default_perm in &new_defaults.allow {
                    if !allow_set.contains(default_perm) && !deny_set.contains(default_perm) {
                        merged_allow.insert(default_perm.clone());
                    }
                }

                (
                    config::Permissions {
                        allow: merged_allow.into_iter().collect(),
                        deny: deny_set.into_iter().collect(),
                    },
                    Some(old_perms),
                )
            } else {
                (default_config.permissions, None)
            };

            let config = config::Config {
                api_url: final_url,
                api_model: final_model,
                api_key: final_api_key,
                log_level: final_log_level,
                permissions: merged_permissions,
                version: None, // Will be set automatically by save_to_dir()
            };

            match config.save_to_dir(dir) {
                Ok(_) => {
                    let config_path = dir.join("squid.config.json");
                    info!("✓ Configuration saved to {:?}", config_path);
                    println!("\nConfiguration saved to: {:?}", config_path);
                    println!("  API URL: {}", config.api_url);
                    println!("  API Model: {}", config.api_model);
                    if config.api_key.is_some() {
                        println!("  API Key: [configured]");
                    } else {
                        println!("  API Key: [not set]");
                    }
                    println!("  Log Level: {}", config.log_level);

                    // Show info about permissions
                    if let Some(old_perms) = old_permissions {
                        // Check if new permissions were added
                        let old_allow_set: std::collections::HashSet<String> =
                            old_perms.allow.iter().cloned().collect();
                        let new_allow_set: std::collections::HashSet<String> =
                            config.permissions.allow.iter().cloned().collect();
                        let added_perms: Vec<_> =
                            new_allow_set.difference(&old_allow_set).collect();

                        if !added_perms.is_empty() {
                            println!("\n✓ Added new default permissions: {:?}", added_perms);
                        }

                        if !config.permissions.allow.is_empty()
                            || !config.permissions.deny.is_empty()
                        {
                            println!("\n✓ Current tool permissions:");
                            if !config.permissions.allow.is_empty() {
                                println!("  Allowed: {:?}", config.permissions.allow);
                            }
                            if !config.permissions.deny.is_empty() {
                                println!("  Denied: {:?}", config.permissions.deny);
                            }
                        }
                    } else {
                        println!("\n✓ Default permissions configured");
                        if !config.permissions.allow.is_empty() {
                            println!("  Allowed: {:?}", config.permissions.allow);
                        }
                    }

                    // Create .squidignore file if it doesn't exist
                    let squidignore_path = dir.join(".squidignore");
                    if !squidignore_path.exists() {
                        match std::fs::write(&squidignore_path, SQUIDIGNORE_TEMPLATE) {
                            Ok(_) => {
                                info!("✓ Created .squidignore file at {:?}", squidignore_path);
                                println!("\n✓ Created .squidignore with default patterns");
                                println!(
                                    "  Edit this file to customize which files squid should ignore"
                                );
                            }
                            Err(e) => {
                                warn!("Failed to create .squidignore: {}", e);
                                println!("\n⚠ Could not create .squidignore: {}", e);
                            }
                        }
                    } else {
                        info!(".squidignore already exists, skipping creation");
                        println!("\n✓ Using existing .squidignore file");
                    }
                }
                Err(e) => {
                    error!("Failed to save configuration: {}", e);
                }
            }
        }
        Commands::Ask {
            question,
            message,
            no_stream,
            file,
            prompt,
        } => {
            let full_question = if let Some(m) = message {
                format!("{} {}", question, m)
            } else {
                question.clone()
            };

            info!("Q: {}", full_question);

            let file_content = if let Some(file_path) = file {
                // Validate path before reading
                let ignore_patterns = validate::PathValidator::load_ignore_patterns();
                let validator = validate::PathValidator::with_ignore_file(Some(ignore_patterns));

                match validator.validate(file_path) {
                    Ok(_) => match std::fs::read_to_string(file_path) {
                        Ok(content) => {
                            info!("Read file content ({} bytes)", content.len());
                            Some(content)
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                println!(
                                    "🦑: I can't find that file. Please check the path and try again."
                                );
                            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                                println!("🦑: I don't have permission to read that file.");
                            } else {
                                println!("🦑: I couldn't read that file - {}", e);
                            }
                            debug!("Failed to read file {}: {}", file_path.display(), e);
                            return;
                        }
                    },
                    Err(validate::PathValidationError::PathIgnored(_)) => {
                        println!("🦑: I can't access that file - it's in your .squidignore list.");
                        return;
                    }
                    Err(validate::PathValidationError::PathNotAllowed(_)) => {
                        println!(
                            "🦑: I can't access that file - it's outside the project directory or in a protected system location."
                        );
                        return;
                    }
                    Err(e) => {
                        debug!("Path validation failed: {}", e);
                        println!("🦑: I can't access that file - {}", e);
                        return;
                    }
                }
            } else {
                None
            };

            let custom_prompt = if let Some(prompt_path) = prompt {
                match std::fs::read_to_string(prompt_path) {
                    Ok(content) => {
                        info!("Using custom system prompt ({} bytes)", content.len());
                        Some(content)
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            println!(
                                "🦑: I can't find that custom prompt file. Please check the path and try again."
                            );
                        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                            println!("🦑: I don't have permission to read that prompt file.");
                        } else {
                            println!("🦑: I couldn't read that prompt file - {}", e);
                        }
                        debug!(
                            "Failed to read custom prompt file {}: {}",
                            prompt_path.display(),
                            e
                        );
                        return;
                    }
                }
            } else {
                None
            };

            if *no_stream {
                match ask_llm(
                    &full_question,
                    file_content.as_deref(),
                    file.as_ref().and_then(|p| p.to_str()),
                    custom_prompt.as_deref(),
                    &app_config,
                )
                .await
                {
                    Ok(response) => {
                        println!("\n🦑: {}", response);
                    }
                    Err(e) => {
                        error!("Failed to get response: {}", e);
                    }
                }
            } else {
                if let Err(e) = ask_llm_streaming(
                    &full_question,
                    file_content.as_deref(),
                    file.as_ref().and_then(|p| p.to_str()),
                    custom_prompt.as_deref(),
                    &app_config,
                )
                .await
                {
                    error!("Failed to get response: {}", e);
                }
            }
        }
        Commands::Review {
            file,
            message,
            no_stream,
        } => {
            info!("Reviewing file: {:?}", file);

            // Validate path before reading
            let ignore_patterns = validate::PathValidator::load_ignore_patterns();
            let validator = validate::PathValidator::with_ignore_file(Some(ignore_patterns));

            let file_content = match validator.validate(file) {
                Ok(_) => match std::fs::read_to_string(file) {
                    Ok(content) => {
                        info!("Read file content ({} bytes)", content.len());
                        content
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            println!(
                                "🦑: I can't find that file. Please check the path and try again."
                            );
                        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                            println!("🦑: I don't have permission to read that file.");
                        } else {
                            println!("🦑: I couldn't read that file - {}", e);
                        }
                        debug!("Failed to read file {}: {}", file.display(), e);
                        return;
                    }
                },
                Err(validate::PathValidationError::PathIgnored(_)) => {
                    println!("🦑: I can't access that file - it's in your .squidignore list.");
                    return;
                }
                Err(validate::PathValidationError::PathNotAllowed(_)) => {
                    println!(
                        "🦑: I can't access that file - it's outside the project directory or in a protected system location."
                    );
                    return;
                }
                Err(e) => {
                    debug!("Path validation failed: {}", e);
                    println!("🦑: I can't access that file - {}", e);
                    return;
                }
            };

            let review_prompt = get_review_prompt_for_file(file);
            let combined_review_prompt = combine_prompts(review_prompt);
            debug!("Using review prompt for file type");

            let question = if let Some(msg) = message {
                format!("Please review this code. {}", msg)
            } else {
                "Please review this code.".to_string()
            };

            if *no_stream {
                match ask_llm(
                    &question,
                    Some(&file_content),
                    file.to_str(),
                    Some(&combined_review_prompt),
                    &app_config,
                )
                .await
                {
                    Ok(response) => {
                        println!("\n🦑: {}", response);
                    }
                    Err(e) => {
                        error!("Failed to get review: {}", e);
                    }
                }
            } else {
                if let Err(e) = ask_llm_streaming(
                    &question,
                    Some(&file_content),
                    file.to_str(),
                    Some(&combined_review_prompt),
                    &app_config,
                )
                .await
                {
                    error!("Failed to get review: {}", e);
                }
            }
        }
    }
}
