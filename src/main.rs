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
use log::{debug, error, info};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

mod config;
mod logger;
mod tools;

const ASK_PROMPT: &str = include_str!("./assets/ask-prompt.md");
const CODE_REVIEW_PROMPT: &str = include_str!("./assets/code-review.md");
const CODE_REVIEW_RUST_PROMPT: &str = include_str!("./assets/review-rust.md");
const CODE_REVIEW_TYPESCRIPT_PROMPT: &str = include_str!("./assets/review-typescript.md");
const CODE_REVIEW_HTML_PROMPT: &str = include_str!("./assets/review-html.md");
const CODE_REVIEW_CSS_PROMPT: &str = include_str!("./assets/review-css.md");

fn get_review_prompt_for_file(file_path: &Path) -> &'static str {
    if let Some(extension) = file_path.extension() {
        match extension.to_str() {
            Some("rs") => CODE_REVIEW_RUST_PROMPT,
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => {
                CODE_REVIEW_TYPESCRIPT_PROMPT
            }
            Some("html") | Some("htm") => CODE_REVIEW_HTML_PROMPT,
            Some("css") | Some("scss") | Some("sass") | Some("less") => CODE_REVIEW_CSS_PROMPT,
            _ => CODE_REVIEW_PROMPT,
        }
    } else {
        CODE_REVIEW_PROMPT
    }
}

async fn ask_llm_streaming(
    question: &str,
    file_content: Option<&str>,
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
        format!(
            "Here is the content of the file:\n\n```\n{}\n```\n\nQuestion: {}",
            content, question
        )
    } else {
        question.to_string()
    };

    let system_message = system_prompt.unwrap_or(ASK_PROMPT);

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

    let mut stream = client.chat().create_stream(request).await?;
    let mut tool_calls: Vec<ChatCompletionMessageToolCall> = Vec::new();
    let mut execution_handles = Vec::new();
    let mut lock = io::stdout().lock();
    writeln!(lock)?;

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
                write!(lock, "{}", content)?;
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
                for tool_call in tool_calls.iter() {
                    let name = tool_call.function.name.clone();
                    let args = tool_call.function.arguments.clone();
                    let tool_call_id = tool_call.id.clone();

                    let handle = tokio::spawn(async move {
                        let result: serde_json::Value = tools::call_tool(&name, &args).await;
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
                    write!(lock, "{}", content)?;
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
        format!(
            "Here is the content of the file:\n\n```\n{}\n```\n\nQuestion: {}",
            content, question
        )
    } else {
        question.to_string()
    };

    let system_message = system_prompt.unwrap_or(ASK_PROMPT);

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

                let handle = tokio::spawn(async move {
                    let result: serde_json::Value = tools::call_tool(&name, &args).await;
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

            let default_config = config::Config::default();

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
                match inquire::Select::new(
                    "Log Level:",
                    vec!["error", "warn", "info", "debug", "trace"],
                )
                .with_help_message("Logging verbosity (info is recommended)")
                .with_starting_cursor(2) // Default to "info"
                .prompt()
                {
                    Ok(level) => level.to_string(),
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            let config = config::Config {
                api_url: final_url,
                api_model: final_model,
                api_key: final_api_key,
                log_level: final_log_level,
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
                match std::fs::read_to_string(file_path) {
                    Ok(content) => {
                        info!("Read file content ({} bytes)", content.len());
                        Some(content)
                    }
                    Err(e) => {
                        error!("Failed to read file: {}", e);
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
                        error!("Failed to read custom prompt file: {}", e);
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
                    custom_prompt.as_deref(),
                    &app_config,
                )
                .await
                {
                    Ok(response) => {
                        println!("\n{}", response);
                    }
                    Err(e) => {
                        error!("Failed to get response: {}", e);
                    }
                }
            } else {
                if let Err(e) = ask_llm_streaming(
                    &full_question,
                    file_content.as_deref(),
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

            let file_content = match std::fs::read_to_string(file) {
                Ok(content) => {
                    info!("Read file content ({} bytes)", content.len());
                    content
                }
                Err(e) => {
                    error!("Failed to read file: {}", e);
                    return;
                }
            };

            let review_prompt = get_review_prompt_for_file(file);
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
                    Some(review_prompt),
                    &app_config,
                )
                .await
                {
                    Ok(response) => {
                        println!("\n{}", response);
                    }
                    Err(e) => {
                        error!("Failed to get review: {}", e);
                    }
                }
            } else {
                if let Err(e) = ask_llm_streaming(
                    &question,
                    Some(&file_content),
                    Some(review_prompt),
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
