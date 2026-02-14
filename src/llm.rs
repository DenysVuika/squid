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
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use std::io::{self, Write};
use std::path::Path;

use crate::config;
use crate::envinfo;
use crate::tools;

// Prompt constants
const PERSONA: &str = include_str!("./assets/persona.md");
const TOOLS: &str = include_str!("./assets/tools.md");
const ENV_TEMPLATE: &str = include_str!("./assets/env.md");
const ASK_PROMPT: &str = include_str!("./assets/ask-prompt.md");
const CODE_REVIEW_PROMPT: &str = include_str!("./assets/code-review.md");
const CODE_REVIEW_RUST_PROMPT: &str = include_str!("./assets/review-rust.md");
const CODE_REVIEW_TYPESCRIPT_PROMPT: &str = include_str!("./assets/review-typescript.md");
const CODE_REVIEW_HTML_PROMPT: &str = include_str!("./assets/review-html.md");
const CODE_REVIEW_CSS_PROMPT: &str = include_str!("./assets/review-css.md");
const CODE_REVIEW_PYTHON_PROMPT: &str = include_str!("./assets/review-py.md");
const CODE_REVIEW_SQL_PROMPT: &str = include_str!("./assets/review-sql.md");
const CODE_REVIEW_SHELL_PROMPT: &str = include_str!("./assets/review-sh.md");
const CODE_REVIEW_DOCKER_PROMPT: &str = include_str!("./assets/review-docker.md");
const CODE_REVIEW_GO_PROMPT: &str = include_str!("./assets/review-go.md");
const CODE_REVIEW_JAVA_PROMPT: &str = include_str!("./assets/review-java.md");
const CODE_REVIEW_JSON_PROMPT: &str = include_str!("./assets/review-json.md");
const CODE_REVIEW_MAKEFILE_PROMPT: &str = include_str!("./assets/review-makefile.md");
const CODE_REVIEW_MARKDOWN_PROMPT: &str = include_str!("./assets/review-md.md");
const CODE_REVIEW_YAML_PROMPT: &str = include_str!("./assets/review-yaml.md");

/// Combines persona, tools, and task-specific prompt into a complete system prompt
pub fn combine_prompts(task_prompt: &str) -> String {
    format!("{}\n\n{}\n\n{}", PERSONA, TOOLS, task_prompt)
}

/// Composes the user message by injecting environment context and optional file content
fn compose_user_message(
    question: &str,
    file_content: Option<&str>,
    file_path: Option<&str>,
    enable_env_context: bool,
) -> String {
    let env_section = if enable_env_context {
        let env_context = envinfo::get_env_context();
        ENV_TEMPLATE.replace("{{ENV_CONTEXT}}", &env_context)
    } else {
        String::new()
    };
    
    if let Some(content) = file_content {
        let file_info = if let Some(path) = file_path {
            format!("the file '{}'", path)
        } else {
            "the file".to_string()
        };
        
        if enable_env_context {
            format!(
                "{}\n\nHere is the content of {}:\n\n```\n{}\n```\n\nUser query: {}",
                env_section, file_info, content, question
            )
        } else {
            format!(
                "Here is the content of {}:\n\n```\n{}\n```\n\nUser query: {}",
                file_info, content, question
            )
        }
    } else {
        if enable_env_context {
            format!("{}\n\nUser query: {}", env_section, question)
        } else {
            format!("User query: {}", question)
        }
    }
}

pub fn get_ask_prompt() -> &'static str {
    ASK_PROMPT
}

/// Returns the appropriate code review prompt based on file extension
pub fn get_review_prompt_for_file(file_path: &Path) -> &'static str {
    // Check for files without extensions first (Dockerfile, Makefile, etc.)
    if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
        let lower_name = file_name.to_lowercase();
        if lower_name == "dockerfile" || lower_name.starts_with("dockerfile.") {
            return CODE_REVIEW_DOCKER_PROMPT;
        }
        if lower_name == "makefile" || lower_name.starts_with("makefile.") {
            return CODE_REVIEW_MAKEFILE_PROMPT;
        }
    }

    // Check by file extension
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
            Some("go") => CODE_REVIEW_GO_PROMPT,
            Some("java") => CODE_REVIEW_JAVA_PROMPT,
            Some("json") => CODE_REVIEW_JSON_PROMPT,
            Some("yaml") | Some("yml") => CODE_REVIEW_YAML_PROMPT,
            Some("md") | Some("markdown") => CODE_REVIEW_MARKDOWN_PROMPT,
            _ => CODE_REVIEW_PROMPT,
        }
    } else {
        CODE_REVIEW_PROMPT
    }
}

/// Sends a streaming request to the LLM and handles tool calls
pub async fn ask_llm_streaming(
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

    let user_message = compose_user_message(question, file_content, file_path, app_config.enable_env_context);

    let default_prompt = combine_prompts(ASK_PROMPT);
    let system_message = system_prompt.unwrap_or(&default_prompt);

    debug!("System message:\n{}", system_message);
    debug!("User message:\n{}", user_message);

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
            debug!(
                "Token usage - Prompt: {}, Completion: {}, Total: {}",
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
            );

            if let Some(prompt_details) = &usage.prompt_tokens_details {
                if let Some(cached) = prompt_details.cached_tokens {
                    debug!("Cached tokens: {}", cached);
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
                debug!(
                    "Follow-up token usage - Prompt: {}, Completion: {}, Total: {}",
                    usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                );

                if let Some(prompt_details) = &usage.prompt_tokens_details {
                    if let Some(cached) = prompt_details.cached_tokens {
                        debug!("Follow-up cached tokens: {}", cached);
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

/// Sends a non-streaming request to the LLM and handles tool calls
pub async fn ask_llm(
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

    let user_message = compose_user_message(question, file_content, file_path, app_config.enable_env_context);

    let default_prompt = combine_prompts(ASK_PROMPT);
    let system_message = system_prompt.unwrap_or(&default_prompt);

    debug!("System message:\n{}", system_message);
    debug!("User message:\n{}", user_message);

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
        debug!(
            "Token usage - Prompt: {}, Completion: {}, Total: {}",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );

        if let Some(prompt_details) = &usage.prompt_tokens_details {
            if let Some(cached) = prompt_details.cached_tokens {
                debug!("Cached tokens: {}", cached);
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
            debug!(
                "Follow-up token usage - Prompt: {}, Completion: {}, Total: {}",
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
            );

            if let Some(prompt_details) = &usage.prompt_tokens_details {
                if let Some(cached) = prompt_details.cached_tokens {
                    debug!("Follow-up cached tokens: {}", cached);
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
