import { create } from 'zustand';
import type { FileUIPart } from 'ai';
import { streamChat, loadSession, sendToolApproval, type Source } from '@/lib/chat-api';
import { toast } from 'sonner';
import { useSessionStore } from './session-store';
import { useModelStore } from './model-store';

export interface ToolApproval {
  approval_id: string;
  tool_name: string;
  tool_args: Record<string, unknown>;
  tool_description: string;
  message_id: string; // Associated message ID
}

export interface ToolApprovalDecision {
  approval_id: string;
  approved: boolean;
  timestamp: number;
}

export interface ReasoningStep {
  type: 'reasoning';
  content: string;
}

export interface ToolStep {
  type: 'tool';
  name: string;
  description: string;
  status: string;
  parameters: Record<string, unknown>;
  result: string | undefined;
  error: string | undefined;
}

export type ThinkingStep = ReasoningStep | ToolStep;

export interface MessageType {
  key: string;
  from: 'user' | 'assistant';
  sources?: { href: string; title: string; content: string }[];
  versions: {
    id: string;
    content: string;
  }[];
  // Legacy reasoning field (for backward compatibility with DB)
  reasoning?: {
    content: string;
    duration?: number;
  };
  // New chain-of-thought steps
  thinkingSteps?: ThinkingStep[];
  tools?: {
    name: string;
    description: string;
    status: string;
    parameters: Record<string, unknown>;
    result: string | undefined;
    error: string | undefined;
  }[];
  toolApprovals?: ToolApproval[];
}

interface ChatStore {
  // State
  messages: MessageType[];
  status: 'submitted' | 'streaming' | 'ready' | 'error';
  streamingMessageId: string | null;
  streamingContentRef: string;
  streamingReasoningRef: string;
  isReasoningStreaming: boolean;
  abortController: AbortController | null;
  useWebSearch: boolean;
  pendingApprovals: Map<string, ToolApproval>;
  toolApprovalDecisions: Map<string, ToolApprovalDecision>;

  // Actions
  addUserMessage: (content: string, files?: FileUIPart[]) => void;
  updateMessageContent: (messageId: string, newContent: string) => void;
  setStatus: (status: 'submitted' | 'streaming' | 'ready' | 'error') => void;
  setStreamingMessageId: (messageId: string | null) => void;
  streamResponse: (messageId: string, userMessage: string, files?: FileUIPart[]) => Promise<void>;
  stopStreaming: () => void;
  loadSessionHistory: (sessionId: string) => Promise<void>;
  clearMessages: () => void;
  toggleWebSearch: () => void;
  updateStreamingContent: (content: string) => void;
  setIsReasoningStreaming: (isStreaming: boolean) => void;
  addPendingApproval: (approval: ToolApproval) => void;
  respondToApproval: (approval_id: string, approved: boolean, save_decision: boolean, scope?: string) => Promise<void>;
  clearApproval: (approval_id: string) => void;
}

