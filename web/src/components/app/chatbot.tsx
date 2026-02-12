import type { PromptInputMessage } from '@/components/ai-elements/prompt-input';
import type { FileUIPart, ToolUIPart } from 'ai';

import { loadSession, streamChat } from '@/lib/chat-api';
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
  ModelSelectorLogo,
  ModelSelectorLogoGroup,
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
import { Source, Sources, SourcesContent, SourcesTrigger } from '@/components/ai-elements/sources';
import { SpeechInput } from '@/components/ai-elements/speech-input';
import { Suggestion, Suggestions } from '@/components/ai-elements/suggestion';
import { CheckIcon, GlobeIcon } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';

interface MessageType {
  key: string;
  from: 'user' | 'assistant';
  sources?: { href: string; title: string }[];
  versions: {
    id: string;
    content: string;
  }[];
  reasoning?: {
    content: string;
    duration: number;
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

const models = [
  {
    chef: 'OpenAI',
    chefSlug: 'openai',
    id: 'gpt-4o',
    name: 'GPT-4o',
    providers: ['openai', 'azure'],
  },
  {
    chef: 'OpenAI',
    chefSlug: 'openai',
    id: 'gpt-4o-mini',
    name: 'GPT-4o Mini',
    providers: ['openai', 'azure'],
  },
  {
    chef: 'Anthropic',
    chefSlug: 'anthropic',
    id: 'claude-opus-4-20250514',
    name: 'Claude 4 Opus',
    providers: ['anthropic', 'azure', 'google', 'amazon-bedrock'],
  },
  {
    chef: 'Anthropic',
    chefSlug: 'anthropic',
    id: 'claude-sonnet-4-20250514',
    name: 'Claude 4 Sonnet',
    providers: ['anthropic', 'azure', 'google', 'amazon-bedrock'],
  },
  {
    chef: 'Google',
    chefSlug: 'google',
    id: 'gemini-2.0-flash-exp',
    name: 'Gemini 2.0 Flash',
    providers: ['google'],
  },
];

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

const chefs = ['OpenAI', 'Anthropic', 'Google'];

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
  m: (typeof models)[0];
  isSelected: boolean;
  onSelect: (id: string) => void;
}) => {
  const handleSelect = useCallback(() => {
    onSelect(m.id);
  }, [onSelect, m.id]);

  return (
    <ModelSelectorItem onSelect={handleSelect} value={m.id}>
      <ModelSelectorLogo provider={m.chefSlug} />
      <ModelSelectorName>{m.name}</ModelSelectorName>
      <ModelSelectorLogoGroup>
        {m.providers.map((provider) => (
          <ModelSelectorLogo key={provider} provider={provider} />
        ))}
      </ModelSelectorLogoGroup>
      {isSelected ? <CheckIcon className="ml-auto size-4" /> : <div className="ml-auto size-4" />}
    </ModelSelectorItem>
  );
};

