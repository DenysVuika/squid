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
  model?: string;
}

export type StreamEventType =
  | 'session'
  | 'sources'
  | 'content'
  | 'reasoning'
  | 'tool_call'
  | 'tool_result'
  | 'tool_approval_request'
  | 'tool_approval_response'
  | 'tool_invocation_completed'
  | 'usage'
  | 'error'
  | 'done';

export interface Source {
  title: string;
  content: string;
}

export interface TokenUsage {
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  reasoning_tokens: number;
  cache_tokens: number;
  context_window: number;
  context_utilization: number;
}

export interface StreamEvent {
  type: StreamEventType;
  session_id?: string;
  sources?: Source[];
  text?: string;
  name?: string;
  arguments?: string;
  result?: string;
  approval_id?: string;
  tool_name?: string;
  tool_args?: Record<string, unknown>;
  tool_description?: string;
  approved?: boolean;
  input_tokens?: number;
  output_tokens?: number;
  reasoning_tokens?: number;
  cache_tokens?: number;
  message?: string;
}

export interface StreamHandlers {
  onSession?: (sessionId: string) => void;
  onSources?: (sources: Source[]) => void;
  onContent: (text: string) => void;
  onReasoning?: (text: string) => void;
  onToolCall?: (name: string, args: string) => void;
  onToolResult?: (name: string, result: string) => void;
  onToolInvocationCompleted?: (tool: {
    name: string;
    arguments: Record<string, unknown>;
    result?: string;
    error?: string;
  }) => void;
  onToolApprovalRequest?: (approval: {
    approval_id: string;
    tool_name: string;
    tool_args: Record<string, unknown>;
    tool_description: string;
  }) => void;
  onToolApprovalResponse?: (approval_id: string, approved: boolean) => void;
  onUsage?: (usage: {
    input_tokens: number;
    output_tokens: number;
    reasoning_tokens: number;
    cache_tokens: number;
  }) => void;
  onError?: (error: string) => void;
  onDone?: () => void;
  signal?: AbortSignal;
}

export interface SessionMessage {
  role: string;
  content: string;
  sources: Source[];
  timestamp: number;
  reasoning?: string;
  tools?: Array<{
    name: string;
    arguments: Record<string, unknown>;
    result?: string;
    error?: string;
  }>;
  thinking_steps?: Array<{
    step_type: string;
    step_order: number;
    content?: string;
    tool_name?: string;
    tool_arguments?: Record<string, unknown>;
    tool_result?: string;
    tool_error?: string;
  }>;
}

export interface SessionData {
  session_id: string;
  messages: SessionMessage[];
  created_at: number;
  updated_at: number;
  title: string | null;
  model_id: string | null;
  token_usage: TokenUsage;
  cost_usd: number;
}

export interface SessionListItem {
  session_id: string;
  message_count: number;
  created_at: number;
  updated_at: number;
  preview: string | null;
  title: string | null;
  model_id: string | null;
  token_usage: TokenUsage;
  cost_usd: number;
}

export interface SessionListResponse {
  sessions: SessionListItem[];
  total: number;
}

export interface ModelInfo {
  id: string;
  name: string;
  max_context_length: number;
  provider: string;
  type?: string;
  pricing_model?: string;
}

export interface ModelsResponse {
  models: ModelInfo[];
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
  const { 
    onSession, 
    onSources, 
    onContent, 
    onReasoning, 
    onToolCall, 
    onToolResult, 
    onToolInvocationCompleted,
    onUsage, 
    onError, 
    onDone, 
    signal 
  } = handlers;

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

