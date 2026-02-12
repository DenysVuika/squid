import type { PromptInputMessage } from '@/components/ai-elements/prompt-input';
import type { FileUIPart, ToolUIPart } from 'ai';

import { streamChat } from '@/lib/chat-api';
import { Attachment, AttachmentPreview, AttachmentRemove, Attachments } from '@/components/ai-elements/attachments';
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
import { nanoid } from 'nanoid';
import { useCallback, useMemo, useRef, useState } from 'react';
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

const initialMessages: MessageType[] = [
  {
    from: 'user',
    key: nanoid(),
    versions: [
      {
        content: 'Can you explain how to use React hooks effectively?',
        id: nanoid(),
      },
    ],
  },
  {
    from: 'assistant',
    key: nanoid(),
    sources: [
      {
        href: 'https://react.dev/reference/react',
        title: 'React Documentation',
      },
      {
        href: 'https://react.dev/reference/react-dom',
        title: 'React DOM Documentation',
      },
    ],
    tools: [
      {
        description: 'Searching React documentation',
        error: undefined,
        name: 'mcp',
        parameters: {
          query: 'React hooks best practices',
          source: 'react.dev',
        },
        result: `{
  "query": "React hooks best practices",
  "results": [
    {
      "title": "Rules of Hooks",
      "url": "https://react.dev/warnings/invalid-hook-call-warning",
      "snippet": "Hooks must be called at the top level of your React function components or custom hooks. Don't call hooks inside loops, conditions, or nested functions."
    },
    {
      "title": "useState Hook",
      "url": "https://react.dev/reference/react/useState",
      "snippet": "useState is a React Hook that lets you add state to your function components. It returns an array with two values: the current state and a function to update it."
    },
    {
      "title": "useEffect Hook",
      "url": "https://react.dev/reference/react/useEffect",
      "snippet": "useEffect lets you synchronize a component with external systems. It runs after render and can be used to perform side effects like data fetching."
    }
  ]
}`,
        status: 'input-available',
      },
    ],
    versions: [
      {
        content: `# React Hooks Best Practices

React hooks are a powerful feature that let you use state and other React features without writing classes. Here are some tips for using them effectively:

## Rules of Hooks

1. **Only call hooks at the top level** of your component or custom hooks
2. **Don't call hooks inside loops, conditions, or nested functions**

## Common Hooks

- **useState**: For local component state
- **useEffect**: For side effects like data fetching
- **useContext**: For consuming context
- **useReducer**: For complex state logic
- **useCallback**: For memoizing functions
- **useMemo**: For memoizing values

## Example of useState and useEffect

\`\`\`jsx
function ProfilePage({ userId }) {
  const [user, setUser] = useState(null);

  useEffect(() => {
    // This runs after render and when userId changes
    fetchUser(userId).then(userData => {
      setUser(userData);
    });
  }, [userId]);

  return user ? <Profile user={user} /> : <Loading />;
}
\`\`\`

Would you like me to explain any specific hook in more detail?`,
        id: nanoid(),
      },
    ],
  },
  {
    from: 'user',
    key: nanoid(),
    versions: [
      {
        content: 'Yes, could you explain useCallback and useMemo in more detail? When should I use one over the other?',
        id: nanoid(),
      },
      {
        content:
          "I'm particularly interested in understanding the performance implications of useCallback and useMemo. Could you break down when each is most appropriate?",
        id: nanoid(),
      },
      {
        content:
          'Thanks for the overview! Could you dive deeper into the specific use cases where useCallback and useMemo make the biggest difference in React applications?',
        id: nanoid(),
      },
    ],
  },
  {
    from: 'assistant',
    key: nanoid(),
    reasoning: {
      content: `The user is asking for a detailed explanation of useCallback and useMemo. I should provide a clear and concise explanation of each hook's purpose and how they differ.

The useCallback hook is used to memoize functions to prevent unnecessary re-renders of child components that receive functions as props.

The useMemo hook is used to memoize values to avoid expensive recalculations on every render.

Both hooks help with performance optimization, but they serve different purposes.`,
      duration: 10,
    },
    versions: [
      {
        content: `## useCallback vs useMemo

Both hooks help with performance optimization, but they serve different purposes:

### useCallback

\`useCallback\` memoizes **functions** to prevent unnecessary re-renders of child components that receive functions as props.

\`\`\`jsx
// Without useCallback - a new function is created on every render
const handleClick = () => {
  console.log(count);
};

// With useCallback - the function is only recreated when dependencies change
const handleClick = useCallback(() => {
  console.log(count);
}, [count]);
\`\`\`

### useMemo

\`useMemo\` memoizes **values** to avoid expensive recalculations on every render.

\`\`\`jsx
// Without useMemo - expensive calculation runs on every render
const sortedList = expensiveSort(items);

// With useMemo - calculation only runs when items change
const sortedList = useMemo(() => expensiveSort(items), [items]);
\`\`\`

### When to use which?

- Use **useCallback** when:
  - Passing callbacks to optimized child components that rely on reference equality
  - Working with event handlers that you pass to child components

- Use **useMemo** when:
  - You have computationally expensive calculations
  - You want to avoid recreating objects that are used as dependencies for other hooks

### Performance Note

Don't overuse these hooks! They come with their own overhead. Only use them when you have identified a genuine performance issue.`,
        id: nanoid(),
      },
    ],
  },
];

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
  const [, setStreamingMessageId] = useState<string | null>(null);
  const streamingContentRef = useRef<string>('');
  const abortControllerRef = useRef<AbortController | null>(null);

  const selectedModelData = useMemo(() => models.find((m) => m.id === model), [model]);

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
        // Read file content if files are attached
        let fileContent: string | undefined;
        let fileName: string | undefined;

        if (files && files.length > 0) {
          const file = files[0];
          if (file.type === 'file' && file.url) {
            // Use the actual filename from the file object
            // FileUIPart from 'ai' package includes filename property
            fileName = 'filename' in file ? String(file.filename) : 'attachment';

            // Try to read the file content
            try {
              const response = await fetch(file.url);
              if (response.ok) {
                fileContent = await response.text();
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

        // Use relative path since web UI is served from the same server
        await streamChat(
          '',
          {
            message: userMessage,
            file_content: fileContent,
            file_path: fileName,
          },
          {
            signal: abortControllerRef.current?.signal,
            onContent: (text) => {
              // Accumulate content in ref for better performance
              streamingContentRef.current += text;
              const currentContent = streamingContentRef.current;

              setMessages((prev) =>
                prev.map((msg) => {
                  if (msg.versions.some((v) => v.id === messageId)) {
                    return {
                      ...msg,
                      versions: msg.versions.map((v) => (v.id === messageId ? { ...v, content: currentContent } : v)),
                    };
                  }
                  return msg;
                })
              );
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
            onDone: () => {
              streamingContentRef.current = ''; // Clear ref after streaming
              abortControllerRef.current = null;
              setStatus('ready');
              setStreamingMessageId(null);
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
    [updateMessageContent]
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

        // Create sources from attached files
        const sources = files?.map((file) => ({
          href: file.url || '#',
          title: 'filename' in file ? String(file.filename) : 'Attached file',
        }));

        const assistantMessage: MessageType = {
          from: 'assistant',
          key: `assistant-${Date.now()}`,
          ...(sources && sources.length > 0 && { sources }),
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

  const isSubmitDisabled = useMemo(() => !(text.trim() || status), [text, status]);

  return (
    <div className="relative flex size-full flex-col divide-y overflow-hidden">
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
                        <MessageResponse>{version.content}</MessageResponse>
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
  );
};

export default Example;
