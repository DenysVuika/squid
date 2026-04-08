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
  agent_id: string;
  use_rag?: boolean;
  use_tools?: boolean;
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
  error?: string;
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
  thinking_steps?: Array<{
    step_type: string;
    step_order: number;
    content?: string;
    tool_name?: string;
    tool_arguments?: Record<string, unknown>;
    tool_result?: string;
    tool_error?: string;
    content_before_tool?: string;
  }>;
}

export interface SessionData {
  session_id: string;
  messages: SessionMessage[];
  created_at: number;
  updated_at: number;
  title: string | null;
  agent_id: string | null;
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
  agent_id: string | null;
  token_usage: TokenUsage;
  cost_usd: number;
}

export interface SessionListResponse {
  sessions: SessionListItem[];
  total: number;
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
    signal,
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
                  // Parse arguments if it's a string
                  let parsedArgs: Record<string, unknown>;
                  try {
                    parsedArgs = typeof event.arguments === 'string' ? JSON.parse(event.arguments) : event.arguments;
                  } catch {
                    parsedArgs = {};
                  }

                  onToolInvocationCompleted({
                    name: event.name,
                    arguments: parsedArgs,
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
                if (handlers.onToolApprovalResponse && event.approval_id && event.approved !== undefined) {
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

export interface AgentInfo {
  id: string;
  name: string;
  description: string;
  model: string;
  enabled: boolean;
  use_tools: boolean;
  permissions: {
    allow: string[];
    deny: string[];
  };
  pricing_model?: string;
  suggestions?: string[];
}

export interface AgentsResponse {
  agents: AgentInfo[];
  default_agent: string;
}

export interface AgentTokenStats {
  agent_id: string;
  total_sessions: number;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  reasoning_tokens: number;
  cache_tokens: number;
  total_cost_usd: number;
  avg_cost_per_session: number;
  first_used_at: number;
  last_used_at: number;
}

export interface AllAgentTokenStatsResponse {
  agents: AgentTokenStats[];
}

/**
 * Fetch available agents from the API
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @returns Promise with list of available agents
 *
 * @example
 * ```typescript
 * const { agents, default_agent } = await fetchAgents('');
 * console.log(`Found ${agents.length} agents, default: ${default_agent}`);
 * ```
 */
export async function fetchAgents(apiUrl: string): Promise<AgentsResponse> {
  const endpoint = apiUrl ? `${apiUrl}/api/agents` : '/api/agents';
  const response = await fetch(endpoint);

  if (!response.ok) {
    throw new Error(`Failed to fetch agents: HTTP ${response.status}`);
  }

  const data: AgentsResponse = await response.json();
  return data;
}

/**
 * Fetch token statistics for all agents
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @returns Promise with agent statistics
 *
 * @example
 * ```typescript
 * const stats = await fetchAllAgentStats('');
 * console.log(`Found stats for ${stats.agents.length} agents`);
 * ```
 */
export async function fetchAllAgentStats(apiUrl: string): Promise<AllAgentTokenStatsResponse> {
  const endpoint = apiUrl ? `${apiUrl}/api/agents/stats` : '/api/agents/stats';
  const response = await fetch(endpoint);

  if (!response.ok) {
    throw new Error(`Failed to fetch agent statistics: HTTP ${response.status}`);
  }

  const data: AllAgentTokenStatsResponse = await response.json();
  return data;
}

/**
 * Fetch token statistics for a specific agent
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param agentId - The agent ID to get statistics for
 * @returns Promise with agent statistics or null if not found
 *
 * @example
 * ```typescript
 * const stats = await fetchAgentStats('', 'general-assistant');
 * if (stats) {
 *   console.log(`Total tokens: ${stats.total_tokens}`);
 * }
 * ```
 */
export async function fetchAgentStats(apiUrl: string, agentId: string): Promise<AgentTokenStats | null> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/agents/${agentId}/stats` : `/api/agents/${agentId}/stats`;
    const response = await fetch(endpoint);

    if (!response.ok) {
      if (response.status === 404) {
        return null;
      }
      throw new Error(`Failed to fetch agent statistics: HTTP ${response.status}`);
    }

    const data: AgentTokenStats = await response.json();
    return data;
  } catch (error) {
    console.error('Failed to fetch agent statistics:', error);
    return null;
  }
}

export interface AgentContentResponse {
  id: string;
  name: string;
  content: string;
}

/**
 * Fetch the raw markdown content for a specific agent
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param agentId - The ID of the agent to fetch
 * @returns Promise with agent content
 */
export async function fetchAgentContent(apiUrl: string, agentId: string): Promise<AgentContentResponse> {
  const endpoint = apiUrl ? `${apiUrl}/api/agents/${agentId}/content` : `/api/agents/${agentId}/content`;
  const response = await fetch(endpoint);

  if (!response.ok) {
    throw new Error(`Failed to fetch agent content: HTTP ${response.status}`);
  }

  return response.json();
}

export interface ConfigResponse {
  api_url: string;
  context_window: number;
  rag_enabled: boolean;
  web_sounds: boolean;
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
 * console.log(`Context window: ${config.context_window}`);
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
 * RAG-related API functions
 */

export interface RagSource {
  filename: string;
  text: string;
  relevance: number;
}

export interface RagQueryResponse {
  context: string;
  sources: RagSource[];
}

export interface DocumentSummary {
  id: number;
  filename: string;
  file_size: number;
  updated_at: number;
}

export interface DocumentListResponse {
  documents: DocumentSummary[];
}

export interface RagStatsResponse {
  doc_count: number;
  chunk_count: number;
  embedding_count: number;
  avg_chunks_per_doc: number;
}

export interface RagResponse {
  success: boolean;
  message: string;
}

/**
 * Query RAG index for relevant context
 *
 * @param apiUrl - The base URL of the Squid API
 * @param query - The query text to search for
 * @param topK - Optional number of results to return
 * @returns Promise with context and sources
 */
export async function queryRag(apiUrl: string, query: string, topK?: number): Promise<RagQueryResponse> {
  const endpoint = apiUrl ? `${apiUrl}/api/rag/query` : '/api/rag/query';
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ query, top_k: topK }),
  });

  if (!response.ok) {
    throw new Error(`Failed to query RAG: HTTP ${response.status}`);
  }

  const data: RagQueryResponse = await response.json();
  return data;
}

/**
 * List all indexed documents
 *
 * @param apiUrl - The base URL of the Squid API
 * @returns Promise with list of documents
 */
export async function listRagDocuments(apiUrl: string): Promise<DocumentListResponse> {
  const endpoint = apiUrl ? `${apiUrl}/api/rag/documents` : '/api/rag/documents';
  const response = await fetch(endpoint);

  if (!response.ok) {
    throw new Error(`Failed to list documents: HTTP ${response.status}`);
  }

  const data: DocumentListResponse = await response.json();
  return data;
}

/**
 * Delete a document from RAG index
 *
 * @param apiUrl - The base URL of the Squid API
 * @param filename - The filename to delete
 * @returns Promise with success status
 */
export async function deleteRagDocument(apiUrl: string, filename: string): Promise<RagResponse> {
  const endpoint = apiUrl
    ? `${apiUrl}/api/rag/documents/${encodeURIComponent(filename)}`
    : `/api/rag/documents/${encodeURIComponent(filename)}`;
  const response = await fetch(endpoint, {
    method: 'DELETE',
  });

  if (!response.ok) {
    throw new Error(`Failed to delete document: HTTP ${response.status}`);
  }

  const data: RagResponse = await response.json();
  return data;
}

/**
 * Get RAG statistics
 *
 * @param apiUrl - The base URL of the Squid API
 * @returns Promise with RAG statistics
 */
export async function getRagStats(apiUrl: string): Promise<RagStatsResponse> {
  const endpoint = apiUrl ? `${apiUrl}/api/rag/stats` : '/api/rag/stats';
  const response = await fetch(endpoint);

  if (!response.ok) {
    throw new Error(`Failed to get RAG stats: HTTP ${response.status}`);
  }

  const data: RagStatsResponse = await response.json();
  return data;
}

/**
 * Upload a document to RAG index
 *
 * @param apiUrl - The base URL of the Squid API
 * @param filename - The filename
 * @param content - The document content
 * @returns Promise with success status
 */
export async function uploadRagDocument(apiUrl: string, filename: string, content: string): Promise<RagResponse> {
  const endpoint = apiUrl ? `${apiUrl}/api/rag/upload` : '/api/rag/upload';
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ filename, content }),
  });

  if (!response.ok) {
    throw new Error(`Failed to upload document: HTTP ${response.status}`);
  }

  const data: RagResponse = await response.json();
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

/**
 * Job-related API functions
 */

export interface JobInfo {
  id: number;
  name: string;
  schedule_type: string;
  cron_expression: string | null;
  priority: number;
  max_cpu_percent: number;
  status: string;
  last_run: string | null;
  next_run: string | null;
  retries: number;
  max_retries: number;
  payload: {
    agent_id: string;
    message: string;
    system_prompt?: string;
    file_path?: string;
    session_id?: string;
  };
  result: Record<string, unknown> | null;
  error_message: string | null;
  is_active: boolean;
  timeout_seconds: number;
}

export interface CreateJobRequest {
  name: string;
  schedule_type: string;
  cron_expression?: string;
  priority?: number;
  max_cpu_percent?: number;
  max_retries?: number;
  timeout_seconds?: number;
  payload: {
    agent_id: string;
    message: string;
    system_prompt?: string;
    file_path?: string;
    session_id?: string;
  };
}

export interface JobUpdateEvent {
  type: 'update' | 'deleted';
  job?: JobInfo;
  job_id?: number;
}

/**
 * Fetch all jobs
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @returns Promise with list of jobs
 *
 * @example
 * ```typescript
 * const jobs = await fetchJobs('');
 * console.log(`Found ${jobs.length} jobs`);
 * ```
 */
export async function fetchJobs(apiUrl: string): Promise<JobInfo[]> {
  const endpoint = apiUrl ? `${apiUrl}/api/jobs` : '/api/jobs';
  const response = await fetch(endpoint);

  if (!response.ok) {
    throw new Error(`Failed to fetch jobs: HTTP ${response.status}`);
  }

  const data: JobInfo[] = await response.json();
  return data;
}

/**
 * Fetch a single job by ID
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param jobId - The job ID to fetch
 * @returns Promise with job details or null if not found
 */
export async function fetchJob(apiUrl: string, jobId: number): Promise<JobInfo | null> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/jobs/${jobId}` : `/api/jobs/${jobId}`;
    const response = await fetch(endpoint);

    if (!response.ok) {
      if (response.status === 404) {
        return null;
      }
      throw new Error(`Failed to fetch job: HTTP ${response.status}`);
    }

    const data: JobInfo = await response.json();
    return data;
  } catch (error) {
    console.error('Failed to fetch job:', error);
    return null;
  }
}

/**
 * Create a new job
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param job - The job creation request
 * @returns Promise with created job
 */
export async function createJob(apiUrl: string, job: CreateJobRequest): Promise<JobInfo> {
  const endpoint = apiUrl ? `${apiUrl}/api/jobs` : '/api/jobs';
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(job),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || `Failed to create job: HTTP ${response.status}`);
  }