              case 'reasoning':
                if (onReasoning && event.text) {
                  onReasoning(event.text);
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

              case 'tool_invocation_completed':
                if (onToolInvocationCompleted && event.name && event.arguments) {
                  onToolInvocationCompleted({
                    name: event.name,
                    arguments: event.arguments,
                    result: event.result,
                    error: event.error,
                  });
                }
                break;

              case 'tool_approval_request':
                if (
                  handlers.onToolApprovalRequest &&
                  event.approval_id &&
                  event.tool_name &&
                  event.tool_args &&
                  event.tool_description
                ) {
                  handlers.onToolApprovalRequest({
                    approval_id: event.approval_id,
                    tool_name: event.tool_name,
                    tool_args: event.tool_args,
                    tool_description: event.tool_description,
                  });
                }
                break;

              case 'tool_approval_response':
                if (
                  handlers.onToolApprovalResponse &&
                  event.approval_id &&
                  event.approved !== undefined
                ) {
                  handlers.onToolApprovalResponse(event.approval_id, event.approved);
                }
                break;

              case 'usage':
                if (onUsage && event.input_tokens !== undefined && event.output_tokens !== undefined) {
                  onUsage({
                    input_tokens: event.input_tokens,
                    output_tokens: event.output_tokens,
                    reasoning_tokens: event.reasoning_tokens || 0,
                    cache_tokens: event.cache_tokens || 0,
                  });
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

/**
 * Load a session's history from the API
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param sessionId - The session ID to load
 * @returns Promise with session data or null if not found
 *
 * @example
 * ```typescript
 * const session = await loadSession('', 'abc-123-def-456');
 * if (session) {
 *   console.log(`Loaded ${session.messages.length} messages`);
 * }
 * ```
 */
export async function loadSession(apiUrl: string, sessionId: string): Promise<SessionData | null> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/sessions/${sessionId}` : `/api/sessions/${sessionId}`;
    const response = await fetch(endpoint);

    if (!response.ok) {
      if (response.status === 404) {
        return null;
      }
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data: SessionData = await response.json();
    return data;
  } catch (error) {
    console.error('Failed to load session:', error);
    return null;
  }
}

/**
 * List all sessions
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @returns Promise with list of sessions
 *
 * @example
 * ```typescript
 * const { sessions, total } = await listSessions('');
 * console.log(`Found ${total} sessions`);
 * ```
 */
export async function listSessions(apiUrl: string): Promise<SessionListResponse> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/sessions` : '/api/sessions';
    const response = await fetch(endpoint);

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data: SessionListResponse = await response.json();
    return data;
  } catch (error) {
    console.error('Failed to list sessions:', error);
    return { sessions: [], total: 0 };
  }
}

/**
 * Delete a session
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param sessionId - The session ID to delete
 * @returns Promise with boolean indicating success
 *
 * @example
 * ```typescript
 * const success = await deleteSession('', 'abc-123-def-456');
 * if (success) {
 *   console.log('Session deleted');
 * }
 * ```
 */
export async function deleteSession(apiUrl: string, sessionId: string): Promise<boolean> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/sessions/${sessionId}` : `/api/sessions/${sessionId}`;
    const response = await fetch(endpoint, {
      method: 'DELETE',
    });

    if (!response.ok) {
      if (response.status === 404) {
        return false;
      }
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return true;
  } catch (error) {
    console.error('Failed to delete session:', error);
    return false;
  }
}

/**
 * Update a session's title
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param sessionId - The session ID to update
 * @param title - The new title for the session
 * @returns Promise with boolean indicating success
 *
 * @example
 * ```typescript
 * const success = await updateSessionTitle('', 'abc-123-def-456', 'My new title');
 * if (success) {
 *   console.log('Session renamed');
 * }
 * ```
 */
export async function updateSessionTitle(apiUrl: string, sessionId: string, title: string): Promise<boolean> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/sessions/${sessionId}` : `/api/sessions/${sessionId}`;
    const response = await fetch(endpoint, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ title }),
    });

    if (!response.ok) {
      if (response.status === 404) {
        return false;
      }
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return true;
  } catch (error) {
    console.error('Failed to update session title:', error);
    return false;
  }
}

/**
 * Fetch available models from the API
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @returns Promise with list of available models
 *
 * @example
 * ```typescript
 * const { models } = await fetchModels('');
 * console.log(`Found ${models.length} models`);
 * ```
 */
export async function fetchModels(apiUrl: string): Promise<ModelsResponse> {
  const endpoint = apiUrl ? `${apiUrl}/api/models` : '/api/models';
  const response = await fetch(endpoint);

  if (!response.ok) {
    throw new Error(`Failed to fetch models: HTTP ${response.status}`);
  }

  const data: ModelsResponse = await response.json();
  return data;
}

export interface ConfigResponse {
  api_url: string;
  api_model: string;
  context_window: number;
}

/**
 * Fetch API configuration including default model
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @returns Promise with API configuration
 *
 * @example
 * ```typescript
 * const config = await fetchConfig('');
 * console.log(`Default model: ${config.api_model}`);
 * ```
 */
export async function fetchConfig(apiUrl: string): Promise<ConfigResponse> {
  const endpoint = apiUrl ? `${apiUrl}/api/config` : '/api/config';
  const response = await fetch(endpoint);

  if (!response.ok) {
    throw new Error(`Failed to fetch config: HTTP ${response.status}`);
  }

  const data: ConfigResponse = await response.json();
  return data;
}

/**
 * Send tool approval response to the backend
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param approval_id - The approval ID from the tool_approval_request event
 * @param approved - Whether the tool execution was approved
 * @param save_decision - Whether to save this decision to config (for "Always"/"Never")
 * @param scope - The scope for saving (e.g., "bash:ls" or just the tool name)
 * @returns Promise with boolean indicating success
 *
 * @example
 * ```typescript
 * const success = await sendToolApproval('', 'approval-123', true, false, '');
 * if (success) {
 *   console.log('Approval sent');
 * }
 * ```
 */
export async function sendToolApproval(
  apiUrl: string,
  approval_id: string,
  approved: boolean,
  save_decision: boolean = false,
  scope: string = ''
): Promise<boolean> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/tool-approval` : '/api/tool-approval';
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        approval_id,
        approved,
        save_decision,
        scope,
      }),
    });

    if (!response.ok) {
      console.error(`Failed to send tool approval: HTTP ${response.status}`);
      return false;
    }

    const data = await response.json();
    return data.success === true;
  } catch (error) {
    console.error('Failed to send tool approval:', error);
    return false;
  }
}

// Re-export React for the hook
import React from 'react';
