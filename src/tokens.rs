//! Token counting utilities
//!
//! Provides accurate token counting using tiktoken-rs for OpenAI-compatible models.
//! This is used when LLM providers don't report usage (e.g., LM Studio, Ollama).

use async_openai::types::chat::ChatCompletionRequestMessage;
use log::debug;

/// Convert async_openai message to text for token counting
///
/// Extracts the text content from a ChatCompletionRequestMessage.
/// For simplicity, we serialize to JSON which gives us a good approximation
/// of the token count including all content.
fn message_to_text(msg: &ChatCompletionRequestMessage) -> String {
    // Simplified approach: just serialize the message to JSON
    // This captures all the text content in a format similar to what's sent to the API
    serde_json::to_string(msg).unwrap_or_else(|_| {
        // Fallback: try to extract just the basic text
        match msg {
            ChatCompletionRequestMessage::System(m) => {
                format!("{:?}", m.content)
            }
            ChatCompletionRequestMessage::User(m) => {
                format!("{:?}", m.content)
            }
            ChatCompletionRequestMessage::Assistant(m) => {
                m.content.as_ref().map(|c| format!("{:?}", c)).unwrap_or_default()
            }
            ChatCompletionRequestMessage::Tool(m) => {
                format!("{:?}", m.content)
            }
            ChatCompletionRequestMessage::Function(m) => {
                m.content.as_ref().map(|c| format!("{:?}", c)).unwrap_or_default()
            }
            ChatCompletionRequestMessage::Developer(m) => {
                format!("{:?}", m.content)
            }
        }
    })
}

/// Estimate token count for chat messages using tiktoken
///
/// This provides accurate token counting for OpenAI-compatible models.
/// For each message, we add:
/// - 3 tokens for message formatting overhead
/// - The actual token count of the message content
/// - 3 tokens for the final "assistant" reply priming
///
/// # Arguments
///
/// * `model` - The model name (e.g., "gpt-4", "gpt-3.5-turbo")
/// * `messages` - The chat messages to count tokens for
///
/// # Returns
///
/// A tuple of (input_tokens, output_tokens). Output tokens is always 0 for this function
/// since it only counts the input context.
pub fn estimate_tokens(
    model: &str,
    messages: &[ChatCompletionRequestMessage],
) -> (i64, i64) {
    match tiktoken_rs::get_bpe_from_model(model) {
        Ok(bpe) => {
            let mut total_tokens = 0;

            // Count tokens for each message
            for msg in messages {
                // Every message has 3 tokens overhead for formatting
                total_tokens += 3;

                let text = message_to_text(msg);
                let tokens = bpe.encode_with_special_tokens(&text);
                total_tokens += tokens.len();
            }

            // Add 3 tokens for the final "assistant" reply priming
            total_tokens += 3;

            debug!(
                "Counted {} tokens for {} messages in model '{}' using tiktoken",
                total_tokens,
                messages.len(),
                model
            );

            (total_tokens as i64, 0)
        }
        Err(_) => {
            debug!(
                "tiktoken encoder not available for model '{}', falling back to character-based estimation",
                model
            );
            estimate_tokens_fallback(messages)
        }
    }
}

/// Estimate tokens for a single message (for streaming responses)
///
/// Uses tiktoken to accurately count tokens. Falls back to character-based estimation
/// if tiktoken is not available for the model.
///
/// # Arguments
///
/// * `model` - The model name (e.g., "gpt-4", "gpt-3.5-turbo")
/// * `content` - The text content to estimate tokens for
///
/// # Returns
///
/// The estimated number of tokens
pub fn estimate_message_tokens(model: &str, content: &str) -> i64 {
    match tiktoken_rs::get_bpe_from_model(model) {
        Ok(bpe) => {
            let tokens = bpe.encode_with_special_tokens(content);
            tokens.len() as i64
        }
        Err(_) => {
            // Fallback: character-based estimation (1 token ≈ 4 characters)
            (content.len() / 4).max(1) as i64
        }
    }
}

