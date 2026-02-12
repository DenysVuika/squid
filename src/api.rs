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
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{config, envinfo, llm, logger, session, tools};

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
}

#[derive(Debug, Serialize, Clone)]
pub struct Source {
    pub title: String,
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
}

#[derive(Debug, Serialize)]
pub struct TokenUsageResponse {
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_tokens: i64,
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
                messages: session.messages.iter().map(|msg| SessionMessage {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                    sources: msg.sources.iter().map(|s| Source {
                        title: s.title.clone(),
                    }).collect(),
                    timestamp: msg.timestamp,
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
) -> Result<HttpResponse, Error> {
    debug!("Received chat request: {:?}", body);

    let question = body.message.clone();
    let files: Vec<session::FileAttachment> = body.files.iter().map(|f| session::FileAttachment {
        filename: f.filename.clone(),
        content: f.content.clone(),
    }).collect();
    let system_prompt = body.system_prompt.clone();
    let app_config_clone = app_config.get_ref().clone();
    let session_manager_clone = session_manager.get_ref().clone();
    let model_id = app_config.api_model.clone();

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
            &app_config_clone,
            &session_manager_clone,
        ).await {
            Ok(content_stream) => {
                // Accumulate assistant content and token usage as we stream
                let mut accumulated_content = String::new();
                let mut total_input_tokens = 0i64;
                let mut total_output_tokens = 0i64;
                let mut total_reasoning_tokens = 0i64;
                let mut total_cache_tokens = 0i64;

                // Stream each content chunk as it arrives
                let mut pinned_stream = Box::pin(content_stream);
                while let Some(result) = pinned_stream.next().await {
                    match result {
                        Ok(chunk) => {
                            // Accumulate content chunks
                            if let StreamEvent::Content { ref text } = chunk {
                                accumulated_content.push_str(text);
                            }

                            // Accumulate token usage
                            if let StreamEvent::Usage { input_tokens, output_tokens, reasoning_tokens, cache_tokens } = chunk {
                                total_input_tokens += input_tokens;
                                total_output_tokens += output_tokens;
                                total_reasoning_tokens += reasoning_tokens;
                                total_cache_tokens += cache_tokens;
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
                if !accumulated_content.is_empty() {
                    debug!("Saving assistant message to session {} (length: {} chars)", session_id, accumulated_content.len());
                    match session_manager_clone.add_assistant_message(
                        &session_id,
                        accumulated_content,
                        sources,
                    ) {
                        Ok(_) => debug!("Assistant message saved successfully"),
                        Err(e) => debug!("Failed to save assistant message: {}", e),
                    }
                } else {
                    debug!("Skipping empty assistant message for session {}", session_id);
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

async fn create_chat_stream(
    session_id: &str,
    question: &str,
    files: &[session::FileAttachment],
    system_prompt: Option<&str>,
    app_config: &config::Config,
    session_manager: &session::SessionManager,
) -> Result<
    impl futures::Stream<Item = Result<StreamEvent, Box<dyn std::error::Error + Send + Sync>>>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    debug!("Using API URL: {}", app_config.api_url);
    debug!("Using API Model: {}", app_config.api_model);

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
            .model(&app_config.api_model)
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

                            // Execute tools
                            for tool_call in tool_calls.iter() {
                                let name = tool_call.function.name.clone();
                                let args = tool_call.function.arguments.clone();
                                let tool_call_id = tool_call.id.clone();

                                let result = tools::call_tool(&name, &args, app_config).await;

                                messages.push(
                                    ChatCompletionRequestToolMessage {
                                        content: result.to_string().into(),
                                        tool_call_id,
                                    }
                                    .into(),
                                );
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
