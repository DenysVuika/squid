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
use log::{debug, error, info, warn};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use crate::config;
use crate::session::{ChatSession, Source, ThinkingStep};
use crate::template;
use crate::tools;
use crate::{db, rag, validate};

// Prompt constants
const PERSONA: &str = include_str!("./assets/persona.md");
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

/// Combines persona and task-specific prompt into a complete system prompt
/// Renders templates with secure context variables
pub fn combine_prompts(task_prompt: &str) -> String {
    let renderer = template::TemplateRenderer::new();

    let persona = renderer.render_string(PERSONA).unwrap_or_else(|e| {
        log::warn!("Failed to render persona template: {}", e);
        PERSONA.to_string()
    });

    let task = renderer.render_string(task_prompt).unwrap_or_else(|e| {
        log::warn!("Failed to render task prompt template: {}", e);
        task_prompt.to_string()
    });

    format!("{}\n\n{}", persona, task)
}

/// Strip <think>...</think> blocks from content
/// Used when sending conversation history back to the model to reduce token usage.
/// The model doesn't need to see its own past reasoning to continue the conversation.
pub fn strip_reasoning_blocks(content: &str) -> String {
    let mut result = content.to_string();

    // Remove all <think>...</think> blocks
    while let Some(start) = result.find("<think>") {
        if let Some(end) = result[start..].find("</think>") {
            let absolute_end = start + end;
            // Remove the <think>...</think> section
            result = format!("{}{}", &result[..start], &result[absolute_end + 8..]);
        } else {
            // Malformed tag - no closing tag found, leave as-is
            break;
        }
    }

    // Trim any extra whitespace that might result from removing blocks
    result.trim().to_string()
}