  const data: JobInfo = await response.json();
  return data;
}

/**
 * Delete a job
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param jobId - The job ID to delete
 * @returns Promise with boolean indicating success
 */
export async function deleteJob(apiUrl: string, jobId: number): Promise<boolean> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/jobs/${jobId}` : `/api/jobs/${jobId}`;
    const response = await fetch(endpoint, {
      method: 'DELETE',
    });

    if (!response.ok) {
      if (response.status === 404) {
        return false;
      }
      throw new Error(`Failed to delete job: HTTP ${response.status}`);
    }

    return true;
  } catch (error) {
    console.error('Failed to delete job:', error);
    return false;
  }
}

/**
 * Cancel a job (soft cancel - sets status to cancelled but keeps in database)
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param jobId - The job ID to cancel
 * @returns Promise with boolean indicating success
 */
export async function cancelJob(apiUrl: string, jobId: number): Promise<boolean> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/jobs/${jobId}/cancel` : `/api/jobs/${jobId}/cancel`;
    const response = await fetch(endpoint, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error(`Failed to cancel job: HTTP ${response.status}`);
    }

    return true;
  } catch (error) {
    console.error('Failed to cancel job:', error);
    return false;
  }
}

/**
 * Pause a job
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param jobId - The job ID to pause
 * @returns Promise with boolean indicating success
 */
