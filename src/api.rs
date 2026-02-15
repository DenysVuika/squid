use actix_web::{web, HttpResponse, Error};
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
use futures::stream::StreamExt;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use std::time::Instant;

use crate::{config, envinfo, llm, logger, session, tokens, tools};

// Tool approval state management
#[derive(Debug)]
pub struct ApprovalState {
    pub tool_name: String,
    pub tool_args: Value,
    pub tool_call_id: String,
    pub sender: oneshot::Sender<bool>,
    pub created_at: Instant,
}

pub type ApprovalStateMap = Arc<Mutex<HashMap<String, ApprovalState>>>;

/// Get a human-readable description for a tool
fn get_tool_description(tool_name: &str) -> String {
    match tool_name {
        "read_file" => "Read the contents of a file from the filesystem".to_string(),
        "write_file" => "Write content to a file on the filesystem".to_string(),
        "grep" => "Search for a pattern in files using regex".to_string(),
        "bash" => "Execute a bash command (safe, read-only commands only)".to_string(),
        "demo_tool" => "A demo tool for testing the approval workflow (safe, read-only)".to_string(),
        _ => format!("Execute tool: {}", tool_name),
    }
}

#[derive(Debug, Deserialize)]
pub struct FileAttachment {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub files: Vec<FileAttachment>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Source {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "session")]
    Session { session_id: String },
    #[serde(rename = "sources")]
    Sources { sources: Vec<Source> },
    #[serde(rename = "content")]
    Content { text: String },
    #[serde(rename = "reasoning")]
    Reasoning { text: String },
    #[serde(rename = "tool_call")]
    ToolCall { name: String, arguments: String },
    #[serde(rename = "tool_result")]
    ToolResult { name: String, result: String },
    #[serde(rename = "usage")]
    Usage {
        input_tokens: i64,
        output_tokens: i64,
        reasoning_tokens: i64,
        cache_tokens: i64,
    },
    #[serde(rename = "tool_approval_request")]
    ToolApprovalRequest {
        approval_id: String,
        tool_name: String,
        tool_args: Value,
        tool_description: String,
    },
    #[serde(rename = "tool_approval_response")]
    ToolApprovalResponse {
        approval_id: String,
        approved: bool,
    },
    #[serde(rename = "tool_invocation_completed")]
    ToolInvocationCompleted {
        name: String,
        arguments: Value,
        result: Option<String>,
        error: Option<String>,
    },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "done")]
    Done,
}

#[derive(Debug, Serialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub sources: Vec<Source>,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_steps: Option<Vec<session::ThinkingStep>>,
}

#[derive(Debug, Serialize)]
pub struct TokenUsageResponse {
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_tokens: i64,
    pub context_window: u32,
    pub context_utilization: f64,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub messages: Vec<SessionMessage>,
    pub created_at: i64,
    pub updated_at: i64,
    pub title: Option<String>,
    pub model_id: Option<String>,
    pub token_usage: TokenUsageResponse,
    pub cost_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct SessionListItem {
    pub session_id: String,
    pub message_count: usize,
    pub created_at: i64,
    pub updated_at: i64,
    pub preview: Option<String>,
    pub title: Option<String>,
    pub model_id: Option<String>,
    pub token_usage: TokenUsageResponse,
    pub cost_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionListItem>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    pub level: Option<String>,
    pub session_id: Option<String>,
}

fn default_page() -> usize {
    1
}

fn default_page_size() -> usize {
    50
}

#[derive(Debug, Serialize)]
pub struct LogEntryResponse {
    pub id: i64,
    pub timestamp: i64,
    pub level: String,
    pub target: String,
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub logs: Vec<LogEntryResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

/// Get session history by ID
pub async fn get_session(
    session_id: web::Path<String>,
    session_manager: web::Data<Arc<session::SessionManager>>,
) -> Result<HttpResponse, Error> {
    debug!("Getting session: {}", session_id);

    match session_manager.get_session(&session_id) {
        Some(session) => {
            let response = SessionResponse {
                session_id: session.id,
                messages: session.messages.iter().map(|msg| {
                    debug!("Serializing message with {} thinking steps", msg.thinking_steps.as_ref().map(|s| s.len()).unwrap_or(0));
                    SessionMessage {
                        role: msg.role.clone(),
                        content: msg.content.clone(),
                        sources: msg.sources.iter().map(|s| Source {
                            title: s.title.clone(),
                            content: s.content.clone(),
                        }).collect(),
                        timestamp: msg.timestamp,
                        thinking_steps: msg.thinking_steps.clone(),
                    }
                }).collect(),
                created_at: session.created_at,
                updated_at: session.updated_at,
                title: session.title.clone(),
                model_id: session.model_id.clone(),
                token_usage: TokenUsageResponse {
                    total_tokens: session.token_usage.total_tokens,
                    input_tokens: session.token_usage.input_tokens,
                    output_tokens: session.token_usage.output_tokens,
                    reasoning_tokens: session.token_usage.reasoning_tokens,
                    cache_tokens: session.token_usage.cache_tokens,
                    context_window: session.token_usage.context_window,
                    context_utilization: session.token_usage.context_utilization,
                },
                cost_usd: session.cost_usd,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Session not found"
        }))),
    }
}

