import type { PromptInputMessage } from '@/components/ai-elements/prompt-input';
import type { FileUIPart, ToolUIPart } from 'ai';

import { fetchModels, loadSession, streamChat, type ModelInfo } from '@/lib/chat-api';
import { SessionList } from '@/components/app/session-list';
import { Shimmer } from '@/components/ai-elements/shimmer';
import { Attachment, AttachmentPreview, AttachmentRemove, Attachments } from '@/components/ai-elements/attachments';
import {
  Context,
  ContextCacheUsage,
  ContextContent,
  ContextContentBody,
  ContextContentFooter,
  ContextContentHeader,
  ContextInputUsage,
  ContextOutputUsage,
  ContextReasoningUsage,
  ContextTrigger,
} from '@/components/ai-elements/context';
import { Conversation, ConversationContent, ConversationScrollButton } from '@/components/ai-elements/conversation';
import {
  Message,
  MessageBranch,
  MessageBranchContent,
  MessageBranchNext,
  MessageBranchPage,
  MessageBranchPrevious,
  MessageBranchSelector,
  MessageContent,
  MessageResponse,
} from '@/components/ai-elements/message';
import {
  ModelSelector,
  ModelSelectorContent,
  ModelSelectorEmpty,
  ModelSelectorGroup,
  ModelSelectorInput,
  ModelSelectorItem,
  ModelSelectorList,
  ModelSelectorName,
  ModelSelectorTrigger,
} from '@/components/ai-elements/model-selector';
import {
  PromptInput,
  PromptInputActionAddAttachments,
  PromptInputActionMenu,
  PromptInputActionMenuContent,
  PromptInputActionMenuTrigger,
  PromptInputBody,
  PromptInputButton,
  PromptInputFooter,
  PromptInputHeader,
  PromptInputSubmit,
  PromptInputTextarea,
  PromptInputTools,
  usePromptInputAttachments,
} from '@/components/ai-elements/prompt-input';
import { Reasoning, ReasoningContent, ReasoningTrigger } from '@/components/ai-elements/reasoning';
import { Sources, SourcesContent, SourcesTrigger } from '@/components/ai-elements/sources';
import {
  CodeBlock,
  CodeBlockActions,
  CodeBlockCopyButton,
  CodeBlockFilename,
  CodeBlockHeader,
  CodeBlockTitle,
} from '@/components/ai-elements/code-block';
import { SpeechInput } from '@/components/ai-elements/speech-input';
import { Suggestion, Suggestions } from '@/components/ai-elements/suggestion';
import { CheckIcon, GlobeIcon, FileIcon } from 'lucide-react';
import type { BundledLanguage } from 'shiki';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';

interface MessageType {
  key: string;
  from: 'user' | 'assistant';
  sources?: { href: string; title: string; content: string }[];
  versions: {
    id: string;
    content: string;
  }[];
  reasoning?: {
    content: string;
    duration?: number;
  };
  tools?: {
    name: string;
    description: string;
    status: ToolUIPart['state'];
    parameters: Record<string, unknown>;
    result: string | undefined;
    error: string | undefined;
  }[];
}

const initialMessages: MessageType[] = [];

const suggestions = [
  'What are the latest trends in AI?',
  'How does machine learning work?',
  'Explain quantum computing',
  'Best practices for React development',
  'Tell me about TypeScript benefits',
  'How to optimize database queries?',
  'What is the difference between SQL and NoSQL?',
  'Explain cloud computing basics',
];

const AttachmentItem = ({
  attachment,
  onRemove,
}: {
  attachment: FileUIPart & { id: string };
  onRemove: (id: string) => void;
}) => {
  const handleRemove = useCallback(() => {
    onRemove(attachment.id);
  }, [onRemove, attachment.id]);

  return (
    <Attachment data={attachment} onRemove={handleRemove}>
      <AttachmentPreview />
      <AttachmentRemove />
    </Attachment>
  );
};

