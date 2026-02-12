/**
 * Squid Chat API Client
 *
 * Provides streaming chat functionality for the Squid chatbot UI.
 * Connects to the Squid API server and handles Server-Sent Events (SSE).
 */

export interface FileAttachment {
  filename: string;
  content: string;
}

export interface ChatMessage {
  message: string;
  session_id?: string;
  files?: FileAttachment[];
  system_prompt?: string;
}

export type StreamEventType = 'session' | 'sources' | 'content' | 'tool_call' | 'tool_result' | 'error' | 'done';

export interface Source {
  title: string;
}

export interface StreamEvent {
  type: StreamEventType;
  session_id?: string;
  sources?: Source[];
  text?: string;
  name?: string;
  arguments?: string;
  result?: string;
  message?: string;
}

export interface StreamHandlers {
  onSession?: (sessionId: string) => void;
  onSources?: (sources: Source[]) => void;
  onContent: (text: string) => void;
  onToolCall?: (name: string, args: string) => void;
  onToolResult?: (name: string, result: string) => void;
  onError?: (error: string) => void;
  onDone?: () => void;
  signal?: AbortSignal;
}

/**
 * Stream chat messages to the Squid API and receive responses via SSE
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param message - The chat message to send
 * @param handlers - Callbacks for different event types and optional abort signal
 * @returns Promise that resolves when the stream is complete
 *
 * @example
 * ```typescript
 * // When served from the same server (use relative path)
 * await streamChat(
 *   '',
 *   { message: 'Explain async/await in Rust' },
 *   {
 *     onContent: (text) => appendToMessage(text),
 *     onError: (error) => console.error(error),
 *     onDone: () => console.log('Stream completed'),
 *   }
 * );
 *
 * // When calling from external origin
 * await streamChat(
 *   'http://127.0.0.1:3000',
 *   { message: 'Explain async/await in Rust' },
 *   {
 *     onContent: (text) => appendToMessage(text),
 *   }
 * );
 * ```
 */
export async function streamChat(apiUrl: string, message: ChatMessage, handlers: StreamHandlers): Promise<void> {
  const { onSession, onSources, onContent, onToolCall, onToolResult, onError, onDone, signal } = handlers;

  try {
    // If apiUrl is empty, use relative path (same origin)
    const endpoint = apiUrl ? `${apiUrl}/api/chat` : '/api/chat';
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(message),
      signal,
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const reader = response.body?.getReader();
    const decoder = new TextDecoder();

    if (!reader) {
      throw new Error('No reader available');
    }

    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();

      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n');

      // Keep the last incomplete line in the buffer
      buffer = lines.pop() || '';

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          try {
            const event: StreamEvent = JSON.parse(data);

            switch (event.type) {
              case 'session':
                if (onSession && event.session_id) {
                  onSession(event.session_id);
                }
                break;

              case 'sources':
                if (onSources && event.sources) {
                  onSources(event.sources);
                }
                break;

              case 'content':
                if (event.text) {
                  onContent(event.text);
                }
                break;

              case 'tool_call':
                if (onToolCall && event.name && event.arguments) {
                  onToolCall(event.name, event.arguments);
                }
                break;

              case 'tool_result':
                if (onToolResult && event.name && event.result) {
                  onToolResult(event.name, event.result);
                }
                break;

              case 'error':
                if (onError && event.message) {
                  onError(event.message);
                }
                break;

              case 'done':
                if (onDone) {
                  onDone();
                }
                return;
            }
          } catch (e) {
            console.error('Failed to parse SSE data:', e, 'Raw data:', data);
          }
        }
      }
    }
  } catch (error) {
    if (onError) {
      onError(error instanceof Error ? error.message : String(error));
    }
    throw error;
  }
}

/**
 * Example usage hook for React components
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 *
 * @example
 * ```tsx
 * // When served from the same server
 * const { sendMessage, isStreaming } = useChatStream('');
 *
 * const handleSend = async () => {
 *   await sendMessage(
 *     { message: userInput },
 *     {
 *       onContent: (text) => {
 *         setMessages(prev => updateLastMessage(prev, text));
 *       },
 *       onDone: () => {
 *         setIsStreaming(false);
 *       },
 *     }
 *   );
 * };
 * ```
 */
export function useChatStream(apiUrl: string) {
  const [isStreaming, setIsStreaming] = React.useState(false);

  const sendMessage = async (message: ChatMessage, handlers: StreamHandlers) => {
    setIsStreaming(true);
    try {
      await streamChat(apiUrl, message, {
        ...handlers,
        onDone: () => {
          setIsStreaming(false);
          handlers.onDone?.();
        },
        onError: (error) => {
          setIsStreaming(false);
          handlers.onError?.(error);
        },
      });
    } catch (error) {
      setIsStreaming(false);
      throw error;
    }
  };

  return { sendMessage, isStreaming };
}

// Re-export React for the hook
import React from 'react';