export async function pauseJob(apiUrl: string, jobId: number): Promise<boolean> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/jobs/${jobId}/pause` : `/api/jobs/${jobId}/pause`;
    const response = await fetch(endpoint, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error(`Failed to pause job: HTTP ${response.status}`);
    }

    return true;
  } catch (error) {
    console.error('Failed to pause job:', error);
    return false;
  }
}

/**
 * Resume a job
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param jobId - The job ID to resume
 * @returns Promise with boolean indicating success
 */
export async function resumeJob(apiUrl: string, jobId: number): Promise<boolean> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/jobs/${jobId}/resume` : `/api/jobs/${jobId}/resume`;
    const response = await fetch(endpoint, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error(`Failed to resume job: HTTP ${response.status}`);
    }

    return true;
  } catch (error) {
    console.error('Failed to resume job:', error);
    return false;
  }
}

/**
 * Manually trigger a job
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param jobId - The job ID to trigger
 * @returns Promise with boolean indicating success
 */
export async function triggerJob(apiUrl: string, jobId: number): Promise<boolean> {
  try {
    const endpoint = apiUrl ? `${apiUrl}/api/jobs/${jobId}/trigger` : `/api/jobs/${jobId}/trigger`;
    const response = await fetch(endpoint, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error(`Failed to trigger job: HTTP ${response.status}`);
    }

    return true;
  } catch (error) {
    console.error('Failed to trigger job:', error);
    return false;
  }
}