const Example = () => {
  const [model, setModel] = useState<string>(models[0].id);
  const [modelSelectorOpen, setModelSelectorOpen] = useState(false);
  const [text, setText] = useState<string>('');
  const [useWebSearch, setUseWebSearch] = useState<boolean>(false);
  const [status, setStatus] = useState<'submitted' | 'streaming' | 'ready' | 'error'>('ready');
  const [messages, setMessages] = useState<MessageType[]>(initialMessages);
  const [sessionId, setSessionId] = useState<string | null>(() => {
    // Load session ID from localStorage on mount
    return localStorage.getItem('squid_session_id');
  });
  const [, setStreamingMessageId] = useState<string | null>(null);
  const streamingContentRef = useRef<string>('');
  const abortControllerRef = useRef<AbortController | null>(null);
  const sessionLoadedRef = useRef<boolean>(false);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [sessionListRefreshTrigger, setSessionListRefreshTrigger] = useState(0);

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

  const selectedModelData = useMemo(() => models.find((m) => m.id === model), [model]);

  // Map local/unknown models to similar OpenAI models for cost estimation
  const getModelIdForPricing = useMemo(() => {
    const currentModelId = sessionModelId || model;

    // If tokenlens knows this model, use it as-is
    // Otherwise, map to a similar OpenAI model for cost comparison
    const modelMappings: Record<string, string> = {
      // Qwen models -> similar GPT-4 class
      'qwen2.5-coder': 'gpt-4o',
      'qwen-coder': 'gpt-4o',
      qwen: 'gpt-4o',

      // DeepSeek models -> GPT-4o
      deepseek: 'gpt-4o',
      'deepseek-coder': 'gpt-4o',
      devstral: 'gpt-4o',

      // Llama models -> GPT-4o
      llama: 'gpt-4o',
      'llama-3': 'gpt-4o',
      codellama: 'gpt-4o',

      // Mistral models -> GPT-4o
      mistral: 'gpt-4o',
      mixtral: 'gpt-4o',

      // Phi models -> GPT-4o-mini (smaller)
      phi: 'gpt-4o-mini',

      // Gemma models -> GPT-4o-mini
      gemma: 'gpt-4o-mini',

      // Yi models -> GPT-4o
      'yi-coder': 'gpt-4o',
      yi: 'gpt-4o',

      // StarCoder models -> GPT-4o
      starcoder: 'gpt-4o',
      wizardcoder: 'gpt-4o',
    };

    // Check if current model matches any pattern
    for (const [pattern, fallback] of Object.entries(modelMappings)) {
      if (currentModelId.toLowerCase().includes(pattern)) {
        return fallback;
      }
    }

    // Default to current model ID if no mapping found
    return currentModelId;
  }, [sessionModelId, model]);

  // Load session history on mount if sessionId exists
  const loadSessionHistory = useCallback(async (targetSessionId: string) => {
    console.log('[Session] Loading session:', targetSessionId);
    const session = await loadSession('', targetSessionId);
    if (!session) {
      console.log('[Session] Session not found, starting fresh');
      localStorage.removeItem('squid_session_id');
      setSessionId(null);
      return;
    }

    console.log(`[Session] Loaded session with ${session.messages.length} messages:`, session.messages);

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
              }))
            : undefined,
        versions: [
          {
            id: `${msg.role}-${msg.timestamp}-v1`,
            content: msg.content,
          },
        ],
      });
    }

    console.log(`[Session] Loaded ${uiMessages.length} messages`);
    setMessages(uiMessages);

    // Load token usage from session
    setTokenUsage(session.token_usage);
    setSessionModelId(session.model_id);
    console.log('[Session] Token usage:', session.token_usage);
  }, []);

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
                      })),
                    };
                  }
                  return msg;
                })
              );
            },
            onContent: (text) => {
              console.log('[Stream] Received content chunk:', text.substring(0, 50));
              // Accumulate content in ref for better performance
              streamingContentRef.current += text;
              const currentContent = streamingContentRef.current;
              console.log('[Stream] Looking for messageId:', messageId);

              setMessages((prev) => {
                console.log('[Stream] Current messages:', prev.length);
                const updated = prev.map((msg) => {
                  const hasVersion = msg.versions.some((v) => v.id === messageId);
                  console.log('[Stream] Message key:', msg.key, 'has matching version:', hasVersion);
                  if (hasVersion) {
                    return {
                      ...msg,
                      versions: msg.versions.map((v) => (v.id === messageId ? { ...v, content: currentContent } : v)),
                    };
                  }
                  return msg;
                });
                console.log('[Stream] Updated messages:', updated.length);
                return updated;
              });
            },
            onUsage: (usage) => {
              console.log('[Token Usage] Received:', usage);
              // Update token usage
              setTokenUsage((prev) => {
                const updated = {
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
                };
                console.log('[Token Usage] Updated from:', prev, 'to:', updated);
                return updated;
              });
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
              streamingContentRef.current = ''; // Clear ref after streaming
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
    [updateMessageContent, sessionId, loadSessionHistory]
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
                }))
              : undefined,
          versions: [
            {
              id: `${msg.role}-${msg.timestamp}`,
              content: msg.content,
            },
          ],
        });
      }

      console.log(`[Session] Switched to session with ${uiMessages.length} messages`);
      setMessages(uiMessages);
      setText('');
      setStatus('ready');

      // Load token usage from session
      setTokenUsage(session.token_usage);
      setSessionModelId(session.model_id);
      console.log('[Session] Token usage:', session.token_usage);

      toast.success('Session loaded');
    },
    [sessionId]
  );

  const isSubmitDisabled = useMemo(() => !(text.trim() || status), [text, status]);

  return (
    <div className="relative flex size-full overflow-hidden">
      {sidebarOpen && (
        <div className="w-64 shrink-0">
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
                        {message.sources?.length && (
                          <Sources>
                            <SourcesTrigger count={message.sources.length} />
                            <SourcesContent>
                              {message.sources.map((source) => (
                                <Source href={source.href} key={source.href} title={source.title} />
                              ))}
                            </SourcesContent>
                          </Sources>
                        )}
                        {message.reasoning && (
                          <Reasoning duration={message.reasoning.duration}>
                            <ReasoningTrigger />
                            <ReasoningContent>{message.reasoning.content}</ReasoningContent>
                          </Reasoning>
                        )}
                        <MessageContent>
                          {message.from === 'assistant' && !version.content && status === 'streaming' ? (
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
            <PromptInput globalDrop multiple onSubmit={handleSubmit}>
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
                        {selectedModelData?.chefSlug && <ModelSelectorLogo provider={selectedModelData.chefSlug} />}
                        {selectedModelData?.name && <ModelSelectorName>{selectedModelData.name}</ModelSelectorName>}
                      </PromptInputButton>
                    </ModelSelectorTrigger>
                    <ModelSelectorContent>
                      <ModelSelectorInput placeholder="Search models..." />
                      <ModelSelectorList>
                        <ModelSelectorEmpty>No models found.</ModelSelectorEmpty>
                        {chefs.map((chef) => (
                          <ModelSelectorGroup heading={chef} key={chef}>
                            {models
                              .filter((m) => m.chef === chef)
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
    </div>
  );
};

export default Example;