export const useChatStore = create<ChatStore>((set, get) => ({
  // Initial state
  messages: [],
  status: 'ready',
  streamingMessageId: null,
  streamingContentRef: '',
  streamingReasoningRef: '',
  isReasoningStreaming: false,
  abortController: null,
  useWebSearch: false,
  pendingApprovals: new Map(),
  toolApprovalDecisions: new Map(),

  // Add user message and trigger streaming
  addUserMessage: (content: string, files?: FileUIPart[]) => {
    const userMessage: MessageType = {
      from: 'user',
      key: `user-${Date.now()}`,
      versions: [
        {
          content,
          id: `user-${Date.now()}`,
        },
      ],
    };

    set((state) => ({ messages: [...state.messages, userMessage] }));

    setTimeout(() => {
      const assistantMessageId = `assistant-${Date.now()}`;

      const assistantMessage: MessageType = {
        from: 'assistant',
        key: `assistant-${Date.now()}`,
        versions: [
          {
            content: '',
            id: assistantMessageId,
          },
        ],
      };

      set((state) => ({ messages: [...state.messages, assistantMessage] }));
      get().streamResponse(assistantMessageId, content, files);
    }, 500);
  },

  // Update message content
  updateMessageContent: (messageId: string, newContent: string) => {
    set((state) => ({
      messages: state.messages.map((msg) => {
        if (msg.versions.some((v) => v.id === messageId)) {
          return {
            ...msg,
            versions: msg.versions.map((v) => (v.id === messageId ? { ...v, content: newContent } : v)),
          };
        }
        return msg;
      }),
    }));
  },

  // Set status
  setStatus: (status) => {
    set({ status });
  },

  // Set streaming message ID
  setStreamingMessageId: (messageId) => {
    set({ streamingMessageId: messageId });
  },

  // Update streaming content ref
  updateStreamingContent: (content: string) => {
    set({ streamingContentRef: content });
  },

  // Set reasoning streaming state
  setIsReasoningStreaming: (isStreaming: boolean) => {
    set({ isReasoningStreaming: isStreaming });
  },

  // Stream response from API
  streamResponse: async (messageId: string, userMessage: string, files?: FileUIPart[]) => {
    // Create new abort controller
    const abortController = new AbortController();
    set({ 
      abortController,
      status: 'streaming',
      streamingMessageId: messageId,
      streamingContentRef: '',
      streamingReasoningRef: '',
      isReasoningStreaming: false,
    });

    const sessionStore = useSessionStore.getState();
    const modelStore = useModelStore.getState();

    try {
      // Read file contents if files are attached
      const fileAttachments = [];
      if (files && files.length > 0) {
        for (const file of files) {
          if (file.type === 'file' && file.url) {
            const fileName = 'filename' in file ? String(file.filename) : 'attachment';
            try {
              const response = await fetch(file.url);
              if (response.ok) {
                const content = await response.text();
                fileAttachments.push({
                  filename: fileName,
                  content,
                });
              } else {
                console.error('Failed to fetch file:', response.statusText);
                toast.error('Failed to read file', {
                  description: `Could not read ${fileName}`,
                });
              }
            } catch (e) {
              console.error('Failed to read file:', e);
              toast.error('Failed to read file', {
                description: e instanceof Error ? e.message : String(e),
              });
            }
          }
        }
      }

      await streamChat(
        '',
        {
          message: userMessage,
          session_id: sessionStore.activeSessionId || undefined,
          files: fileAttachments,
          model: modelStore.selectedModel || undefined,
        },
        {
          signal: abortController.signal,
          onSession: (newSessionId) => {
            sessionStore.setActiveSession(newSessionId);
          },
          onSources: (sources: Source[]) => {
            set((state) => ({
              messages: state.messages.map((msg) => {
                if (msg.versions.some((v) => v.id === messageId)) {
                  return {
                    ...msg,
                    sources: sources.map((s) => ({
                      href: '#',
                      title: s.title,
                      content: s.content,
                    })),
                  };
                }
                return msg;
              }),
            }));
          },
          onContent: (text) => {
            const state = get();
            const fullContent = state.streamingContentRef + text;
            set({ streamingContentRef: fullContent });

            // Parse out ALL <think> tags and build thinking steps
            // Keep each <think> block as a SEPARATE step (don't merge)
            let displayContent = '';
            const thinkingSteps: ThinkingStep[] = [];
            let hasOpenTag = false;
            let currentPos = 0;

            // Find all <think> and </think> tags
            while (currentPos < fullContent.length) {
              const thinkStart = fullContent.indexOf('<think>', currentPos);
              
              if (thinkStart === -1) {
                // No more <think> tags, add remaining content
                const remaining = fullContent.substring(currentPos);
                if (!hasOpenTag) {
                  displayContent += remaining;
                }
                break;
              }

              // Add content before <think> to display (only if no open tag)
              if (!hasOpenTag) {
                displayContent += fullContent.substring(currentPos, thinkStart);
              }

              // Look for closing </think>
              const thinkEnd = fullContent.indexOf('</think>', thinkStart);
              
              if (thinkEnd === -1) {
                // Opening tag without closing - reasoning is streaming
                const reasoningText = fullContent.substring(thinkStart + 7);
                if (reasoningText) {
                  // Add as a new reasoning step (don't merge)
                  thinkingSteps.push({
                    type: 'reasoning',
                    content: reasoningText,
                  });
                }
                hasOpenTag = true;
                break;
              }

              // Complete <think>...</think> block found
              const reasoningText = fullContent.substring(thinkStart + 7, thinkEnd);
              if (reasoningText) {
                // Add each closed <think> block as a SEPARATE reasoning step
                thinkingSteps.push({
                  type: 'reasoning',
                  content: reasoningText,
                });
              }
              
              // Reset hasOpenTag since we found the closing tag
              hasOpenTag = false;
              
              // Move past the closing tag
              currentPos = thinkEnd + 8;
            }

            const isReasoningStreamingNow = hasOpenTag;

            // Control reasoning streaming state
            if (thinkingSteps.length > 0 && !state.isReasoningStreaming && isReasoningStreamingNow) {
              set({ isReasoningStreaming: true });
            } else if (!isReasoningStreamingNow && state.isReasoningStreaming) {
              set({ isReasoningStreaming: false });
            }

            set((state) => ({
              messages: state.messages.map((msg) => {
                const hasVersion = msg.versions.some((v) => v.id === messageId);
                if (hasVersion) {
                  // For backward compatibility, keep the old reasoning format too
                  const reasoningContent = thinkingSteps
                    .filter(s => s.type === 'reasoning')
                    .map(s => s.content)
                    .join('\n\n');
                  
                  // Build final thinking steps by intelligently merging:
                  // 1. Keep existing steps that are already there (both reasoning and tools)
                  // 2. Update reasoning steps with new content from current parse
                  // 3. Preserve tool step positions
                  
                  const currentSteps = msg.thinkingSteps || [];
                  const finalSteps: ThinkingStep[] = [];
                  
                  // Strategy: Replace reasoning steps, keep tool steps in their positions
                  let reasoningIdx = 0;
                  
                  for (const step of currentSteps) {
                    if (step.type === 'reasoning') {
                      // Replace with new reasoning content if available
                      if (reasoningIdx < thinkingSteps.length) {
                        finalSteps.push(thinkingSteps[reasoningIdx]);
                        reasoningIdx++;
                      }
                    } else {
                      // Keep tool steps in their original positions
                      finalSteps.push(step);
                    }
                  }
                  
                  // Add any remaining new reasoning steps
                  while (reasoningIdx < thinkingSteps.length) {
                    finalSteps.push(thinkingSteps[reasoningIdx]);
                    reasoningIdx++;
                  }
                  
                  return {
                    ...msg,
                    versions: msg.versions.map((v) => (v.id === messageId ? { ...v, content: displayContent } : v)),
                    reasoning: reasoningContent
                      ? {
                          content: reasoningContent,
                        }
                      : msg.reasoning,
                    thinkingSteps: finalSteps.length > 0 ? finalSteps : undefined,
                  };
                }
                return msg;
              }),
            }));
          },
          onUsage: (usage) => {
            modelStore.updateTokenUsage({
              total_tokens:
                modelStore.tokenUsage.total_tokens +
                usage.input_tokens +
                usage.output_tokens +
                usage.reasoning_tokens +
                usage.cache_tokens,
              input_tokens: modelStore.tokenUsage.input_tokens + usage.input_tokens,
              output_tokens: modelStore.tokenUsage.output_tokens + usage.output_tokens,
              reasoning_tokens: modelStore.tokenUsage.reasoning_tokens + usage.reasoning_tokens,
              cache_tokens: modelStore.tokenUsage.cache_tokens + usage.cache_tokens,
            });
          },
          onToolInvocationCompleted: (tool) => {
            // Add tool step to thinking steps immediately when it completes
            set((state) => ({
              messages: state.messages.map((msg) => {
                if (msg.versions.some((v) => v.id === messageId)) {
                  const currentSteps = msg.thinkingSteps || [];
                  const newToolStep: ThinkingStep = {
                    type: 'tool',
                    name: tool.name,
                    description: '',
                    status: tool.error ? 'error' : 'completed',
                    parameters: tool.arguments,
                    result: tool.result,
                    error: tool.error,
                  };
                  
                  return {
                    ...msg,
                    thinkingSteps: [...currentSteps, newToolStep],
                  };
                }
                return msg;
              }),
            }));
          },
          onToolApprovalRequest: (approval) => {
            // Add pending approval
            get().addPendingApproval({
              ...approval,
              message_id: messageId,
            });
          },
          onToolApprovalResponse: (approval_id) => {
            // Clear the approval from pending
            get().clearApproval(approval_id);
          },
          onError: (error) => {
            console.error('Stream error:', error);
            get().updateMessageContent(messageId, `Error: ${error}`);
            toast.error('Failed to get response', {
              description: error,
            });
            set({ 
              status: 'ready',
              streamingMessageId: null,
              abortController: null,
            });
          },
          onDone: async () => {
            set({
              streamingContentRef: '',
              streamingReasoningRef: '',
              isReasoningStreaming: false,
              abortController: null,
              status: 'ready',
              streamingMessageId: null,
            });

            // Refresh sessions list (to update sidebar)
            sessionStore.refreshSessions();
            
            // Reload the full session to get the complete message with tools
            if (sessionStore.activeSessionId) {
              try {
                const session = await loadSession('', sessionStore.activeSessionId);
                if (session) {
                  modelStore.updateTokenUsage(session.token_usage);
                  
                  // Find the assistant message we just created and update it with tools
                  const lastMessage = session.messages[session.messages.length - 1];
                  if (lastMessage && lastMessage.role === 'assistant' && lastMessage.tools) {
                    console.log('[Chain] Loading tools from session:', lastMessage.tools);
                    
                    // Update the message with thinking steps that include tools
                    set((state) => ({
                      messages: state.messages.map((msg) => {
                        if (msg.versions.some((v) => v.id === messageId)) {
                          const thinkingSteps: ThinkingStep[] = [];
                          
                          // Add existing reasoning steps
                          const existingReasoningSteps = (msg.thinkingSteps || []).filter(s => s.type === 'reasoning');
                          thinkingSteps.push(...existingReasoningSteps);
                          
                          // Add tool steps from the session
                          if (lastMessage.tools) {
                            lastMessage.tools.forEach((t) => {
                              thinkingSteps.push({
                                type: 'tool',
                                name: t.name,
                                description: '',
                                status: t.error ? 'error' : 'completed',
                                parameters: typeof t.arguments === 'object' ? t.arguments : {},
                                result: t.result,
                                error: t.error,
                              });
                            });
                          }
                          
                          console.log('[Chain] Final thinking steps:', thinkingSteps);
                          
                          // Convert tools to proper format (we already checked lastMessage.tools exists)
                          const convertedTools = lastMessage.tools!.map((t) => ({
                            name: t.name,
                            description: '',
                            status: t.error ? 'error' : 'completed',
                            parameters: typeof t.arguments === 'object' ? t.arguments : {},
                            result: t.result,
                            error: t.error,
                          }));
                          
                          return {
                            ...msg,
                            thinkingSteps: thinkingSteps.length > 0 ? thinkingSteps : undefined,
                            tools: convertedTools,
                          };
                        }
                        return msg;
                      }),
                    }));
                  }
                }
              } catch (error) {
                console.error('Failed to reload session:', error);
              }
            }
          },
        }
      );
    } catch (error) {
      const state = get();
      
      // Don't show error if it was aborted by user
      if (error instanceof Error && error.name === 'AbortError') {
        get().updateMessageContent(messageId, state.streamingContentRef || 'Response stopped by user.');
      } else {
        console.error('Chat error:', error);
        get().updateMessageContent(messageId, `Error: ${error instanceof Error ? error.message : String(error)}`);
        toast.error('Failed to send message', {
          description: error instanceof Error ? error.message : String(error),
        });
      }

      set({ 
        abortController: null,
        status: 'ready',
        streamingMessageId: null,
      });
    }
  },

  // Stop streaming
  stopStreaming: () => {
    const { abortController } = get();
    if (abortController) {
      abortController.abort();
      set({ 
        abortController: null,
        status: 'ready',
        streamingMessageId: null,
      });
    }
  },

  // Load session history
  loadSessionHistory: async (sessionId: string) => {
    console.log('[Session] Loading session:', sessionId);
    const session = await loadSession('', sessionId);

    if (!session) {
      toast.error('Session not found');
      return;
    }

    const sessionStore = useSessionStore.getState();
    const modelStore = useModelStore.getState();

    // Update session ID
    sessionStore.setActiveSession(sessionId);

    // Convert session messages to UI format
    const uiMessages: MessageType[] = [];
    for (const msg of session.messages) {
      // Build thinking steps from the message
      // Prefer thinking_steps if available (new format), fallback to reasoning+tools (old format)
      const thinkingSteps: ThinkingStep[] = [];
      
      if (msg.thinking_steps && msg.thinking_steps.length > 0) {
        // New format: use thinking_steps directly from the message
        msg.thinking_steps.forEach((step: any) => {
          if (step.step_type === 'reasoning') {
            thinkingSteps.push({
              type: 'reasoning',
              content: step.content || '',
            });
          } else if (step.step_type === 'tool') {
            thinkingSteps.push({
              type: 'tool',
              name: step.tool_name || '',
              description: '',
              status: step.tool_error ? 'error' : 'completed',
              parameters: typeof step.tool_arguments === 'object' ? step.tool_arguments : {},
              result: step.tool_result,
              error: step.tool_error,
            });
          }
        });
      } else {
        // Old format: build from reasoning and tools (backward compatibility)
        if (msg.reasoning && msg.tools) {
          // Add reasoning step
          thinkingSteps.push({
            type: 'reasoning',
            content: msg.reasoning,
          });
          
          // Add tool steps after reasoning
          msg.tools.forEach((t) => {
            thinkingSteps.push({
              type: 'tool',
              name: t.name,
              description: '',
              status: t.error ? 'error' : 'completed',
              parameters: typeof t.arguments === 'object' ? t.arguments : {},
              result: t.result,
              error: t.error,
            });
          });
        } else if (msg.reasoning) {
          // Only reasoning, no tools
          thinkingSteps.push({
            type: 'reasoning',
            content: msg.reasoning,
          });
        } else if (msg.tools) {
          // Only tools, no reasoning
          msg.tools.forEach((t) => {
            thinkingSteps.push({
              type: 'tool',
              name: t.name,
              description: '',
              status: t.error ? 'error' : 'completed',
              parameters: typeof t.arguments === 'object' ? t.arguments : {},
              result: t.result,
              error: t.error,
            });
          });
        }
      }

      uiMessages.push({
        from: msg.role as 'user' | 'assistant',
        key: `${msg.role}-${msg.timestamp}`,
        sources:
          msg.sources.length > 0
            ? msg.sources.map((s) => ({
                href: '#',
                title: s.title,
                content: s.content,
              }))
            : undefined,
        versions: [
          {
            id: `${msg.role}-${msg.timestamp}-v1`,
            content: msg.content,
          },
        ],
        reasoning: msg.reasoning
          ? {
              content: msg.reasoning,
              duration: undefined,
            }
          : undefined,
        thinkingSteps: thinkingSteps.length > 0 ? thinkingSteps : undefined,
        tools: msg.tools?.map((t) => ({
          name: t.name,
          description: '',
          status: t.error ? 'error' : 'completed',
          parameters: typeof t.arguments === 'object' ? t.arguments : {},
          result: t.result,
          error: t.error,
        })),
      });
    }

    console.log(`[Session] Loaded session with ${uiMessages.length} messages`);
    set({ 
      messages: uiMessages,
      status: 'ready',
    });

    // Load token usage from session
    modelStore.updateTokenUsage(session.token_usage);
    modelStore.setSessionModelId(session.model_id);

    // Update model selector if session has a model_id
    if (session.model_id && modelStore.models.length > 0) {
      let matchedModel = modelStore.models.find((m) => m.id === session.model_id);

      // Fuzzy matching if no exact match
      if (!matchedModel) {
        const sessionModelLower = session.model_id.toLowerCase();
        matchedModel = modelStore.models.find(
          (m) => m.id.toLowerCase().includes(sessionModelLower) || sessionModelLower.includes(m.id.toLowerCase())
        );
      }

      if (matchedModel) {
        modelStore.setSelectedModel(matchedModel.id);
      }
    }
  },

  // Clear all messages
  clearMessages: () => {
    set({ 
      messages: [],
      status: 'ready',
      streamingMessageId: null,
      streamingContentRef: '',
      streamingReasoningRef: '',
      isReasoningStreaming: false,
    });
  },

  // Toggle web search
  toggleWebSearch: () => {
    set((state) => ({ useWebSearch: !state.useWebSearch }));
  },

  // Add pending tool approval
  addPendingApproval: (approval: ToolApproval) => {
    set((state) => {
      const newPendingApprovals = new Map(state.pendingApprovals);
      newPendingApprovals.set(approval.approval_id, approval);

      // Add approval to the associated message
      const messages = state.messages.map((msg) => {
        if (msg.versions.some((v) => v.id === approval.message_id)) {
          return {
            ...msg,
            toolApprovals: [...(msg.toolApprovals || []), approval],
          };
        }
        return msg;
      });

      return {
        pendingApprovals: newPendingApprovals,
        messages,
      };
    });
  },

  // Respond to tool approval
  respondToApproval: async (
    approval_id: string,
    approved: boolean,
    save_decision: boolean,
    scope?: string
  ) => {
    const state = get();
    const approval = state.pendingApprovals.get(approval_id);

    if (!approval) {
      console.error('Approval not found:', approval_id);
      return;
    }

    // Send approval to backend
    const success = await sendToolApproval('', approval_id, approved, save_decision, scope || '');

    if (success) {
      // Record the decision
      const decision: ToolApprovalDecision = {
        approval_id,
        approved,
        timestamp: Date.now(),
      };

      set((state) => {
        const newDecisions = new Map(state.toolApprovalDecisions);
        newDecisions.set(approval_id, decision);

        return {
          toolApprovalDecisions: newDecisions,
        };
      });

      // Show toast
      if (approved) {
        toast.success('Tool execution approved', {
          description: `${approval.tool_name} can now execute`,
        });
      } else {
        toast.info('Tool execution rejected', {
          description: `${approval.tool_name} was not executed`,
        });
      }
    } else {
      toast.error('Failed to send approval', {
        description: 'Could not communicate with the server',
      });
    }
  },

  // Clear approval
  clearApproval: (approval_id: string) => {
    set((state) => {
      const newPendingApprovals = new Map(state.pendingApprovals);
      newPendingApprovals.delete(approval_id);

      return {
        pendingApprovals: newPendingApprovals,
      };
    });
  },
}));