const PromptInputAttachmentsDisplay = () => {
  const attachments = usePromptInputAttachments();

  const handleRemove = useCallback(
    (id: string) => {
      attachments.remove(id);
    },
    [attachments]
  );

  if (attachments.files.length === 0) {
    return null;
  }

  return (
    <Attachments variant="inline">
      {attachments.files.map((attachment) => (
        <AttachmentItem attachment={attachment} key={attachment.id} onRemove={handleRemove} />
      ))}
    </Attachments>
  );
};

const SuggestionItem = ({ suggestion, onClick }: { suggestion: string; onClick: (suggestion: string) => void }) => {
  const handleClick = useCallback(() => {
    onClick(suggestion);
  }, [onClick, suggestion]);

  return <Suggestion onClick={handleClick} suggestion={suggestion} />;
};

const ModelItem = ({
  m,
  isSelected,
  onSelect,
}: {
  m: ModelInfo;
  isSelected: boolean;
  onSelect: (id: string) => void;
}) => {
  const handleSelect = useCallback(() => {
    onSelect(m.id);
  }, [onSelect, m.id]);

  return (
    <ModelSelectorItem onSelect={handleSelect} value={m.id}>
      <ModelSelectorName>{m.name}</ModelSelectorName>
      {isSelected ? <CheckIcon className="ml-auto size-4" /> : <div className="ml-auto size-4" />}
    </ModelSelectorItem>
  );
};

