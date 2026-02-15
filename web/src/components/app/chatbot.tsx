import type { PromptInputMessage } from '@/components/ai-elements/prompt-input';
import type { FileUIPart } from 'ai';
import React from 'react';

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
import { Sources, SourcesContent, SourcesTrigger } from '@/components/ai-elements/sources';
import { Suggestions } from '@/components/ai-elements/suggestion';
import {
  ChainOfThought,
  ChainOfThoughtHeader,
  ChainOfThoughtContent,
  ChainOfThoughtStep,
} from '@/components/ai-elements/chain-of-thought';
import {
  Tool,
  ToolContent,
  ToolHeader,
  ToolInput,
  ToolOutput,
} from '@/components/ai-elements/tool';
import type { BundledLanguage } from 'shiki';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';
import { BrainIcon, WrenchIcon } from 'lucide-react';

// App components
import { SourceContentSidebar } from './source-content-sidebar';
import { ModelItem } from './model-item';
import { SuggestionItem } from './suggestion-item';
import { ToolApprovalComponent } from './tool-approval';

// Zustand stores
import { useSessionStore } from '@/stores/session-store';
import { useModelStore } from '@/stores/model-store';
import { useChatStore } from '@/stores/chat-store';

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

const Chatbot = () => {
  // Zustand stores
  const { activeSessionId } = useSessionStore();
  const {
    models,
    modelGroups,
    selectedModel,
    tokenUsage,
    modelSelectorOpen,
    setSelectedModel,
    setModelSelectorOpen,
    loadModels,
    getModelForPricing,
  } = useModelStore();
  const {
    messages,
    status,
    streamingMessageId,
    addUserMessage,
    setStatus,
    stopStreaming,
    loadSessionHistory,
    respondToApproval,
    toolApprovalDecisions,
  } = useChatStore();

  // Local UI state
  const [text, setText] = useState<string>('');
  const [sourceContentOpen, setSourceContentOpen] = useState(false);
  const [sourceContentData, setSourceContentData] = useState<{ title: string; content: string } | null>(null);

  const selectedModelData = useMemo(() => models.find((m) => m.id === selectedModel), [selectedModel, models]);

  // Fetch available models on mount
  useEffect(() => {
    void loadModels();
  }, [loadModels]);

  // Track previous activeSessionId to detect actual changes
  const prevActiveSessionIdRef = useRef<string | null>(null);

  // Load session when activeSessionId changes to a different value
  useEffect(() => {
    const prevId = prevActiveSessionIdRef.current;
    
    // Update ref for next comparison
    prevActiveSessionIdRef.current = activeSessionId;
    
    // Don't load if:
    // 1. No session ID
    // 2. Same as previous (no actual change)
    // 3. Currently streaming (new session being created)
    if (!activeSessionId || activeSessionId === prevId || status === 'streaming' || status === 'submitted') {
      return;
    }
    
    // Load session history
    void loadSessionHistory(activeSessionId);
  }, [activeSessionId, status, loadSessionHistory]);

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
    [addUserMessage, setStatus]
  );

  const handleSuggestionClick = useCallback(
    (suggestion: string) => {
      setStatus('submitted');
      addUserMessage(suggestion);
    },
    [addUserMessage, setStatus]
  );

  const handleTextChange = useCallback((event: React.ChangeEvent<HTMLTextAreaElement>) => {
    setText(event.target.value);
  }, []);

  const handleModelSelect = useCallback((modelId: string) => {
    setSelectedModel(modelId);
  }, [setSelectedModel]);

  const handleStop = useCallback(() => {
    stopStreaming();
  }, [stopStreaming]);

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

  const isSubmitDisabled = useMemo(() => !(text.trim() || status), [text, status]);

  const handleViewSourceContent = useCallback((title: string, content: string) => {
    setSourceContentData({ title, content });
    setSourceContentOpen(true);
  }, []);

  // Tool approval handlers
  const handleToolApprove = useCallback(
    (approval_id: string, save_decision: boolean, scope?: string) => {
      void respondToApproval(approval_id, true, save_decision, scope);
    },
    [respondToApproval]
  );

  const handleToolReject = useCallback(
    (approval_id: string, save_decision: boolean) => {
      void respondToApproval(approval_id, false, save_decision);
    },
    [respondToApproval]
  );

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
    <div className="relative flex flex-1 w-full flex-col overflow-hidden rounded-xl border bg-background min-h-0">
      <div className="flex shrink-0 items-center justify-end gap-2 border-b bg-white px-4 py-2 dark:bg-gray-950 rounded-t-xl">
        <Context
          maxTokens={tokenUsage.context_window || 128000}
          modelId={getModelForPricing()}
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
      </div>
      <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
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
                      {/* Display chain of thought if there are thinking steps with reasoning */}
                      {message.thinkingSteps && message.thinkingSteps.some(s => s.type === 'reasoning') ? (
                        <ChainOfThought defaultOpen={status === 'streaming' && streamingMessageId === version.id}>
                          <ChainOfThoughtHeader>Chain of Thought</ChainOfThoughtHeader>
                          <ChainOfThoughtContent>
                            {message.thinkingSteps.map((step, idx) => {
                              if (step.type === 'reasoning') {
                                return (
                                  <ChainOfThoughtStep
                                    key={`reasoning-${idx}`}
                                    icon={BrainIcon}
                                    label="Reasoning"
                                    status={
                                      status === 'streaming' &&
                                      streamingMessageId === version.id &&
                                      idx === message.thinkingSteps!.length - 1
                                        ? 'active'
                                        : 'complete'
                                    }
                                  >
                                    <div className="text-sm whitespace-pre-wrap">{step.content}</div>
                                  </ChainOfThoughtStep>
                                );
                              } else {
                                return (
                                  <ChainOfThoughtStep
                                    key={`tool-${idx}`}
                                    icon={WrenchIcon}
                                    label={`Tool: ${step.name}`}
                                    status={step.status === 'pending' ? 'active' : 'complete'}
                                  >
                                    <div className="space-y-2">
                                      {step.parameters && Object.keys(step.parameters).length > 0 && (
                                        <div className="text-xs">
                                          <div className="font-medium mb-1">Input:</div>
                                          <pre className="bg-muted p-2 rounded text-xs overflow-x-auto">
                                            {JSON.stringify(step.parameters, null, 2)}
                                          </pre>
                                        </div>
                                      )}
                                      {step.result && (
                                        <div className="text-xs">
                                          <div className="font-medium mb-1">Output:</div>
                                          <div className="bg-muted p-2 rounded text-xs whitespace-pre-wrap">
                                            {step.result}
                                          </div>
                                        </div>
                                      )}
                                      {step.error && (
                                        <div className="text-xs text-red-600 dark:text-red-400">
                                          <div className="font-medium mb-1">Error:</div>
                                          <div className="bg-red-50 dark:bg-red-950 p-2 rounded text-xs">
                                            {step.error}
                                          </div>
                                        </div>
                                      )}
                                      {!step.result && !step.error && step.status === 'pending' && (
                                        <div className="text-xs text-muted-foreground">
                                          Waiting for tool execution...
                                        </div>
                                      )}
                                    </div>
                                  </ChainOfThoughtStep>
                                );
                              }
                            })}
                          </ChainOfThoughtContent>
                        </ChainOfThought>
                      ) : null}
                      {/* Render content and tools in order using split markers (for loaded sessions with contentBeforeTool) */}
                      {message.from === 'assistant' && 
                       !message.thinkingSteps?.some(s => s.type === 'reasoning') && 
                       message.thinkingSteps && 
                       message.thinkingSteps.some(s => s.type === 'tool' && s.contentBeforeTool !== undefined) ? (
                        <>
                          {(() => {
                            let contentPosition = 0;
                            const elements: React.ReactNode[] = [];

                            message.thinkingSteps!.forEach((step, idx) => {
                              if (step.type === 'tool') {
                                // Show content before this tool
                                if (step.contentBeforeTool) {
                                  elements.push(
                                    <MessageContent key={`content-before-${idx}`}>
                                      <MessageResponse>{step.contentBeforeTool}</MessageResponse>
                                    </MessageContent>
                                  );
                                  contentPosition += step.contentBeforeTool.length;
                                }

                                // Find corresponding tool approval if exists
                                const approval = message.toolApprovals?.find(
                                  a => a.tool_name === step.name
                                );
                                const decision = approval ? toolApprovalDecisions.get(approval.approval_id) : null;

                                // Show tool approval or result
                                if (approval) {
                                  elements.push(
                                    <ToolApprovalComponent
                                      key={`tool-${approval.approval_id}`}
                                      approval={approval}
                                      onApprove={(save_decision, scope) =>
                                        handleToolApprove(approval.approval_id, save_decision, scope)
                                      }
                                      onReject={(save_decision) =>
                                        handleToolReject(approval.approval_id, save_decision)
                                      }
                                      isApproved={decision?.approved === true}
                                      isRejected={decision?.approved === false}
                                    />
                                  );
                                } else if (step.result || step.error) {
                                  // No approval exists but we have a result (loaded from session)
                                  // Show the tool execution details using Tool component
                                  const toolState = step.status === 'error' ? 'output-error' : 'output-available';
                                  elements.push(
                                    <Tool key={`tool-result-${idx}`} className="mt-4">
                                      <ToolHeader
                                        type="dynamic-tool"
                                        toolName={step.name}
                                        state={toolState}
                                      />
                                      <ToolContent>
                                        {step.parameters && Object.keys(step.parameters).length > 0 && (
                                          <ToolInput input={step.parameters} />
                                        )}
                                        <ToolOutput
                                          errorText={step.error}
                                          output={step.result}
                                        />
                                      </ToolContent>
                                    </Tool>
                                  );
                                }
                              }
                            });

                            // Show remaining content after all tools
                            if (contentPosition < version.content.length) {
                              const remainingContent = version.content.substring(contentPosition);
                              if (remainingContent.trim()) {
                                elements.push(
                                  <MessageContent key="content-after-tools">
                                    <MessageResponse>{remainingContent}</MessageResponse>
                                  </MessageContent>
                                );
                              }
                            }

                            return <>{elements}</>;
                          })()}
                        </>
                      ) : message.from === 'assistant' && !message.thinkingSteps?.some(s => s.type === 'reasoning') && message.toolApprovals && message.toolApprovals.length > 0 ? (
                        <>
                          {/* Fallback: Legacy split mode using contentBeforeApproval */}
                          {message.toolApprovals[0].contentBeforeApproval !== undefined ? (
                            <>
                              {/* Split mode: content before approval */}
                              {message.toolApprovals[0].contentBeforeApproval && (
                                <MessageContent>
                                  <MessageResponse>
                                    {message.toolApprovals[0].contentBeforeApproval}
                                  </MessageResponse>
                                </MessageContent>
                              )}
                              {/* Tool approvals */}
                              {message.toolApprovals.map((approval) => {
                                const decision = toolApprovalDecisions.get(approval.approval_id);
                                return (
                                  <ToolApprovalComponent
                                    key={approval.approval_id}
                                    approval={approval}
                                    onApprove={(save_decision, scope) =>
                                      handleToolApprove(approval.approval_id, save_decision, scope)
                                    }
                                    onReject={(save_decision) =>
                                      handleToolReject(approval.approval_id, save_decision)
                                    }
                                    isApproved={decision?.approved === true}
                                    isRejected={decision?.approved === false}
                                  />
                                );
                              })}
                              {/* Split mode: content after approval */}
                              {version.content && 
                               version.content.length > (message.toolApprovals[0].contentBeforeApproval?.length || 0) && (
                                <MessageContent>
                                  <MessageResponse>
                                    {version.content.substring(message.toolApprovals[0].contentBeforeApproval?.length || 0)}
                                  </MessageResponse>
                                </MessageContent>
                              )}
                            </>
                          ) : (
                            <>
                              {/* No split mode: show all content then approval (streaming before approval or legacy sessions) */}
                              {version.content && (
                                <MessageContent>
                                  <MessageResponse>
                                    {version.content}
                                  </MessageResponse>
                                </MessageContent>
                              )}
                              {/* Tool approvals */}
                              {message.toolApprovals.map((approval) => {
                                const decision = toolApprovalDecisions.get(approval.approval_id);
                                return (
                                  <ToolApprovalComponent
                                    key={approval.approval_id}
                                    approval={approval}
                                    onApprove={(save_decision, scope) =>
                                      handleToolApprove(approval.approval_id, save_decision, scope)
                                    }
                                    onReject={(save_decision) =>
                                      handleToolReject(approval.approval_id, save_decision)
                                    }
                                    isApproved={decision?.approved === true}
                                    isRejected={decision?.approved === false}
                                  />
                                );
                              })}
                            </>
                          )}
                        </>
                      ) : (
                        <>
                          {/* Normal message content (reasoning mode or no approvals) */}
                          <MessageContent>
                            {message.from === 'assistant' &&
                            !version.content &&
                            status === 'streaming' &&
                            !message.thinkingSteps ? (
                              <Shimmer className="text-muted-foreground">Thinking...</Shimmer>
                            ) : (
                              <MessageResponse>{version.content}</MessageResponse>
                            )}
                          </MessageContent>
                          {/* Tool approvals for reasoning mode (after content) */}
                          {message.toolApprovals?.map((approval) => {
                            const decision = toolApprovalDecisions.get(approval.approval_id);
                            const hasReasoning = message.thinkingSteps?.some(s => s.type === 'reasoning');
                            // Hide approved/rejected approvals when chain of thought is active (has reasoning)
                            if (hasReasoning && decision) {
                              return null;
                            }
                            return (
                              <ToolApprovalComponent
                                key={approval.approval_id}
                                approval={approval}
                                onApprove={(save_decision, scope) =>
                                  handleToolApprove(approval.approval_id, save_decision, scope)
                                }
                                onReject={(save_decision) =>
                                  handleToolReject(approval.approval_id, save_decision)
                                }
                                isApproved={decision?.approved === true}
                                isRejected={decision?.approved === false}
                              />
                            );
                          })}
                        </>
                      )}
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
      </div>
      <div className="grid shrink-0 gap-4 border-t pt-4">
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
                {/* <SpeechInput
                  className="shrink-0"
                  onTranscriptionChange={handleTranscriptionChange}
                  size="icon-sm"
                  variant="ghost"
                /> */}
                {/* <PromptInputButton onClick={toggleWebSearch} variant={useWebSearch ? 'default' : 'ghost'}>
                  <GlobeIcon size={16} />
                  <span>Search</span>
                </PromptInputButton> */}
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
                              <ModelItem isSelected={selectedModel === m.id} key={m.id} m={m} onSelect={handleModelSelect} />
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

      {/* Source Content Sidebar */}
      {sourceContentOpen && sourceContentData && (
        <SourceContentSidebar
          title={sourceContentData.title}
          content={sourceContentData.content}
          language={getLanguageFromFilename(sourceContentData.title)}
          onClose={() => setSourceContentOpen(false)}
        />
      )}
    </div>
  );
};

export default Chatbot;