export interface JobUpdateHandlers {
  onJobUpdate: (job: JobInfo) => void;
  onJobDeleted: (jobId: number) => void;
  onError?: (error: string) => void;
}

/**
 * Subscribe to job updates via Server-Sent Events
 *
 * @param apiUrl - The base URL of the Squid API. Use empty string '' for relative path (same origin)
 * @param handlers - Handlers for job update events
 * @returns EventSource instance for managing the connection
 *
 * @example
 * ```typescript
 * const eventSource = subscribeToJobUpdates('', {
 *   onJobUpdate: (job) => console.log('Job updated:', job),
 *   onJobDeleted: (jobId) => console.log('Job deleted:', jobId),
 *   onError: (error) => console.error('SSE error:', error),
 * });
 *
 * // Later, close the connection
 * eventSource.close();
 * ```
 */
export function subscribeToJobUpdates(apiUrl: string, handlers: JobUpdateHandlers): EventSource {
  const endpoint = apiUrl ? `${apiUrl}/api/jobs/events` : '/api/jobs/events';
  const eventSource = new EventSource(endpoint);

  eventSource.addEventListener('job_update', (event) => {
    try {
      const data: JobUpdateEvent = JSON.parse(event.data);
      if (data.type === 'update' && data.job) {
        handlers.onJobUpdate(data.job);
      } else if (data.type === 'deleted' && data.job_id) {
        handlers.onJobDeleted(data.job_id);
      }
    } catch (error) {
      console.error('Failed to parse job update event:', error);
      handlers.onError?.('Failed to parse job update');
    }
  });

  eventSource.onerror = (error) => {
    console.error('Job SSE connection error:', error);
    handlers.onError?.('SSE connection error');
  };

  return eventSource;
}

// Re-export React for the hook
import React from 'react';