const Example = () => {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [modelGroups, setModelGroups] = useState<string[]>([]);
  const [model, setModel] = useState<string>('');
  const [modelSelectorOpen, setModelSelectorOpen] = useState(false);
  const [text, setText] = useState<string>('');
  const [useWebSearch, setUseWebSearch] = useState<boolean>(false);
  const [status, setStatus] = useState<'submitted' | 'streaming' | 'ready' | 'error'>('ready');
  const [messages, setMessages] = useState<MessageType[]>(initialMessages);
  const [sessionId, setSessionId] = useState<string | null>(() => {
    // Load session ID from localStorage on mount
    return localStorage.getItem('squid_session_id');
  });
  const [streamingMessageId, setStreamingMessageId] = useState<string | null>(null);
  const streamingContentRef = useRef<string>('');
  const streamingReasoningRef = useRef<string>('');
  const [isReasoningStreaming, setIsReasoningStreaming] = useState<boolean>(false);
  const reasoningStartTimeRef = useRef<number | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);
  const sessionLoadedRef = useRef<boolean>(false);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [sessionListRefreshTrigger, setSessionListRefreshTrigger] = useState(0);
  const [sourceContentOpen, setSourceContentOpen] = useState(false);
  const [sourceContentData, setSourceContentData] = useState<{ title: string; content: string } | null>(null);

  // Token usage tracking
  const [tokenUsage, setTokenUsage] = useState({
    total_tokens: 0,
    input_tokens: 0,
    output_tokens: 0,
    reasoning_tokens: 0,
    cache_tokens: 0,
    context_window: 0,
    context_utilization: 0,
  });
  const [sessionModelId, setSessionModelId] = useState<string | null>(null);

  const selectedModelData = useMemo(() => models.find((m) => m.id === model), [model, models]);

  // Get pricing model from backend metadata or fallback to model ID
  const getModelIdForPricing = useMemo(() => {
    const currentModelId = sessionModelId || model;

    // If model ID is empty, return default
    if (!currentModelId) {
      return 'gpt-4o';
    }

    // Find the model in the models list
    const modelData = models.find((m) => m.id === currentModelId);

    // Use pricing_model from backend if available, otherwise use the model ID itself
    return modelData?.pricing_model || currentModelId;
  }, [sessionModelId, model, models]);

  // Load session history on mount if sessionId exists
  const loadSessionHistory = useCallback(
    async (targetSessionId: string) => {
      const session = await loadSession('', targetSessionId);
      if (!session) {
        localStorage.removeItem('squid_session_id');
        setSessionId(null);
        return;
      }

      // Convert session messages to UI format
      const uiMessages: MessageType[] = [];
      for (const msg of session.messages) {
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
        });
      }

      console.log(`[Session] Loaded ${uiMessages.length} messages`);
      setMessages(uiMessages);

      // Load token usage from session
      setTokenUsage(session.token_usage);
      setSessionModelId(session.model_id);

      // Update model selector if session has a model_id
      if (session.model_id && models.length > 0) {
        // Try exact match first
        let matchedModel = models.find((m) => m.id === session.model_id);

        // If no exact match, try fuzzy matching
        if (!matchedModel) {
          const sessionModelLower = session.model_id.toLowerCase();
          matchedModel = models.find(
            (m) => m.id.toLowerCase().includes(sessionModelLower) || sessionModelLower.includes(m.id.toLowerCase())
          );
        }

        if (matchedModel) {
          setModel(matchedModel.id);
        }
      }
    },
    [models]
  );

  // Fetch available models on mount
  useEffect(() => {
    const loadModels = async () => {
      const { models: fetchedModels } = await fetchModels('');

      if (fetchedModels.length > 0) {
        setModels(fetchedModels);

        // Extract unique providers and sort them
        const providers = Array.from(new Set(fetchedModels.map((m) => m.provider))).sort();
        setModelGroups(providers);

        // Set default model - prefer Qwen Coder 2.5
        const defaultModel =
          fetchedModels.find((m) => m.id.includes('qwen2.5-coder')) ||
          fetchedModels.find((m) => m.id.includes('qwen') && m.id.includes('coder')) ||
          fetchedModels.find((m) => m.provider === 'Qwen') ||
          fetchedModels[0];

        if (defaultModel) {
          setModel(defaultModel.id);
          console.log(`🤖 Default model: ${defaultModel.name} (${defaultModel.id})`);
        }
      }
    };

    loadModels();
  }, []);

  // Update context window when model changes
  useEffect(() => {
    if (model && models.length > 0) {
      const selectedModel = models.find((m) => m.id === model);
      if (selectedModel) {
        setTokenUsage((prev) => ({
          ...prev,
          context_window: selectedModel.max_context_length,
        }));
      }
    }
  }, [model, models]);

  useEffect(() => {
    if (!sessionId || sessionLoadedRef.current) return;
    sessionLoadedRef.current = true;

    loadSessionHistory(sessionId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run on mount

  const updateMessageContent = useCallback((messageId: string, newContent: string) => {
    setMessages((prev) =>
      prev.map((msg) => {
        if (msg.versions.some((v) => v.id === messageId)) {
          return {
            ...msg,
            versions: msg.versions.map((v) => (v.id === messageId ? { ...v, content: newContent } : v)),
          };
        }
        return msg;
      })
    );
  }, []);

  const streamResponse = useCallback(
    async (messageId: string, userMessage: string, files?: FileUIPart[]) => {
      // Create new abort controller for this request
      abortControllerRef.current = new AbortController();

      setStatus('streaming');
      setStreamingMessageId(messageId);
      streamingContentRef.current = ''; // Reset streaming content
      streamingReasoningRef.current = ''; // Reset streaming reasoning
      setIsReasoningStreaming(false);
      reasoningStartTimeRef.current = null; // Reset reasoning timer

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

        // Use relative path since web UI is served from the same server
        await streamChat(
          '',
          {
            message: userMessage,
            session_id: sessionId || undefined,
            files: fileAttachments,
            model: model || undefined,
          },
          {
            signal: abortControllerRef.current?.signal,
            onSession: (newSessionId) => {
              setSessionId(newSessionId);
              // Persist session ID to localStorage
              localStorage.setItem('squid_session_id', newSessionId);
              // Trigger session list refresh
              setSessionListRefreshTrigger((prev) => prev + 1);
            },
            onSources: (sources) => {
              // Update the assistant message with sources
              setMessages((prev) =>
                prev.map((msg) => {
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
                })
              );
            },
            onContent: (text) => {
              // Accumulate content in ref for better performance
              streamingContentRef.current += text;
              const fullContent = streamingContentRef.current;

              // Parse out <think> tags
              let displayContent = fullContent;
              let reasoningContent = '';

              const thinkStart = fullContent.indexOf('<think>');
              const thinkEnd = fullContent.indexOf('</think>');

              if (thinkStart !== -1 && thinkEnd !== -1 && thinkEnd > thinkStart) {
                // Extract reasoning between tags
                reasoningContent = fullContent.substring(thinkStart + 7, thinkEnd);
                // Remove the entire <think>...</think> section from display content
                displayContent = fullContent.substring(0, thinkStart) + fullContent.substring(thinkEnd + 8);
              } else if (thinkStart !== -1) {
                // Opening tag found but no closing tag yet
                reasoningContent = fullContent.substring(thinkStart + 7);
                displayContent = fullContent.substring(0, thinkStart);
              }

              // Update reasoning if found and start timer
              if (reasoningContent) {
                if (!isReasoningStreaming) {
                  setIsReasoningStreaming(true);
                  reasoningStartTimeRef.current = Date.now();
                }
              }

              setMessages((prev) => {
                const updated = prev.map((msg) => {
                  const hasVersion = msg.versions.some((v) => v.id === messageId);
                  if (hasVersion) {
                    return {
                      ...msg,
                      versions: msg.versions.map((v) => (v.id === messageId ? { ...v, content: displayContent } : v)),
                      reasoning: reasoningContent
                        ? {
                            content: reasoningContent,
                            duration: 0,
                          }
                        : msg.reasoning,
                    };
                  }
                  return msg;
                });
                return updated;
              });
            },
            onUsage: (usage) => {
              // Update token usage
              setTokenUsage((prev) => ({
                total_tokens:
                  prev.total_tokens +
                  usage.input_tokens +
                  usage.output_tokens +
                  usage.reasoning_tokens +
                  usage.cache_tokens,
                input_tokens: prev.input_tokens + usage.input_tokens,
                output_tokens: prev.output_tokens + usage.output_tokens,
                reasoning_tokens: prev.reasoning_tokens + usage.reasoning_tokens,
                cache_tokens: prev.cache_tokens + usage.cache_tokens,
                context_window: prev.context_window,
                context_utilization: prev.context_utilization,
              }));
            },
            onError: (error) => {
              console.error('Stream error:', error);
              updateMessageContent(messageId, `Error: ${error}`);
              toast.error('Failed to get response', {
                description: error,
              });
              setStatus('ready');
              setStreamingMessageId(null);
            },
            onDone: async () => {
              // Calculate reasoning duration if we were tracking it
              let reasoningDuration = 0;
              if (reasoningStartTimeRef.current !== null) {
                reasoningDuration = Math.ceil((Date.now() - reasoningStartTimeRef.current) / 1000);
                reasoningStartTimeRef.current = null;
              }

              // Update message with final reasoning duration
              if (reasoningDuration > 0) {
                setMessages((prev) =>
                  prev.map((msg) => {
                    const hasVersion = msg.versions.some((v) => v.id === messageId);
                    if (hasVersion && msg.reasoning) {
                      return {
                        ...msg,
                        reasoning: {
                          ...msg.reasoning,
                          duration: reasoningDuration,
                        },
                      };
                    }
                    return msg;
                  })
                );
              }

              streamingContentRef.current = ''; // Clear ref after streaming
              streamingReasoningRef.current = ''; // Clear reasoning ref
              setIsReasoningStreaming(false);
              abortControllerRef.current = null;
              setStatus('ready');
              setStreamingMessageId(null);

              // Reload session to get updated context_window and token usage from backend
              if (sessionId) {
                try {
                  await loadSessionHistory(sessionId);
                } catch (error) {
                  console.error('Failed to reload session after streaming:', error);
                }
              }
            },
          }
        );
      } catch (error) {
        // Don't show error if it was aborted by user
        if (error instanceof Error && error.name === 'AbortError') {
          updateMessageContent(messageId, streamingContentRef.current || 'Response stopped by user.');
        } else {
          console.error('Chat error:', error);
          updateMessageContent(messageId, `Error: ${error instanceof Error ? error.message : String(error)}`);
          toast.error('Failed to send message', {
            description: error instanceof Error ? error.message : String(error),
          });
        }
        abortControllerRef.current = null;
        setStatus('ready');
        setStreamingMessageId(null);
      }
    },
    [updateMessageContent, sessionId, loadSessionHistory, model]
  );

  const addUserMessage = useCallback(
    (content: string, files?: FileUIPart[]) => {
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

      setMessages((prev) => [...prev, userMessage]);

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

        setMessages((prev) => [...prev, assistantMessage]);
        streamResponse(assistantMessageId, content, files);
      }, 500);
    },
    [streamResponse]
  );

  const handleSubmit = useCallback(
    (message: PromptInputMessage) => {
      const hasText = Boolean(message.text);
      const hasAttachments = Boolean(message.files?.length);

      if (!(hasText || hasAttachments)) {
        return;
      }

      setStatus('submitted');

      if (message.files?.length) {
        toast.success('Files attached', {
          description: `${message.files.length} file(s) attached to message`,
        });
      }

      addUserMessage(message.text || 'Sent with attachments', message.files);
      setText('');
    },
    [addUserMessage]
  );

  const handleSuggestionClick = useCallback(
    (suggestion: string) => {
      setStatus('submitted');
      addUserMessage(suggestion);
    },
    [addUserMessage]
  );

  const handleTranscriptionChange = useCallback((transcript: string) => {
    setText((prev) => (prev ? `${prev} ${transcript}` : transcript));
  }, []);

  const handleTextChange = useCallback((event: React.ChangeEvent<HTMLTextAreaElement>) => {
    setText(event.target.value);
  }, []);

  const toggleWebSearch = useCallback(() => {
    setUseWebSearch((prev) => !prev);
  }, []);

  const handleModelSelect = useCallback((modelId: string) => {
    setModel(modelId);
    setModelSelectorOpen(false);
  }, []);

  const handleStop = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
      setStatus('ready');
      setStreamingMessageId(null);
    }
  }, []);

  const handleFileUploadError = useCallback((error: { code: string; message: string }) => {
    if (error.code === 'max_file_size') {
      toast.error('File too large', {
        description: 'Files must be smaller than 10MB. Large files will also be rejected by the server.',
      });
    } else {
      toast.error('Upload failed', {
        description: error.message,
      });
    }
  }, []);

  const handleNewChat = useCallback(() => {
    // Clear session ID from state and localStorage
    setSessionId(null);
    localStorage.removeItem('squid_session_id');

    // Reset loaded flag
    sessionLoadedRef.current = false;

    // Reset messages to empty (clear the chat)
    setMessages([]);

    // Clear input
    setText('');

    // Reset status
    setStatus('ready');

    // Reset token usage
    setTokenUsage({
      total_tokens: 0,
      input_tokens: 0,
      output_tokens: 0,
      reasoning_tokens: 0,
      cache_tokens: 0,
      context_window: 0,
      context_utilization: 0,
    });
    setSessionModelId(null);

    toast.success('New chat started');
  }, []);

  const handleSessionSelect = useCallback(
    async (selectedSessionId: string) => {
      // Don't reload if already on this session
      if (selectedSessionId === sessionId) return;

      console.log('[Session] Selecting session:', selectedSessionId);
      const session = await loadSession('', selectedSessionId);

      if (!session) {
        toast.error('Session not found');
        return;
      }

      // Update session ID
      setSessionId(selectedSessionId);
      localStorage.setItem('squid_session_id', selectedSessionId);

      // Convert session messages to UI format
      const uiMessages: MessageType[] = [];
      for (const msg of session.messages) {
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
        });
      }

      console.log(`[Session] Switched to session with ${uiMessages.length} messages`);
      setMessages(uiMessages);
      setText('');
      setStatus('ready');

      // Load token usage from session
      setTokenUsage(session.token_usage);
      setSessionModelId(session.model_id);

      // Update model selector if session has a model_id
      if (session.model_id && models.length > 0) {
        // Try exact match first
        let matchedModel = models.find((m) => m.id === session.model_id);

        // If no exact match, try fuzzy matching
        if (!matchedModel) {
          const sessionModelLower = session.model_id.toLowerCase();
          matchedModel = models.find(
            (m) => m.id.toLowerCase().includes(sessionModelLower) || sessionModelLower.includes(m.id.toLowerCase())
          );
        }

        if (matchedModel) {
          setModel(matchedModel.id);
        }
      }
      toast.success('Session loaded');
    },
    [sessionId, models]
  );

  const isSubmitDisabled = useMemo(() => !(text.trim() || status), [text, status]);

  const handleViewSourceContent = useCallback((title: string, content: string) => {
    setSourceContentData({ title, content });
    setSourceContentOpen(true);
  }, []);

  // Detect language from filename
  const getLanguageFromFilename = useCallback((filename: string): BundledLanguage => {
    const ext = filename.split('.').pop()?.toLowerCase() || '';
    const languageMap: Record<string, BundledLanguage> = {
      ts: 'typescript',
      tsx: 'tsx',
      js: 'javascript',
      jsx: 'jsx',
      py: 'python',
      rs: 'rust',
      go: 'go',
      java: 'java',
      cpp: 'cpp',
      c: 'c',
      cs: 'csharp',
      rb: 'ruby',
      php: 'php',
      swift: 'swift',
      kt: 'kotlin',
      scala: 'scala',
      sh: 'bash',
      bash: 'bash',
      zsh: 'zsh',
      fish: 'fish',
      sql: 'sql',
      html: 'html',
      css: 'css',
      scss: 'scss',
      sass: 'sass',
      less: 'less',
      json: 'json',
      yaml: 'yaml',
      yml: 'yaml',
      toml: 'toml',
      xml: 'xml',
      md: 'markdown',
      markdown: 'markdown',
      vue: 'vue',
      svelte: 'svelte',
      graphql: 'graphql',
      dart: 'dart',
      lua: 'lua',
      r: 'r',
      matlab: 'matlab',
      latex: 'latex',
      tex: 'latex',
    };
    return languageMap[ext] || 'text';
  }, []);

  return (
    <div className="relative flex size-full overflow-hidden">
      {sidebarOpen && (
        <div className="flex h-full w-64 shrink-0">
          <SessionList
            currentSessionId={sessionId}
            onSessionSelect={handleSessionSelect}
            onNewChat={handleNewChat}
            refreshTrigger={sessionListRefreshTrigger}
            apiUrl=""
          />
        </div>
      )}
      <div className="relative flex size-full flex-col divide-y overflow-hidden">
        <div className="flex shrink-0 items-center justify-between border-b bg-white px-4 py-2 dark:bg-gray-950">
          <div className="flex items-center gap-2">
            <button
              className="rounded p-1 hover:bg-gray-100 dark:hover:bg-gray-800"
              onClick={() => setSidebarOpen(!sidebarOpen)}
              type="button"
            >
              <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
            <h2 className="text-sm font-semibold">Squid Chat</h2>
          </div>
          <div className="flex items-center gap-2">
            <Context
              maxTokens={tokenUsage.context_window || 128000}
              modelId={getModelIdForPricing}
              usage={{
                inputTokens: tokenUsage.input_tokens,
                outputTokens: tokenUsage.output_tokens,
                totalTokens: tokenUsage.total_tokens,
                inputTokenDetails: {
                  noCacheTokens: tokenUsage.input_tokens - tokenUsage.cache_tokens,
                  cacheReadTokens: tokenUsage.cache_tokens,
                  cacheWriteTokens: undefined,
                },
                outputTokenDetails: {
                  textTokens: tokenUsage.output_tokens - tokenUsage.reasoning_tokens,
                  reasoningTokens: tokenUsage.reasoning_tokens,
                },
              }}
              usedTokens={tokenUsage.total_tokens}
            >
              <ContextTrigger />
              <ContextContent>
                <ContextContentHeader />
                <ContextContentBody>
                  <div className="space-y-2">
                    <ContextInputUsage />
                    <ContextOutputUsage />
                    <ContextReasoningUsage />
                    <ContextCacheUsage />
                  </div>
                </ContextContentBody>
                <ContextContentFooter />
              </ContextContent>
            </Context>
            <button
              className="rounded border border-gray-300 bg-white px-3 py-1 text-sm font-medium hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700"
              onClick={handleNewChat}
              type="button"
            >
              New Chat
            </button>
          </div>
        </div>
        <Conversation>
          <ConversationContent>
            {messages.map(({ versions, ...message }) => (
              <MessageBranch defaultBranch={0} key={message.key}>
                <MessageBranchContent>
                  {versions.map((version) => (
                    <Message from={message.from} key={`${message.key}-${version.id}`}>
                      <div>
                        {message.from === 'assistant' && message.sources?.length && (
                          <Sources>
                            <SourcesTrigger count={message.sources.length} />
                            <SourcesContent>
                              {message.sources.map((source) => (
                                <button
                                  key={source.href}
                                  className="flex items-center gap-2 cursor-pointer hover:text-primary/80 transition-colors text-left"
                                  onClick={() => handleViewSourceContent(source.title, source.content)}
                                  type="button"
                                >
                                  <svg
                                    className="h-4 w-4"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                    xmlns="http://www.w3.org/2000/svg"
                                  >
                                    <path
                                      strokeLinecap="round"
                                      strokeLinejoin="round"
                                      strokeWidth={2}
                                      d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
                                    />
                                  </svg>
                                  <span className="block font-medium">{source.title}</span>
                                </button>
                              ))}
                            </SourcesContent>
                          </Sources>
                        )}
                        {message.reasoning && (
                          <Reasoning
                            duration={message.reasoning.duration}
                            isStreaming={
                              isReasoningStreaming && status === 'streaming' && streamingMessageId === version.id
                            }
                          >
                            <ReasoningTrigger />
                            <ReasoningContent>{message.reasoning.content}</ReasoningContent>
                          </Reasoning>
                        )}
                        <MessageContent>
                          {message.from === 'assistant' &&
                          !version.content &&
                          status === 'streaming' &&
                          !message.reasoning ? (
                            <Shimmer className="text-muted-foreground">Thinking...</Shimmer>
                          ) : (
                            <MessageResponse>{version.content}</MessageResponse>
                          )}
                        </MessageContent>
                      </div>
                    </Message>
                  ))}
                </MessageBranchContent>
                {versions.length > 1 && (
                  <MessageBranchSelector>
                    <MessageBranchPrevious />
                    <MessageBranchPage />
                    <MessageBranchNext />
                  </MessageBranchSelector>
                )}
              </MessageBranch>
            ))}
          </ConversationContent>
          <ConversationScrollButton />
        </Conversation>
        <div className="grid shrink-0 gap-4 pt-4">
          <Suggestions className="px-4">
            {suggestions.map((suggestion) => (
              <SuggestionItem key={suggestion} onClick={handleSuggestionClick} suggestion={suggestion} />
            ))}
          </Suggestions>
          <div className="w-full px-4 pb-4">
            <PromptInput
              globalDrop
              multiple
              maxFileSize={10 * 1024 * 1024}
              onError={handleFileUploadError}
              onSubmit={handleSubmit}
            >
              <PromptInputHeader>
                <PromptInputAttachmentsDisplay />
              </PromptInputHeader>
              <PromptInputBody>
                <PromptInputTextarea onChange={handleTextChange} value={text} />
              </PromptInputBody>
              <PromptInputFooter>
                <PromptInputTools>
                  <PromptInputActionMenu>
                    <PromptInputActionMenuTrigger />
                    <PromptInputActionMenuContent>
                      <PromptInputActionAddAttachments />
                    </PromptInputActionMenuContent>
                  </PromptInputActionMenu>
                  <SpeechInput
                    className="shrink-0"
                    onTranscriptionChange={handleTranscriptionChange}
                    size="icon-sm"
                    variant="ghost"
                  />
                  <PromptInputButton onClick={toggleWebSearch} variant={useWebSearch ? 'default' : 'ghost'}>
                    <GlobeIcon size={16} />
                    <span>Search</span>
                  </PromptInputButton>
                  <ModelSelector onOpenChange={setModelSelectorOpen} open={modelSelectorOpen}>
                    <ModelSelectorTrigger asChild>
                      <PromptInputButton>
                        {selectedModelData?.name && <ModelSelectorName>{selectedModelData.name}</ModelSelectorName>}
                        {!selectedModelData && <ModelSelectorName>Select model...</ModelSelectorName>}
                      </PromptInputButton>
                    </ModelSelectorTrigger>
                    <ModelSelectorContent>
                      <ModelSelectorInput placeholder="Search models..." />
                      <ModelSelectorList>
                        <ModelSelectorEmpty>No models found.</ModelSelectorEmpty>
                        {modelGroups.map((provider) => (
                          <ModelSelectorGroup heading={provider} key={provider}>
                            {models
                              .filter((m) => m.provider === provider)
                              .map((m) => (
                                <ModelItem isSelected={model === m.id} key={m.id} m={m} onSelect={handleModelSelect} />
                              ))}
                          </ModelSelectorGroup>
                        ))}
                      </ModelSelectorList>
                    </ModelSelectorContent>
                  </ModelSelector>
                </PromptInputTools>
                <PromptInputSubmit disabled={isSubmitDisabled} onStop={handleStop} status={status} />
              </PromptInputFooter>
            </PromptInput>
          </div>
        </div>
      </div>

      {/* Source Content Sidebar */}
      {sourceContentOpen && sourceContentData && (
        <div className="fixed right-0 top-0 h-full w-150 border-l bg-background shadow-lg z-50 flex flex-col">
          <div className="flex items-center justify-between border-b p-4">
            <div className="flex-1 min-w-0">
              <h3 className="font-semibold truncate">{sourceContentData.title}</h3>
              <p className="text-xs text-muted-foreground">
                {sourceContentData.content.length.toLocaleString()} characters
              </p>
            </div>
            <button
              onClick={() => setSourceContentOpen(false)}
              className="ml-2 rounded-md p-2 hover:bg-accent"
              type="button"
            >
              <svg
                className="h-4 w-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          <div className="flex-1 overflow-auto">
            <CodeBlock
              code={sourceContentData.content}
              language={getLanguageFromFilename(sourceContentData.title)}
              showLineNumbers={true}
            >
              <CodeBlockHeader>
                <CodeBlockTitle>
                  <FileIcon size={14} />
                  <CodeBlockFilename>{sourceContentData.title}</CodeBlockFilename>
                </CodeBlockTitle>
                <CodeBlockActions>
                  <CodeBlockCopyButton />
                </CodeBlockActions>
              </CodeBlockHeader>
            </CodeBlock>
          </div>
        </div>
      )}
    </div>
  );
};

export default Example;