/// List all sessions with metadata
pub async fn list_sessions(
    session_manager: web::Data<Arc<session::SessionManager>>,
) -> Result<HttpResponse, Error> {
    debug!("Listing all sessions");

    let session_ids = session_manager.list_sessions();
    let mut sessions = Vec::new();

    for session_id in session_ids {
        if let Some(session) = session_manager.get_session(&session_id) {
            // Get preview from first user message
            let preview = session.messages
                .iter()
                .find(|msg| msg.role == "user")
                .map(|msg| {
                    let content = &msg.content;
                    if content.len() > 100 {
                        format!("{}...", &content[..100])
                    } else {
                        content.clone()
                    }
                });

            sessions.push(SessionListItem {
                session_id: session.id.clone(),
                message_count: session.messages.len(),
                created_at: session.created_at,
                updated_at: session.updated_at,
                preview,
                title: session.title.clone(),
                model_id: session.model_id.clone(),
                token_usage: TokenUsageResponse {
                    total_tokens: session.token_usage.total_tokens,
                    input_tokens: session.token_usage.input_tokens,
                    output_tokens: session.token_usage.output_tokens,
                    reasoning_tokens: session.token_usage.reasoning_tokens,
                    cache_tokens: session.token_usage.cache_tokens,
                    context_window: session.token_usage.context_window,
                    context_utilization: session.token_usage.context_utilization,
                },
                cost_usd: session.cost_usd,
            });
        }
    }

    // Sort by updated_at descending (most recent first)
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    let total = sessions.len();
    Ok(HttpResponse::Ok().json(SessionListResponse { sessions, total }))
}

/// Delete a session by ID
pub async fn delete_session(
    session_id: web::Path<String>,
    session_manager: web::Data<Arc<session::SessionManager>>,
) -> Result<HttpResponse, Error> {
    debug!("Deleting session: {}", session_id);

    let deleted = session_manager.delete_session(&session_id);

    if deleted {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Session deleted successfully"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Session not found"
        })))
    }
}

/// Update a session (e.g., rename)
pub async fn update_session(
    session_id: web::Path<String>,
    update_request: web::Json<UpdateSessionRequest>,
    session_manager: web::Data<Arc<session::SessionManager>>,
) -> Result<HttpResponse, Error> {
    debug!("Updating session: {} with title: {}", session_id, update_request.title);

    // Validate title is not empty
    let title = update_request.title.trim();
    if title.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Title cannot be empty"
        })));
    }

    match session_manager.update_session_title(&session_id, title.to_string()) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Session updated successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update session: {}", e)
        }))),
    }
}

