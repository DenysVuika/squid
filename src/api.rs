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

use crate::{config, envinfo, llm, tools};

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub file_content: Option<String>,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub system_prompt: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "content")]
    Content { text: String },
    #[serde(rename = "tool_call")]
    ToolCall { name: String, arguments: String },
    #[serde(rename = "tool_result")]
    ToolResult { name: String, result: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "done")]
    Done,
}

/// Handles streaming chat requests
pub async fn chat_stream(
    body: web::Json<ChatRequest>,
    app_config: web::Data<Arc<config::Config>>,
) -> Result<HttpResponse, Error> {
    debug!("Received chat request: {:?}", body);

    let question = body.message.clone();
    let file_content = body.file_content.clone();
    let file_path = body.file_path.clone();
    let system_prompt = body.system_prompt.clone();
    let app_config_clone = app_config.get_ref().clone();

    // Create SSE stream
    let stream = async_stream::stream! {
        match create_chat_stream(
            &question,
            file_content.as_deref(),
            file_path.as_deref(),
            system_prompt.as_deref(),
            &app_config_clone,
        ).await {
            Ok(content_stream) => {
                // Stream each content chunk as it arrives
                let mut pinned_stream = Box::pin(content_stream);
                while let Some(result) = pinned_stream.next().await {
                    match result {
                        Ok(chunk) => {
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
    question: &str,
    file_content: Option<&str>,
    file_path: Option<&str>,
    system_prompt: Option<&str>,
    app_config: &config::Config,
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

    let user_message = if let Some(content) = file_content {
        let file_info = if let Some(path) = file_path {
            format!("the file '{}'", path)
        } else {
            "the file".to_string()
        };
        format!(
            "{}\n\nHere is the content of {}:\n\n```\n{}\n```\n\nUser query: {}",
            env_context, file_info, content, question
        )
    } else {
        format!("{}\n\nUser query: {}", env_context, question)
    };

    let default_prompt = llm::combine_prompts(llm::get_ask_prompt());
    let system_message = system_prompt.unwrap_or(&default_prompt);

    debug!("System message:\n{}", system_message);
    debug!("User message:\n{}", user_message);

    let mut messages: Vec<ChatCompletionRequestMessage> = vec![
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

                // Log token usage statistics from streaming response
                if let Some(usage) = &response.usage {
                    debug!(
                        "Token usage - Prompt: {}, Completion: {}, Total: {}",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                    );
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