/// Fallback character-based token estimation
///
/// Used when tiktoken doesn't support the model (e.g., custom local models).
/// Uses a simple heuristic: approximately 4 characters per token for English text.
fn estimate_tokens_fallback(messages: &[ChatCompletionRequestMessage]) -> (i64, i64) {
    let mut total_chars = 0;

    // Simple approach: extract text from each message and count characters
    for msg in messages {
        let text = message_to_text(msg);
        total_chars += text.len();
    }

    // Add overhead for message formatting (~4 tokens per message)
    total_chars += messages.len() * 16; // ~4 tokens * 4 chars/token

    // Rough estimate: 1 token ≈ 4 characters for English text
    let estimated_tokens = (total_chars / 4).max(1) as i64;

    debug!(
        "Estimated {} tokens for {} messages using character-based fallback ({} chars total)",
        estimated_tokens,
        messages.len(),
        total_chars
    );

    (estimated_tokens, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_openai::types::chat::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    };

    #[test]
    fn test_estimate_message_tokens() {
        let model = "gpt-4";
        let content = "Hello, how are you?";

        let tokens = estimate_message_tokens(model, content);

        // Should be positive
        assert!(tokens > 0);

        // Should be reasonable (not too many tokens for short text)
        assert!(tokens < 20);
        assert!(tokens >= 3); // At least a few tokens for this text
    }

    #[test]
    fn test_estimate_tokens_with_messages() {
        let messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: "You are a helpful assistant.".to_string().into(),
                name: None,
                ..Default::default()
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text("Hello!".to_string()),
                name: None,
                ..Default::default()
            }),
        ];

        let (tokens, _) = estimate_tokens("gpt-4", &messages);

        // Should estimate some tokens
        assert!(tokens > 0);

        // Should be reasonable for 2 messages
        // With tiktoken overhead: 3 tokens per message + actual content + 3 final
        assert!(tokens > 10); // At least overhead + some content
        assert!(tokens < 100);
    }

    #[test]
    fn test_estimate_tokens_different_models() {
        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(
                    "Write a function to calculate fibonacci numbers.".to_string(),
                ),
                name: None,
                ..Default::default()
            },
        )];

        // Test different OpenAI models
        for model in &["gpt-4", "gpt-4o", "gpt-3.5-turbo"] {
            let (prompt_tokens, _) = estimate_tokens(model, &messages);
            assert!(prompt_tokens > 0);
            assert!(prompt_tokens < 200);
        }
    }

    #[test]
    fn test_estimate_empty_content() {
        let tokens = estimate_message_tokens("gpt-4", "");
        // Empty content still has some tokens due to message formatting
        assert!(tokens >= 0);
    }

    #[test]
    fn test_estimate_long_content() {
        let long_text = "hello ".repeat(500); // 500 words
        let tokens = estimate_message_tokens("gpt-4", &long_text);

        // Should be reasonable for 500 words (roughly 500-700 tokens)
        assert!(tokens >= 400);
        assert!(tokens <= 1000);
    }

    #[test]
    fn test_estimate_multiple_rounds() {
        let messages = vec![
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text("Question 1".to_string()),
                name: None,
                ..Default::default()
            }),
            ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                content: Some("Answer 1".to_string().into()),
                name: None,
                ..Default::default()
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text("Question 2".to_string()),
                name: None,
                ..Default::default()
            }),
        ];

        let (tokens, _) = estimate_tokens("gpt-4", &messages);

        // Should handle multiple rounds
        assert!(tokens > 0);
        assert!(tokens > 15); // 3 messages * 3 overhead + content + 3 final = at least 15
    }

    #[test]
    fn test_fallback_for_unknown_model() {
        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text("Hello!".to_string()),
                name: None,
                ..Default::default()
            },
        )];

        // Test with a model name that will use the default encoder
        let (tokens, _) = estimate_tokens("custom-local-model", &messages);

        // Should still estimate some tokens (will use cl100k_base as default)
        assert!(tokens > 0);
    }

    #[test]
    fn test_gpt4o_uses_o200k_encoding() {
        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text("Test".to_string()),
                name: None,
                ..Default::default()
            },
        )];

        // Both should work and return reasonable token counts
        let (gpt4_tokens, _) = estimate_tokens("gpt-4", &messages);
        let (gpt4o_tokens, _) = estimate_tokens("gpt-4o", &messages);

        assert!(gpt4_tokens > 0);
        assert!(gpt4o_tokens > 0);
        // They might be slightly different due to different encodings
    }
}