/// Handles streaming chat requests
pub async fn chat_stream(
    body: web::Json<ChatRequest>,
    app_config: web::Data<Arc<config::Config>>,
    session_manager: web::Data<Arc<session::SessionManager>>,
    approval_map: web::Data<ApprovalStateMap>,
) -> Result<HttpResponse, Error> {
    debug!("Received chat request: {:?}", body);

    let question = body.message.clone();

    // Validate file sizes (10MB limit per file)
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    for file in &body.files {
        if file.content.len() > MAX_FILE_SIZE {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("File '{}' exceeds size limit of 10MB ({} bytes > {} bytes)",
                    file.filename, file.content.len(), MAX_FILE_SIZE)
            })));
        }
    }

    let files: Vec<session::FileAttachment> = body.files.iter().map(|f| session::FileAttachment {
        filename: f.filename.clone(),
        content: f.content.clone(),
    }).collect();
    let system_prompt = body.system_prompt.clone();
    let system_prompt_for_stream = system_prompt.clone(); // Clone for use inside stream
    let app_config_clone = app_config.get_ref().clone();
    let session_manager_clone = session_manager.get_ref().clone();
    // Use the model from request, or fall back to config default
    let model_id = body.model.clone().unwrap_or_else(|| app_config.api_model.clone());

    // Get or create session
    let session_id = body.session_id.clone().unwrap_or_else(|| {
        session_manager_clone.create_session()
    });

    // Create SSE stream
    let stream = async_stream::stream! {
        // Send session ID first
        let session_event = StreamEvent::Session {
            session_id: session_id.clone(),
        };
        let json = serde_json::to_string(&session_event).unwrap_or_default();
        yield Ok::<_, actix_web::Error>(
            web::Bytes::from(format!("data: {}\n\n", json))
        );

        // Add user message to session and get sources
        let sources = match session_manager_clone.add_user_message(
            &session_id,
            question.clone(),
            files.clone(),
        ) {
            Ok(sources) => sources,
            Err(e) => {
                let error_event = StreamEvent::Error {
                    message: format!("Failed to add message to session: {}", e),
                };
                let json = serde_json::to_string(&error_event).unwrap_or_default();
                yield Ok::<_, actix_web::Error>(
                    web::Bytes::from(format!("data: {}\n\n", json))
                );
                return;
            }
        };

        // Send sources if any
        if !sources.is_empty() {
            let sources_event = StreamEvent::Sources {
                sources: sources.iter().map(|s| Source {
                    title: s.title.clone(),
                    content: s.content.clone(),
                }).collect(),
            };
            let json = serde_json::to_string(&sources_event).unwrap_or_default();
            yield Ok::<_, actix_web::Error>(
                web::Bytes::from(format!("data: {}\n\n", json))
            );
        }

        match create_chat_stream(
            &session_id,
            &question,
            &files,
            system_prompt.as_deref(),
            &model_id,
            &app_config_clone,
            &session_manager_clone,
            approval_map.get_ref(),
        ).await {
            Ok(content_stream) => {
                // Accumulate assistant content and token usage as we stream
                let mut accumulated_content = String::new();
                let mut accumulated_reasoning = String::new();
                let mut total_input_tokens = 0i64;
                let mut total_output_tokens = 0i64;
                let mut total_reasoning_tokens = 0i64;
                let mut total_cache_tokens = 0i64;
                let mut received_usage = false; // Track if provider sent usage
                // Track thinking steps in order as they occur during streaming
                let mut thinking_steps_ordered: Vec<session::ThinkingStep> = Vec::new();
                let mut step_order = 0i32;
                
                // Track reasoning blocks separately - don't merge them
                let mut last_closed_think_pos = 0;

                // Stream each content chunk as it arrives
                let mut pinned_stream = Box::pin(content_stream);
                while let Some(result) = pinned_stream.next().await {
                    match result {
                        Ok(chunk) => {
                            // Accumulate content chunks
                            if let StreamEvent::Content { ref text } = chunk {
                                accumulated_content.push_str(text);
                                
                                // Check if we completed any <think>...</think> blocks
                                // Process each closed block as a separate reasoning step
                                loop {
                                    let search_start = last_closed_think_pos;
                                    if let Some(relative_start) = accumulated_content[search_start..].find("<think>") {
                                        let absolute_start = search_start + relative_start;
                                        if let Some(relative_end) = accumulated_content[absolute_start..].find("</think>") {
                                            let absolute_end = absolute_start + relative_end;
                                            let reasoning_text = accumulated_content[absolute_start + 7..absolute_end].to_string();
                                            if !reasoning_text.trim().is_empty() {
                                                // Add reasoning step immediately as a SEPARATE step (don't merge)
                                                thinking_steps_ordered.push(session::ThinkingStep {
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
                                            last_closed_think_pos = absolute_end + 8;
                                        } else {
                                            break; // Incomplete block, wait for more content
                                        }
                                    } else {
                                        break; // No more think blocks
                                    }
                                }
                            }

                            // Accumulate reasoning chunks
                            if let StreamEvent::Reasoning { ref text } = chunk {
                                accumulated_reasoning.push_str(text);
                            }

                            // Accumulate token usage
                            if let StreamEvent::Usage { input_tokens, output_tokens, reasoning_tokens, cache_tokens } = chunk {
                                total_input_tokens += input_tokens;
                                total_output_tokens += output_tokens;
                                total_reasoning_tokens += reasoning_tokens;
                                total_cache_tokens += cache_tokens;
                                received_usage = true;
                            }

                            // Add tool invocation to thinking steps immediately
                            // This preserves the order: when a tool completes, it gets added right after the last reasoning step
                            if let StreamEvent::ToolInvocationCompleted { name, arguments, result, error } = &chunk {
                                // Capture content accumulated before this tool
                                let content_snapshot = accumulated_content.trim().to_string();
                                
                                // Add tool as thinking step immediately (preserves order)
                                thinking_steps_ordered.push(session::ThinkingStep {
                                    step_type: "tool".to_string(),
                                    step_order,
                                    content: None,
                                    tool_name: Some(name.clone()),
                                    tool_arguments: Some(arguments.clone()),
                                    tool_result: result.clone(),
                                    tool_error: error.clone(),
                                    content_before_tool: if content_snapshot.is_empty() {
                                        None
                                    } else {
                                        Some(content_snapshot)
                                    },
                                });
                                step_order += 1;
                            }

                            let json = serde_json::to_string(&chunk).unwrap_or_default();
                            yield Ok::<_, actix_web::Error>(
                                web::Bytes::from(format!("data: {}\n\n", json))
                            );
                        }
                        Err(e) => {
                            let error_event = StreamEvent::Error {
                                message: e.to_string(),
                            };
                            let json = serde_json::to_string(&error_event).unwrap_or_default();
                            yield Ok::<_, actix_web::Error>(
                                web::Bytes::from(format!("data: {}\n\n", json))
                            );
                            break;
                        }
                    }
                }

                // Add assistant message to session with sources
                // Parse out ALL <think> tags from accumulated content for final display
                let mut final_content = accumulated_content.clone();
                let mut reasoning_parts = Vec::new();
                
                // Remove all <think>...</think> tags
                while let Some(think_start) = final_content.find("<think>") {
                    if let Some(think_end) = final_content.find("</think>") {
                        if think_end > think_start {
                            // Extract reasoning between tags for backward compatibility
                            let reasoning_text = final_content[think_start + 7..think_end].to_string();
                            if !reasoning_text.trim().is_empty() {
                                reasoning_parts.push(reasoning_text);
                            }
                            // Remove this <think>...</think> section from content
                            final_content = format!(
                                "{}{}",
                                &final_content[..think_start],
                                &final_content[think_end + 8..]
                            );
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                // Remove all <tool_call>...</tool_call> tags (model's way of indicating tool calls)
                while let Some(tool_start) = final_content.find("<tool_call>") {
                    if let Some(tool_end) = final_content.find("</tool_call>") {
                        if tool_end > tool_start {
                            // Remove this <tool_call>...</tool_call> section from content
                            final_content = format!(
                                "{}{}",
                                &final_content[..tool_start],
                                &final_content[tool_end + 12..]
                            );
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Use the thinking steps we built during streaming
                let thinking_steps_opt = if thinking_steps_ordered.is_empty() {
                    None
                } else {
                    Some(thinking_steps_ordered)
                };

                // Trim whitespace and check if we have actual content
                let final_content_trimmed = final_content.trim();

                if !final_content_trimmed.is_empty() || thinking_steps_opt.is_some() {
                    debug!("Saving assistant message to session {} (content length: {} chars, thinking_steps: {})",
                        session_id, final_content_trimmed.len(),
                        thinking_steps_opt.as_ref().map(|s| s.len()).unwrap_or(0));
                    
                    match session_manager_clone.add_assistant_message(
                        &session_id,
                        final_content_trimmed.to_string(),
                        sources,
                        thinking_steps_opt,
                    ) {
                        Ok(_) => debug!("Assistant message saved successfully"),
                        Err(e) => debug!("Failed to save assistant message: {}", e),
                    }
                } else {
                    debug!("Skipping empty assistant message for session {}", session_id);
                }

                // If provider didn't send usage stats, estimate them client-side
                if !received_usage {
                    debug!("Provider didn't report token usage, estimating client-side for model: {}", model_id);

                    // Estimate input tokens from the full conversation context
                    if let Some(session) = session_manager_clone.get_session(&session_id) {
                        // Build message list same way as the API call
                        let mut estimate_messages = Vec::new();

                        // Add system message (same logic as create_chat_stream)
                        let sys_msg_text = if let Some(ref prompt) = system_prompt_for_stream {
                            prompt.clone()
                        } else {
                            llm::combine_prompts(llm::get_ask_prompt())
                        };
                        let system_msg = ChatCompletionRequestSystemMessage {
                            content: sys_msg_text.into(),
                            ..Default::default()
                        };
                        estimate_messages.push(system_msg.into());

                        // Add all conversation history
                        for msg in &session.messages {
                            if msg.role == "user" {
                                estimate_messages.push(
                                    ChatCompletionRequestUserMessage {
                                        content: msg.content.clone().into(),
                                        ..Default::default()
                                    }
                                    .into(),
                                );
                            } else if msg.role == "assistant" {
                                estimate_messages.push(
                                    ChatCompletionRequestAssistantMessage {
                                        content: Some(msg.content.clone().into()),
                                        ..Default::default()
                                    }
                                    .into(),
                                );
                            }
                        }

                        let (estimated_input, _) = tokens::estimate_tokens(&model_id, &estimate_messages);
                        total_input_tokens = estimated_input;

                        // Estimate output tokens from accumulated content
                        if !accumulated_content.is_empty() {
                            total_output_tokens = tokens::estimate_message_tokens(&model_id, accumulated_content.as_str());
                        }

                        debug!(
                            "Estimated tokens for session {}: input={}, output={}",
                            session_id, total_input_tokens, total_output_tokens
                        );

                        // Send estimated usage to UI
                        let usage_event = StreamEvent::Usage {
                            input_tokens: total_input_tokens,
                            output_tokens: total_output_tokens,
                            reasoning_tokens: total_reasoning_tokens,
                            cache_tokens: total_cache_tokens,
                        };
                        let json = serde_json::to_string(&usage_event).unwrap_or_default();
                        yield Ok::<_, actix_web::Error>(
                            web::Bytes::from(format!("data: {}\n\n", json))
                        );
                    }
                }

                // Update session with token usage and model info
                if total_input_tokens > 0 || total_output_tokens > 0 {
                    debug!("Updating session {} with token usage: input={}, output={}",
                           session_id, total_input_tokens, total_output_tokens);
                    if let Err(e) = session_manager_clone.update_token_usage(
                        &session_id,
                        &model_id,
                        total_input_tokens,
                        total_output_tokens,
                        total_reasoning_tokens,
                        total_cache_tokens,
                        app_config_clone.context_window,
                    ) {
                        debug!("Failed to update token usage: {}", e);
                    }
                }

                // Send done event
                let done_event = StreamEvent::Done;
                let json = serde_json::to_string(&done_event).unwrap_or_default();
                yield Ok::<_, actix_web::Error>(
                    web::Bytes::from(format!("data: {}\n\n", json))
                );
            }
            Err(e) => {
                let error_event = StreamEvent::Error {
                    message: e.to_string(),
                };
                let json = serde_json::to_string(&error_event).unwrap_or_default();
                yield Ok::<_, actix_web::Error>(
                    web::Bytes::from(format!("data: {}\n\n", json))
                );
            }
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(Box::pin(stream)))
}

#[allow(unused_variables)] // approval_map is used inside async_stream::stream! macro
async fn create_chat_stream(
    session_id: &str,
    question: &str,
    files: &[session::FileAttachment],
    system_prompt: Option<&str>,
    model_id: &str,
    app_config: &config::Config,
    session_manager: &session::SessionManager,
    approval_map: &ApprovalStateMap,
) -> Result<
    impl futures::Stream<Item = Result<StreamEvent, Box<dyn std::error::Error + Send + Sync>>>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    debug!("Using API URL: {}", app_config.api_url);
    debug!("Using API Model: {}", model_id);

    let config = OpenAIConfig::new()
        .with_api_base(&app_config.api_url)
        .with_api_key(app_config.get_api_key());

    let client = Client::with_config(config);

    // Get environment context
    let env_context = envinfo::get_env_context();

    // Build user message with file contents if present
    let user_message = if !files.is_empty() {
        let mut message = env_context.clone();
        message.push_str("\n\n");

        for file in files {
            message.push_str(&format!(
                "Here is the content of '{}':\n\n```\n{}\n```\n\n",
                file.filename, file.content
            ));
        }

        message.push_str(&format!("User query: {}", question));
        message
    } else {
        format!("{}\n\nUser query: {}", env_context, question)
    };

    let default_prompt = llm::combine_prompts(llm::get_ask_prompt());
    let system_message = system_prompt.unwrap_or(&default_prompt);

    debug!("System message:\n{}", system_message);
    debug!("User message:\n{}", user_message);

    // Get conversation history from session
    let session = session_manager
        .get_session(session_id)
        .ok_or("Session not found")?;

    let mut messages: Vec<ChatCompletionRequestMessage> = vec![
        ChatCompletionRequestSystemMessage {
            content: system_message.to_string().into(),
            ..Default::default()
        }
        .into(),
    ];

    // Add conversation history (excluding the last user message we just added)
    for (i, msg) in session.messages.iter().enumerate() {
        if i == session.messages.len() - 1 {
            break; // Skip the last message as we'll add it with full context
        }

        if msg.role == "user" {
            messages.push(
                ChatCompletionRequestUserMessage {
                    content: msg.content.clone().into(),
                    ..Default::default()
                }
                .into(),
            );
        } else if msg.role == "assistant" {
            messages.push(
                ChatCompletionRequestAssistantMessage {
                    content: Some(msg.content.clone().into()),
                    ..Default::default()
                }
                .into(),
            );
        }
    }

    // Add the current user message with full context
    messages.push(
        ChatCompletionRequestUserMessage {
            content: user_message.into(),
            ..Default::default()
        }
        .into(),
    );

    let mut tool_calls: Vec<ChatCompletionMessageToolCall> = Vec::new();

    let output_stream = async_stream::stream! {
        loop {
        let request = CreateChatCompletionRequestArgs::default()
            .model(model_id)
            .messages(messages.clone())
            .tools(tools::get_tools())
            .stream_options(ChatCompletionStreamOptions {
                include_usage: Some(true),
                include_obfuscation: None,
            })
            .build();

            let request = match request {
                Ok(req) => req,
                Err(e) => {
                    yield Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
                    break;
                }
            };

            debug!("Sending streaming request...");

            let stream_result = client.chat().create_stream(request).await;
            let mut stream = match stream_result {
                Ok(s) => s,
                Err(e) => {
                    yield Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
                    break;
                }
            };

            tool_calls.clear();

            while let Some(result) = stream.next().await {
                let response = match result {
                    Ok(r) => r,
                    Err(e) => {
                        yield Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
                        break;
                    }
                };

                // Yield token usage statistics from streaming response
                if let Some(usage) = &response.usage {
                    debug!(
                        "Token usage - Prompt: {}, Completion: {}, Total: {}",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                    );

                    yield Ok(StreamEvent::Usage {
                        input_tokens: usage.prompt_tokens as i64,
                        output_tokens: usage.completion_tokens as i64,
                        reasoning_tokens: 0, // Not provided by OpenAI streaming API
                        cache_tokens: 0,     // Not provided by OpenAI streaming API
                    });
                }

                for choice in response.choices {
                    // Handle content - yield it immediately
                    if let Some(content) = &choice.delta.content {
                        yield Ok(StreamEvent::Content {
                            text: content.clone(),
                        });
                    }

                // Handle tool calls
                if let Some(tool_calls_delta) = &choice.delta.tool_calls {
                    for tool_call_delta in tool_calls_delta {
                        let index = tool_call_delta.index as usize;

                        // Ensure we have enough space in the vector
                        while tool_calls.len() <= index {
                            tool_calls.push(ChatCompletionMessageToolCall {
                                id: String::new(),
                                function: Default::default(),
                            });
                        }

                        // Update the tool call with delta information
                        if let Some(id) = &tool_call_delta.id {
                            tool_calls[index].id = id.clone();
                        }
                        if let Some(function) = &tool_call_delta.function {
                            if let Some(name) = &function.name {
                                tool_calls[index].function.name = name.clone();
                            }
                            if let Some(args) = &function.arguments {
                                tool_calls[index].function.arguments.push_str(args);
                            }
                        }
                    }
                }

                // Handle finish reason
                if let Some(finish_reason) = &choice.finish_reason {
                    match finish_reason {
                        FinishReason::ToolCalls => {
                            debug!("Executing tool calls...");

                            // Add assistant message with tool calls
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

                            // Execute tools with approval handling
                            for tool_call in tool_calls.iter() {
                                let name = &tool_call.function.name;
                                let args_str = &tool_call.function.arguments;
                                let tool_call_id = &tool_call.id;

                                // Parse arguments
                                let args_value: Value = match args_str.parse() {
                                    Ok(v) => v,
                                    Err(e) => {
                                        let error_result = json!({
                                            "error": format!("Failed to parse tool arguments: {}", e)
                                        });
                                        messages.push(
                                            ChatCompletionRequestToolMessage {
                                                content: error_result.to_string().into(),
                                                tool_call_id: tool_call_id.clone(),
                                            }
                                            .into(),
                                        );
                                        continue;
                                    }
                                };

                                // Check permission status
                                let permission_status = tools::check_tool_permission(name, &args_value, app_config);
                                
                                debug!("Tool '{}' permission status: {:?}", name, permission_status);

                                match permission_status {
                                    tools::ToolPermissionStatus::Denied { reason } => {
                                        // Tool is denied, don't execute
                                        let deny_result = json!({
                                            "error": reason,
                                            "skipped": true
                                        });
                                        messages.push(
                                            ChatCompletionRequestToolMessage {
                                                content: deny_result.to_string().into(),
                                                tool_call_id: tool_call_id.clone(),
                                            }
                                            .into(),
                                        );
                                    }
                                    tools::ToolPermissionStatus::Allowed => {
                                        // Tool is auto-allowed, execute directly
                                        let result = tools::execute_tool_direct(name, &args_value, app_config).await;
                                        
                                        // Emit tool invocation completed event
                                        yield Ok(StreamEvent::ToolInvocationCompleted {
                                            name: name.clone(),
                                            arguments: args_value.clone(),
                                            result: Some(result.to_string()),
                                            error: None,
                                        });
                                        
                                        messages.push(
                                            ChatCompletionRequestToolMessage {
                                                content: result.to_string().into(),
                                                tool_call_id: tool_call_id.clone(),
                                            }
                                            .into(),
                                        );
                                    }
                                    tools::ToolPermissionStatus::NeedsApproval => {
                                        use uuid::Uuid;
                                        use std::time::Duration;
                                        
                                        // Generate unique approval ID
                                        let approval_id = Uuid::new_v4().to_string();
                                        
                                        info!("Tool '{}' needs approval, generated approval_id: {}", name, approval_id);
                                        
                                        // Create oneshot channel for approval response
                                        let (sender, receiver) = tokio::sync::oneshot::channel::<bool>();
                                        
                                        // Store approval state in map
                                        {
                                            let mut approvals = approval_map.lock().await;
                                            approvals.insert(approval_id.clone(), ApprovalState {
                                                tool_name: name.clone(),
                                                tool_args: args_value.clone(),
                                                tool_call_id: tool_call_id.clone(),
                                                sender,
                                                created_at: Instant::now(),
                                            });
                                        }
                                        
                                        // Yield approval request event
                                        yield Ok(StreamEvent::ToolApprovalRequest {
                                            approval_id: approval_id.clone(),
                                            tool_name: name.clone(),
                                            tool_args: args_value.clone(),
                                            tool_description: get_tool_description(name),
                                        });
                                        
                                        info!("Emitted ToolApprovalRequest event for approval_id: {}", approval_id);
                                        
                                        // Wait for approval with 5 minute timeout
                                        let approved = match tokio::time::timeout(
                                            Duration::from_secs(300),
                                            receiver
                                        ).await {
                                            Ok(Ok(decision)) => {
                                                info!("Tool '{}' approval received: {}", name, decision);
                                                decision
                                            }
                                            Ok(Err(_)) => {
                                                warn!("Tool '{}' approval channel closed without response", name);
                                                false
                                            }
                                            Err(_) => {
                                                warn!("Tool '{}' approval timed out after 5 minutes", name);
                                                false
                                            }
                                        };
                                        
                                        // Clean up from map
                                        {
                                            let mut approvals = approval_map.lock().await;
                                            approvals.remove(&approval_id);
                                        }
                                        
                                        // Yield approval response event
                                        yield Ok(StreamEvent::ToolApprovalResponse {
                                            approval_id: approval_id.clone(),
                                            approved,
                                        });
                                        
                                        // Execute based on approval
                                        if approved {
                                            let result = tools::execute_tool_direct(name, &args_value, app_config).await;
                                            
                                            // Emit tool invocation completed event
                                            yield Ok(StreamEvent::ToolInvocationCompleted {
                                                name: name.clone(),
                                                arguments: args_value.clone(),
                                                result: Some(result.to_string()),
                                                error: None,
                                            });
                                            
                                            messages.push(
                                                ChatCompletionRequestToolMessage {
                                                    content: result.to_string().into(),
                                                    tool_call_id: tool_call_id.clone(),
                                                }
                                                .into(),
                                            );
                                        } else {
                                            let reject_result = json!({
                                                "message": format!("Tool '{}' was not executed because you rejected it.", name),
                                                "skipped": true
                                            });
                                            messages.push(
                                                ChatCompletionRequestToolMessage {
                                                    content: reject_result.to_string().into(),
                                                    tool_call_id: tool_call_id.clone(),
                                                }
                                                .into(),
                                            );
                                        }
                                    }
                                }
                            }

                            // Continue the loop to make another request with tool results
                            break;
                        }
                        FinishReason::Stop => {
                            return;
                        }
                        _ => {}
                    }
                }
            }


        }

            // If we had tool calls, the loop continues to make another request
            // If not, we should have returned by now
            if tool_calls.is_empty() {
                break;
            }
        }
    };

    Ok(output_stream)
}

/// Get logs with pagination
pub async fn get_logs(
    query: web::Query<LogsQuery>,
    app_config: web::Data<Arc<config::Config>>,
) -> Result<HttpResponse, Error> {
    let db_path = &app_config.get_ref().database_path;

    // Calculate offset from page number
    let offset = (query.page.saturating_sub(1)) * query.page_size;

    // Query logs with limit for total count + 1 to check if there are more
    let all_logs = logger::query_logs(
        db_path,
        None, // Get all to calculate total
        query.level.as_deref(),
        query.session_id.as_deref(),
    )
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to query logs: {}", e))
    })?;

    let total = all_logs.len();
    let total_pages = (total + query.page_size - 1) / query.page_size;

    // Get the paginated subset
    let logs: Vec<LogEntryResponse> = all_logs
        .into_iter()
        .skip(offset)
        .take(query.page_size)
        .map(|entry| LogEntryResponse {
            id: entry.id,
            timestamp: entry.timestamp,
            level: entry.level,
            target: entry.target,
            message: entry.message,
            session_id: entry.session_id,
        })
        .collect();

    Ok(HttpResponse::Ok().json(LogsResponse {
        logs,
        total,
        page: query.page,
        page_size: query.page_size,
        total_pages,
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub max_context_length: u32,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModelMetadataFile {
    models: HashMap<String, ModelMetadata>,
    default_max_context_length: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_pricing_model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub max_context_length: u32,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub models: Vec<ModelInfo>,
}

/// Get available models from the LLM provider and augment with metadata
pub async fn get_models(
    app_config: web::Data<Arc<config::Config>>,
) -> Result<HttpResponse, Error> {
    debug!("Fetching available models from API");

    // Load model metadata from JSON file
    let metadata_json = include_str!("assets/model-metadata.json");
    let metadata_file: ModelMetadataFile = serde_json::from_str(metadata_json)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to parse model metadata: {}", e))
        })?;

    // Fetch models from the LLM provider
    let client = reqwest::Client::new();
    let models_url = format!("{}/models", app_config.api_url);

    let response = client
        .get(&models_url)
        .send()
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to fetch models: {}", e))
        })?;

    if !response.status().is_success() {
        return Err(actix_web::error::ErrorInternalServerError(
            format!("API returned error status: {}", response.status())
        ));
    }

    let api_response: Value = response.json().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to parse API response: {}", e))
    })?;

    // Extract model IDs from the API response
    let mut models = Vec::new();

    if let Some(data) = api_response.get("data").and_then(|d| d.as_array()) {
        for model in data {
            if let Some(id) = model.get("id").and_then(|i| i.as_str()) {
                // Skip embedding models
                if id.contains("embedding") {
                    continue;
                }

                // Try to find metadata for this model
                let mut metadata = metadata_file.models.get(id).cloned();

                // If not found, try to find by alias or fuzzy match
                if metadata.is_none() {
                    for (key, meta) in &metadata_file.models {
                        if let Some(aliases) = &meta.aliases {
                            if aliases.contains(&id.to_string()) {
                                metadata = Some(meta.clone());
                                break;
                            }
                        }
                        // Also check if the id contains the key (fuzzy match)
                        if id.contains(key) || key.contains(id) {
                            metadata = Some(meta.clone());
                            break;
                        }
                    }
                }

                let model_info = if let Some(meta) = metadata {
                    ModelInfo {
                        id: id.to_string(),
                        name: meta.name,
                        max_context_length: meta.max_context_length,
                        provider: meta.provider,
                        r#type: meta.r#type,
                        pricing_model: meta.pricing_model,
                    }
                } else {
                    // Use defaults if no metadata found
                    ModelInfo {
                        id: id.to_string(),
                        name: id.to_string(),
                        max_context_length: metadata_file.default_max_context_length,
                        provider: "Unknown".to_string(),
                        r#type: None,
                        pricing_model: metadata_file.default_pricing_model.clone(),
                    }
                };

                models.push(model_info);
            }
        }
    }

    // Sort models: Qwen first, then by provider name
    models.sort_by(|a, b| {
        let a_is_qwen = a.provider == "Qwen" || a.id.contains("qwen");
        let b_is_qwen = b.provider == "Qwen" || b.id.contains("qwen");

        if a_is_qwen && !b_is_qwen {
            std::cmp::Ordering::Less
        } else if !a_is_qwen && b_is_qwen {
            std::cmp::Ordering::Greater
        } else {
            a.provider.cmp(&b.provider).then(a.name.cmp(&b.name))
        }
    });

    Ok(HttpResponse::Ok().json(ModelsResponse { models }))
}

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub api_url: String,
    pub api_model: String,
    pub context_window: u32,
}

/// Get API configuration (default model, etc.)
pub async fn get_config(
    app_config: web::Data<Arc<config::Config>>,
) -> Result<HttpResponse, Error> {
    debug!("Fetching API configuration");

    let response = ConfigResponse {
        api_url: app_config.api_url.clone(),
        api_model: app_config.api_model.clone(),
        context_window: app_config.context_window,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
pub struct ToolApprovalRequest {
    pub approval_id: String,
    pub approved: bool,
    #[serde(default)]
    pub save_decision: bool,
    #[serde(default)]
    pub scope: String, // "tool" or "tool:specific" (e.g., "bash:ls")
}

#[derive(Debug, Serialize)]
pub struct ToolApprovalResponse {
    pub success: bool,
    pub message: String,
}

/// Handle tool approval requests from the web UI
pub async fn handle_tool_approval(
    body: web::Json<ToolApprovalRequest>,
    approval_map: web::Data<ApprovalStateMap>,
) -> Result<HttpResponse, Error> {
    debug!("Received tool approval: {:?}", body);

    // Find the pending approval
    let mut approvals = approval_map.lock().await;
    
    if let Some(approval_state) = approvals.remove(&body.approval_id) {
        // Send the approval decision through the channel
        if approval_state.sender.send(body.approved).is_err() {
            return Ok(HttpResponse::InternalServerError().json(ToolApprovalResponse {
                success: false,
                message: "Failed to send approval response".to_string(),
            }));
        }

        // If save_decision is true, update the config file
        if body.save_decision {
            let tool_name = &approval_state.tool_name;
            let scope = if body.scope.is_empty() {
                tool_name.clone()
            } else {
                body.scope.clone()
            };

            let mut config = config::Config::load();
            let result = if body.approved {
                config.allow_tool(&scope)
            } else {
                config.deny_tool(&scope)
            };

            if let Err(e) = result {
                return Ok(HttpResponse::InternalServerError().json(ToolApprovalResponse {
                    success: false,
                    message: format!("Approval processed but failed to save to config: {}", e),
                }));
            }
        }

        Ok(HttpResponse::Ok().json(ToolApprovalResponse {
            success: true,
            message: "Approval processed successfully".to_string(),
        }))
    } else {
        Ok(HttpResponse::NotFound().json(ToolApprovalResponse {
            success: false,
            message: "Approval request not found or expired".to_string(),
        }))
    }
}