/// Composes the user message with optional file content
/// Uses template rendering for variable substitution
fn compose_user_message(
    question: &str,
    file_content: Option<&str>,
    file_path: Option<&str>,
) -> String {
    let renderer = template::TemplateRenderer::new();

    if let Some(content) = file_content {
        let file_info = if let Some(path) = file_path {
            format!("the file '{}'", path)
        } else {
            "the file".to_string()
        };

        let template = format!(
            "Here is the content of {}:\n\n```\n{}\n```\n\nUser query: {}",
            file_info, content, question
        );

        renderer.render_string(&template).unwrap_or(template)
    } else {
        let template = format!("User query: {}", question);
        renderer.render_string(&template).unwrap_or(template)
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
/// Optionally saves the conversation to a session if session_id and db are provided
pub async fn ask_llm_streaming(
    question: &str,
    file_content: Option<&str>,
    file_path: Option<&str>,
    system_prompt: Option<&str>,
    model: &str,
    app_config: &config::Config,
    session: Option<&mut ChatSession>,
    db: Option<&db::Database>,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Using API URL: {}", app_config.api_url);
    debug!("Using Model: {}", model);

    let config = OpenAIConfig::new()
        .with_api_base(&app_config.api_url)
        .with_api_key(app_config.get_api_key());

    let client = Client::with_config(config);

    let user_message = compose_user_message(question, file_content, file_path);

    let default_prompt = combine_prompts(ASK_PROMPT);
    let system_prompt_str = system_prompt.unwrap_or(&default_prompt);

    // Render template variables in system message
    let renderer = template::TemplateRenderer::new();
    let system_message = renderer
        .render_string(system_prompt_str)
        .unwrap_or_else(|e| {
            log::warn!("Failed to render system prompt template: {}", e);
            system_prompt_str.to_string()
        });

    debug!("System message:\n{}", system_message);
    debug!("User message:\n{}", user_message);

    let initial_messages = vec![
        ChatCompletionRequestSystemMessage {
            content: system_message.into(),
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
        .model(model)
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

    // Accumulate response content and thinking steps for session saving
    let mut accumulated_content = String::new();
    let mut thinking_steps: Vec<ThinkingStep> = Vec::new();
    let mut step_order = 0i32;
    let mut total_input_tokens = 0i64;
    let mut total_output_tokens = 0i64;
    let total_reasoning_tokens = 0i64;
    let total_cache_tokens = 0i64;

    while let Some(result) = stream.next().await {
        let response = result?;

        // Log token usage statistics from streaming response (only present in final chunk)
        if let Some(usage) = &response.usage {
            writeln!(lock)?; // Add newline before logging token stats
            debug!(
                "Token usage - Prompt: {}, Completion: {}, Total: {}",
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
            );

            total_input_tokens = usage.prompt_tokens as i64;
            total_output_tokens = usage.completion_tokens as i64;
            // reasoning_tokens and cache_tokens are not directly available in CompletionUsage
            // They may be in completion_tokens_details depending on the API provider

            if let Some(prompt_details) = &usage.prompt_tokens_details
                && let Some(cached) = prompt_details.cached_tokens
            {
                debug!("Cached tokens: {}", cached);
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
                accumulated_content.push_str(content);

                // Check for <think>...</think> blocks in the content
                while let Some(think_start) = accumulated_content.find("<think>") {
                    if let Some(think_end) = accumulated_content.find("</think>") {
                        if think_end > think_start {
                            let reasoning_text =
                                accumulated_content[think_start + 7..think_end].to_string();
                            if !reasoning_text.trim().is_empty() {
                                thinking_steps.push(ThinkingStep {
                                    step_type: "reasoning".to_string(),
                                    step_order,
                                    content: Some(reasoning_text),
                                    tool_name: None,
                                    tool_arguments: None,
                                    tool_result: None,
                                    tool_error: None,
                                    content_before_tool: None,
                                });
                                step_order += 1;
                            }
                            // Remove the <think> block from accumulated content for final display
                            accumulated_content = format!(
                                "{}{}",
                                &accumulated_content[..think_start],
                                &accumulated_content[think_end + 8..]
                            );
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
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
                            tool_call
                                .function
                                .arguments
                                .push_str(&arguments.to_string());
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
                            tools::call_tool(&name, &args, None, &config_clone).await;
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
                }
                .into(),
            );
        }

        let follow_up_request = CreateChatCompletionRequestArgs::default()
            .model(model)
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

                total_input_tokens += usage.prompt_tokens as i64;
                total_output_tokens += usage.completion_tokens as i64;
                // reasoning_tokens and cache_tokens are not directly available in CompletionUsage

                if let Some(prompt_details) = &usage.prompt_tokens_details
                    && let Some(cached) = prompt_details.cached_tokens
                {
                    debug!("Follow-up cached tokens: {}", cached);
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
                    accumulated_content.push_str(content);
                }
            }
            lock.flush()?;
        }
    }

    writeln!(lock)?;

    // Save to session if provided
    if let Some(sess) = session
        && let Some(database) = db
    {
        // Save session metadata FIRST (before messages, due to foreign key constraint)
        if sess.title.is_none() {
            // Generate title from first user message
            let title = if question.len() > 100 {
                format!("{}...", &question[..97])
            } else {
                question.to_string()
            };
            sess.title = Some(title);
        }

        if let Err(e) = database.save_session(sess) {
            debug!("Failed to save session: {}", e);
        } else {
            debug!("Session saved successfully: {}", sess.id);
        }

        // Save user message
        let user_msg = crate::session::ChatMessage {
            role: "user".to_string(),
            content: question.to_string(),
            sources: if let Some(path) = file_path {
                if let Some(content) = file_content {
                    vec![Source {
                        title: path.to_string(),
                        content: content.to_string(),
                    }]
                } else {
                    vec![]
                }
            } else {
                vec![]
            },
            timestamp: chrono::Utc::now().timestamp(),
            thinking_steps: None,
        };

        if let Err(e) = database.save_message(&sess.id, &user_msg) {
            debug!("Failed to save user message to session: {}", e);
        } else {
            debug!("User message saved successfully to session {}", sess.id);
        }

        // Save assistant message
        let thinking_steps_opt = if thinking_steps.is_empty() {
            None
        } else {
            Some(thinking_steps)
        };
        let assistant_msg = crate::session::ChatMessage {
            role: "assistant".to_string(),
            content: accumulated_content.trim().to_string(),
            sources: vec![],
            timestamp: chrono::Utc::now().timestamp(),
            thinking_steps: thinking_steps_opt,
        };

        if let Err(e) = database.save_message(&sess.id, &assistant_msg) {
            debug!("Failed to save assistant message to session: {}", e);
        } else {
            debug!(
                "Assistant message saved successfully to session {}",
                sess.id
            );
        }

        // Update session token usage
        sess.add_tokens(
            total_input_tokens,
            total_output_tokens,
            total_reasoning_tokens,
            total_cache_tokens,
        );
        if let Err(e) = database.save_session(sess) {
            debug!("Failed to update session: {}", e);
        }
    }

    Ok(accumulated_content.trim().to_string())
}

/// Sends a non-streaming request to the LLM and handles tool calls
/// Optionally saves the conversation to a session if session_id and db are provided
pub async fn ask_llm(
    question: &str,
    file_content: Option<&str>,
    file_path: Option<&str>,
    system_prompt: Option<&str>,
    model: &str,
    app_config: &config::Config,
    session: Option<&mut ChatSession>,
    db: Option<&db::Database>,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Using API URL: {}", app_config.api_url);
    debug!("Using Model: {}", model);

    let config = OpenAIConfig::new()
        .with_api_base(&app_config.api_url)
        .with_api_key(app_config.get_api_key());

    let client = Client::with_config(config);

    let user_message = compose_user_message(question, file_content, file_path);

    let default_prompt = combine_prompts(ASK_PROMPT);
    let system_prompt_str = system_prompt.unwrap_or(&default_prompt);

    // Render template variables in system message
    let renderer = template::TemplateRenderer::new();
    let system_message = renderer
        .render_string(system_prompt_str)
        .unwrap_or_else(|e| {
            log::warn!("Failed to render system prompt template: {}", e);
            system_prompt_str.to_string()
        });

    debug!("System message:\n{}", system_message);
    debug!("User message:\n{}", user_message);

    let initial_messages = vec![
        ChatCompletionRequestSystemMessage {
            content: system_message.into(),
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
        .model(model)
        .messages(initial_messages.clone())
        .tools(tools::get_tools())
        .build()?;

    debug!("Sending request...");

    let response = client.chat().create(request).await?;

    // Log token usage statistics
    let mut total_input_tokens = 0i64;
    let mut total_output_tokens = 0i64;
    let total_reasoning_tokens = 0i64;
    let total_cache_tokens = 0i64;

    if let Some(usage) = &response.usage {
        debug!(
            "Token usage - Prompt: {}, Completion: {}, Total: {}",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );

        total_input_tokens = usage.prompt_tokens as i64;
        total_output_tokens = usage.completion_tokens as i64;
        // reasoning_tokens and cache_tokens are not directly available in CompletionUsage

        if let Some(prompt_details) = &usage.prompt_tokens_details
            && let Some(cached) = prompt_details.cached_tokens
        {
            debug!("Cached tokens: {}", cached);
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
                        tools::call_tool(&name, &args, None, &config_clone).await;
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
                    }
                    .into(),
                );
            }
        }

        let follow_up_request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages)
            .build()?;

        let final_response = client.chat().create(follow_up_request).await?;

        // Log token usage statistics for follow-up request
        if let Some(usage) = &final_response.usage {
            debug!(
                "Follow-up token usage - Prompt: {}, Completion: {}, Total: {}",
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
            );

            total_input_tokens += usage.prompt_tokens as i64;
            total_output_tokens += usage.completion_tokens as i64;
            // reasoning_tokens and cache_tokens are not directly available in CompletionUsage

            if let Some(prompt_details) = &usage.prompt_tokens_details
                && let Some(cached) = prompt_details.cached_tokens
            {
                debug!("Follow-up cached tokens: {}", cached);
            }
        }

        let answer = final_response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or("No response from LLM")?;

        let answer_str = answer.to_string();

        // Save to session if provided
        if let Some(sess) = session
            && let Some(database) = db
        {
            // Save session metadata FIRST (before messages, due to foreign key constraint)
            if sess.title.is_none() {
                let title = if question.len() > 100 {
                    format!("{}...", &question[..97])
                } else {
                    question.to_string()
                };
                sess.title = Some(title);
            }

            if let Err(e) = database.save_session(sess) {
                debug!("Failed to save session: {}", e);
            } else {
                debug!("Session saved successfully: {}", sess.id);
            }

            // Save user message
            let user_msg = crate::session::ChatMessage {
                role: "user".to_string(),
                content: question.to_string(),
                sources: if let Some(path) = file_path {
                    if let Some(content) = file_content {
                        vec![Source {
                            title: path.to_string(),
                            content: content.to_string(),
                        }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                },
                timestamp: chrono::Utc::now().timestamp(),
                thinking_steps: None,
            };

            if let Err(e) = database.save_message(&sess.id, &user_msg) {
                debug!("Failed to save user message to session: {}", e);
            } else {
                debug!("User message saved successfully to session {}", sess.id);
            }

            // Save assistant message (no thinking steps for non-streaming)
            let assistant_msg = crate::session::ChatMessage {
                role: "assistant".to_string(),
                content: answer_str.clone(),
                sources: vec![],
                timestamp: chrono::Utc::now().timestamp(),
                thinking_steps: None,
            };

            if let Err(e) = database.save_message(&sess.id, &assistant_msg) {
                debug!("Failed to save assistant message to session: {}", e);
            } else {
                debug!(
                    "Assistant message saved successfully to session {}",
                    sess.id
                );
            }

            // Update session token usage
            sess.add_tokens(
                total_input_tokens,
                total_output_tokens,
                total_reasoning_tokens,
                total_cache_tokens,
            );
            if let Err(e) = database.save_session(sess) {
                debug!("Failed to update session: {}", e);
            }
        }

        return Ok(answer_str);
    }

    let answer = response_message.content.ok_or("No response from LLM")?;
    let answer_str = answer.to_string();

    // Save to session if provided (for simple responses without tool calls)
    if let Some(sess) = session
        && let Some(database) = db
    {
        // Save session metadata FIRST (before messages, due to foreign key constraint)
        if sess.title.is_none() {
            let title = if question.len() > 100 {
                format!("{}...", &question[..97])
            } else {
                question.to_string()
            };
            sess.title = Some(title);
        }

        if let Err(e) = database.save_session(sess) {
            debug!("Failed to save session: {}", e);
        } else {
            debug!("Session saved successfully: {}", sess.id);
        }

        // Save user message
        let user_msg = crate::session::ChatMessage {
            role: "user".to_string(),
            content: question.to_string(),
            sources: if let Some(path) = file_path {
                if let Some(content) = file_content {
                    vec![Source {
                        title: path.to_string(),
                        content: content.to_string(),
                    }]
                } else {
                    vec![]
                }
            } else {
                vec![]
            },
            timestamp: chrono::Utc::now().timestamp(),
            thinking_steps: None,
        };

        if let Err(e) = database.save_message(&sess.id, &user_msg) {
            debug!("Failed to save user message to session: {}", e);
        } else {
            debug!("User message saved successfully to session {}", sess.id);
        }

        // Save assistant message
        let assistant_msg = crate::session::ChatMessage {
            role: "assistant".to_string(),
            content: answer_str.clone(),
            sources: vec![],
            timestamp: chrono::Utc::now().timestamp(),
            thinking_steps: None,
        };

        if let Err(e) = database.save_message(&sess.id, &assistant_msg) {
            debug!("Failed to save assistant message to session: {}", e);
        } else {
            debug!(
                "Assistant message saved successfully to session {}",
                sess.id
            );
        }

        // Update session token usage
        sess.add_tokens(
            total_input_tokens,
            total_output_tokens,
            total_reasoning_tokens,
            total_cache_tokens,
        );
        if let Err(e) = database.save_session(sess) {
            debug!("Failed to update session: {}", e);
        }
    }

    Ok(answer_str)
}

/// Initialize RAG system if needed based on config and CLI flags
async fn initialize_rag_if_needed(
    config_enabled: bool,
    rag_flag: bool,
    no_rag_flag: bool,
    app_config: &config::Config,
) -> Option<Arc<rag::RagSystem>> {
    let should_enable = if no_rag_flag {
        false
    } else if rag_flag {
        true
    } else {
        config_enabled
    };

    if !should_enable {
        return None;
    }

    match db::Database::new(&app_config.database_path) {
        Ok(db) => match rag::RagSystem::new(Arc::new(db), &app_config.rag).await {
            Ok(system) => Some(Arc::new(system)),
            Err(e) => {
                warn!("RAG initialization failed: {}", e);
                None
            }
        },
        Err(e) => {
            warn!("Failed to open database for RAG: {}", e);
            None
        }
    }
}

/// Handles the `ask` command: resolves file content, custom prompt, RAG context,
/// and agent model, then dispatches to the LLM (streaming or non-streaming).
pub async fn run_ask_command(
    question: &str,
    message: Option<&str>,
    no_stream: bool,
    file: Option<&Path>,
    prompt: Option<&Path>,
    agent: Option<&str>,
    rag_flag: bool,
    no_rag_flag: bool,
    app_config: &config::Config,
) {
    let full_question = if let Some(m) = message {
        format!("{} {}", question, m)
    } else {
        question.to_string()
    };

    info!("Q: {}", full_question);

    let file_content = if let Some(file_path) = file {
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

    let rag_system =
        initialize_rag_if_needed(app_config.rag.enabled, rag_flag, no_rag_flag, app_config).await;

    let rag_context = if let Some(ref system) = rag_system {
        println!("🦑: Using RAG for enhanced context...");
        match system.query.execute(&full_question).await {
            Ok(context) if !context.is_empty() => {
                debug!("RAG retrieved {} bytes of context", context.len());
                Some(context)
            }
            Ok(_) => {
                debug!("RAG returned empty context");
                None
            }
            Err(e) => {
                warn!("RAG query failed: {}", e);
                println!("🦑: RAG query failed, continuing without RAG context");
                None
            }
        }
    } else {
        None
    };

    let enhanced_file_content = match (rag_context, file_content) {
        (Some(rag), Some(file)) => Some(format!("{}\n\n# Provided File:\n\n{}", rag, file)),
        (Some(rag), None) => Some(rag),
        (None, file_opt) => file_opt,
    };

    let agent_id = agent.unwrap_or(app_config.agents.default_agent.as_str());
    let model = match app_config.get_agent(agent_id) {
        Some(agent_config) => {
            info!(
                "Using agent '{}' with model '{}'",
                agent_id, agent_config.model
            );
            agent_config.model.clone()
        }
        None => {
            error!("Agent '{}' not found", agent_id);
            println!("🦑: Configuration error - agent '{}' not found", agent_id);
            println!(
                "Available agents: {}",
                app_config
                    .agents
                    .agents
                    .keys()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return;
        }
    };

    // Create session and open database for saving conversation
    let mut session = ChatSession::new();
    session.set_model(model.clone());
    let db = match db::Database::new(&app_config.database_path) {
        Ok(db) => Some(db),
        Err(e) => {
            warn!("Failed to open database for session saving: {}", e);
            None
        }
    };

    if no_stream {
        match ask_llm(
            &full_question,
            enhanced_file_content.as_deref(),
            file.and_then(|p| p.to_str()),
            custom_prompt.as_deref(),
            &model,
            app_config,
            Some(&mut session),
            db.as_ref(),
        )
        .await
        {
            Ok(response) => println!("\n🦑: {}", response),
            Err(e) => error!("Failed to get response: {}", e),
        }
    } else if let Err(e) = ask_llm_streaming(
        &full_question,
        enhanced_file_content.as_deref(),
        file.and_then(|p| p.to_str()),
        custom_prompt.as_deref(),
        &model,
        app_config,
        Some(&mut session),
        db.as_ref(),
    )
    .await
    {
        error!("Failed to get response: {}", e);
    }

    println!("💾 Session saved: {}", &session.id[..8]);
}

/// Handles the `review` command: validates and reads the file, initialises RAG,
/// selects the language-specific prompt, and dispatches to the LLM.
pub async fn run_review_command(
    file: &Path,
    message: Option<&str>,
    no_stream: bool,
    agent: Option<&str>,
    rag_flag: bool,
    no_rag_flag: bool,
    app_config: &config::Config,
) {
    info!("Reviewing file: {:?}", file);

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
                    println!("🦑: I can't find that file. Please check the path and try again.");
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

    let rag_system =
        initialize_rag_if_needed(app_config.rag.enabled, rag_flag, no_rag_flag, app_config).await;

    let rag_context = if let Some(ref system) = rag_system {
        println!("🦑: Using RAG for enhanced context...");
        let file_extension = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");
        let review_query = format!(
            "code review best practices and common issues for {} files{}",
            file_extension,
            message.map(|m| format!(": {}", m)).unwrap_or_default()
        );

        match system.query.execute(&review_query).await {
            Ok(context) if !context.is_empty() => {
                debug!(
                    "RAG retrieved {} bytes of context for review",
                    context.len()
                );
                Some(context)
            }
            Ok(_) => {
                debug!("RAG returned empty context");
                None
            }
            Err(e) => {
                warn!("RAG query failed: {}", e);
                println!("🦑: RAG query failed, continuing without RAG context");
                None
            }
        }
    } else {
        None
    };

    let enhanced_content = if let Some(rag) = rag_context {
        format!("{}\n\n# Code to Review:\n\n{}", rag, file_content)
    } else {
        file_content
    };

    let agent_id = agent.unwrap_or(app_config.agents.default_agent.as_str());
    let model = match app_config.get_agent(agent_id) {
        Some(agent_config) => {
            info!(
                "Using agent '{}' with model '{}'",
                agent_id, agent_config.model
            );
            agent_config.model.clone()
        }
        None => {
            error!("Agent '{}' not found", agent_id);
            println!("🦑: Configuration error - agent '{}' not found", agent_id);
            println!(
                "Available agents: {}",
                app_config
                    .agents
                    .agents
                    .keys()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return;
        }
    };

    // Create session and open database for saving conversation
    let mut session = ChatSession::new();
    session.set_model(model.clone());
    let db = match db::Database::new(&app_config.database_path) {
        Ok(db) => Some(db),
        Err(e) => {
            warn!("Failed to open database for session saving: {}", e);
            None
        }
    };

    if no_stream {
        match ask_llm(
            &question,
            Some(&enhanced_content),
            file.to_str(),
            Some(&combined_review_prompt),
            &model,
            app_config,
            Some(&mut session),
            db.as_ref(),
        )
        .await
        {
            Ok(response) => println!("\n🦑: {}", response),
            Err(e) => error!("Failed to get review: {}", e),
        }
    } else if let Err(e) = ask_llm_streaming(
        &question,
        Some(&enhanced_content),
        file.to_str(),
        Some(&combined_review_prompt),
        &model,
        app_config,
        Some(&mut session),
        db.as_ref(),
    )
    .await
    {
        error!("Failed to get review: {}", e);
    }

    println!("💾 Session saved: {}", &session.id[..8]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_reasoning_blocks_single() {
        let content = "Hello <think>internal reasoning here</think> world!";
        let result = strip_reasoning_blocks(content);
        assert_eq!(result, "Hello  world!");
    }

    #[test]
    fn test_strip_reasoning_blocks_multiple() {
        let content = "Start <think>first thought</think> middle <think>second thought</think> end";
        let result = strip_reasoning_blocks(content);
        assert_eq!(result, "Start  middle  end");
    }

    #[test]
    fn test_strip_reasoning_blocks_empty() {
        let content = "Hello <think></think> world!";
        let result = strip_reasoning_blocks(content);
        assert_eq!(result, "Hello  world!");
    }

    #[test]
    fn test_strip_reasoning_blocks_no_blocks() {
        let content = "Hello world!";
        let result = strip_reasoning_blocks(content);
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_strip_reasoning_blocks_malformed() {
        // Missing closing tag - should leave as-is
        let content = "Hello <think>unclosed reasoning";
        let result = strip_reasoning_blocks(content);
        assert_eq!(result, "Hello <think>unclosed reasoning");
    }

    #[test]
    fn test_strip_reasoning_blocks_with_newlines() {
        let content = "Text before\n<think>\nMulti-line\nreasoning\n</think>\nText after";
        let result = strip_reasoning_blocks(content);
        assert_eq!(result, "Text before\n\nText after");
    }
}
