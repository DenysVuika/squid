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
) -> Result<(), Box<dyn std::error::Error>> {
    let api_url =
        std::env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:1234/v1".to_string());
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "not-needed".to_string());
    let api_model = std::env::var("API_MODEL").unwrap_or_else(|_| "local-model".to_string());

    debug!("Using API URL: {}", api_url);
    debug!("Using API Model: {}", api_model);

    let config = OpenAIConfig::new()
        .with_api_base(api_url)
        .with_api_key(api_key);

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
        .model(&api_model)
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
            .model(&api_model)
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
) -> Result<String, Box<dyn std::error::Error>> {
    let api_url =
        std::env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:1234/v1".to_string());
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "not-needed".to_string());
    let api_model = std::env::var("API_MODEL").unwrap_or_else(|_| "local-model".to_string());

    debug!("Using API URL: {}", api_url);
    debug!("Using API Model: {}", api_model);

    let config = OpenAIConfig::new()
        .with_api_base(api_url)
        .with_api_key(api_key);

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
        .model(&api_model)
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

        let subsequent_request = CreateChatCompletionRequestArgs::default()
            .model(&api_model)
            .messages(messages)
            .build()?;

        let final_response = client.chat().create(subsequent_request).await?;

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
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init,
    /// Run a specific command
    Run {
        /// The command to run
        command: String,
    },
    /// Ask a question to the LLM
    Ask {
        /// The question to ask
        question: String,
        /// Optional additional context or instructions
        #[arg(short, long)]
        message: Option<String>,
        /// Stream the response
        #[arg(short, long)]
        stream: bool,
        /// Optional file to provide context
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Review code from a file
    Review {
        /// Path to the file to review
        file: PathBuf,
        /// Optional additional message or specific question about the code
        #[arg(short, long)]
        message: Option<String>,
        /// Stream the response
        #[arg(short, long)]
        stream: bool,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            info!("Initializing project...");
        }
        Commands::Run { command } => {
            info!("Running command: {}", command);
            debug!("This is a debug message while running the command.");
            if command == "fail" {
                error!("An error occurred while executing the command.");
            }
        }
        Commands::Ask {
            question,
            message,
            stream,
            file,
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

            if *stream {
                if let Err(e) =
                    ask_llm_streaming(&full_question, file_content.as_deref(), None).await
                {
                    error!("Failed to get response: {}", e);
                }
            } else {
                match ask_llm(&full_question, file_content.as_deref(), None).await {
                    Ok(response) => {
                        println!("\n{}", response);
                    }
                    Err(e) => {
                        error!("Failed to get response: {}", e);
                    }
                }
            }
        }
        Commands::Review {
            file,
            message,
            stream,
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

            if *stream {
                if let Err(e) =
                    ask_llm_streaming(&question, Some(&file_content), Some(review_prompt)).await
                {
                    error!("Failed to get review: {}", e);
                }
            } else {
                match ask_llm(&question, Some(&file_content), Some(review_prompt)).await {
                    Ok(response) => {
                        println!("\n{}", response);
                    }
                    Err(e) => {
                        error!("Failed to get review: {}", e);
                    }
                }
            }
        }
    }
}
