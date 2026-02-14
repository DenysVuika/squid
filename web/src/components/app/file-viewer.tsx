import { useParams, useNavigate } from 'react-router-dom';
import { useEffect, useState, useCallback, useMemo } from 'react';
import { Loader2, FileIcon, CopyIcon, DownloadIcon } from 'lucide-react';
import type { BundledLanguage } from 'shiki';
import type { PromptInputMessage } from '@/components/ai-elements/prompt-input';
import {
  Artifact,
  ArtifactAction,
  ArtifactActions,
  ArtifactContent,
  ArtifactDescription,
  ArtifactHeader,
  ArtifactTitle,
} from '@/components/ai-elements/artifact';
import { CodeBlock } from '@/components/ai-elements/code-block';
import {
  PromptInput,
  PromptInputBody,
  PromptInputButton,
  PromptInputFooter,
  PromptInputSubmit,
  PromptInputTextarea,
  PromptInputTools,
} from '@/components/ai-elements/prompt-input';
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
import { Suggestions } from '@/components/ai-elements/suggestion';
import { toast } from 'sonner';

// App components
import { ModelItem } from './model-item';
import { SuggestionItem } from './suggestion-item';

// Zustand stores
import { useSessionStore } from '@/stores/session-store';
import { useChatStore } from '@/stores/chat-store';
import { useModelStore } from '@/stores/model-store';

const suggestions = [
  'Review this file for potential bugs',
  'Explain what this code does',
  'Suggest improvements for code quality',
  'Check for security vulnerabilities',
  'Analyze the code structure',
  'Find performance optimization opportunities',
  'Review coding standards compliance',
  'Identify potential refactoring areas',
];

// Detect language from filename
const getLanguageFromFilename = (filename: string): BundledLanguage => {
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
};

export function FileViewer() {
  const { '*': filePath } = useParams();
  const navigate = useNavigate();
  const [content, setContent] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [promptText, setPromptText] = useState('');

  // Zustand stores
  const { startNewChat } = useSessionStore();
  const { clearMessages, addUserMessage, setStatus } = useChatStore();
  const {
    models,
    modelGroups,
    selectedModel,
    modelSelectorOpen,
    setSelectedModel,
    setModelSelectorOpen,
    loadModels,
    resetTokenUsage,
  } = useModelStore();

  const selectedModelData = useMemo(() => models.find((m) => m.id === selectedModel), [selectedModel, models]);

  // Fetch available models on mount
  useEffect(() => {
    void loadModels();
  }, [loadModels]);

  useEffect(() => {
    const fetchFileContent = async () => {
      if (!filePath) {
        setError('No file path provided');
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        setError(null);
        const response = await fetch(`/api/workspace/files/${encodeURIComponent(filePath)}`);
        if (!response.ok) {
          throw new Error('Failed to fetch file content');
        }
        const text = await response.text();
        setContent(text);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    void fetchFileContent();
  }, [filePath]);

  const language = useMemo(() => {
    const result = filePath ? getLanguageFromFilename(filePath) : 'plaintext';
    return result as BundledLanguage;
  }, [filePath]);

  const fileName = useMemo(() => {
    return filePath ? filePath.split('/').pop() || filePath : 'Unknown';
  }, [filePath]);

  const handlePromptChange = useCallback((event: React.ChangeEvent<HTMLTextAreaElement>) => {
    setPromptText(event.target.value);
  }, []);

  const handleModelSelect = useCallback(
    (modelId: string) => {
      setSelectedModel(modelId);
    },
    [setSelectedModel]
  );

  const handlePromptSubmit = useCallback(
    (message: PromptInputMessage) => {
      const hasText = Boolean(message.text);

      if (!hasText || !filePath) {
        return;
      }

      // Create a file attachment with the current file
      const fileAttachment = {
        id: `file-${Date.now()}`,
        type: 'file' as const,
        url: `/api/workspace/files/${encodeURIComponent(filePath)}`,
        filename: fileName,
        mediaType: 'text/plain',
        size: content.length,
      };

      // Start a new chat session
      startNewChat();
      clearMessages();
      resetTokenUsage();

      // Navigate to the chat page
      navigate('/');

      // Small delay to ensure navigation completes and chat component is mounted
      setTimeout(() => {
        // Set status and add the message with file attachment
        setStatus('submitted');
        toast.success('File attached', {
          description: `${fileName} attached to message`,
        });
        addUserMessage(message.text, [fileAttachment]);
      }, 100);
    },
    [filePath, fileName, content, startNewChat, clearMessages, resetTokenUsage, navigate, setStatus, addUserMessage]
  );

  const handleSuggestionClick = useCallback(
    (suggestion: string) => {
      handlePromptSubmit({ text: suggestion, files: [] });
    },
    [handlePromptSubmit]
  );

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(content);
      toast.success('Copied to clipboard');
    } catch {
      toast.error('Failed to copy to clipboard');
    }
  }, [content]);

  const handleDownload = useCallback(() => {
    try {
      const blob = new Blob([content], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      toast.success('File downloaded');
    } catch {
      toast.error('Failed to download file');
    }
  }, [content, fileName]);

  return (
    <div className="relative flex flex-1 w-full flex-col overflow-hidden min-h-0">
      {/* Content Area with proper scrolling */}
      <div className="flex-1 min-h-0 p-4 flex flex-col">
        {loading && (
          <div className="flex items-center justify-center flex-1">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        )}
        {error && (
          <div className="flex items-center justify-center flex-1">
            <div className="text-sm text-destructive">
              Error: {error}
            </div>
          </div>
        )}
        {!loading && !error && content && (
          <Artifact className="flex-1 flex flex-col min-h-0">
            <ArtifactHeader>
              <div>
                <ArtifactTitle>
                  <div className="flex items-center gap-2">
                    <FileIcon className="h-4 w-4" />
                    <span>{fileName}</span>
                  </div>
                </ArtifactTitle>
                <ArtifactDescription>{filePath}</ArtifactDescription>
              </div>
              <ArtifactActions>
                <ArtifactAction
                  icon={CopyIcon}
                  label="Copy"
                  onClick={handleCopy}
                  tooltip="Copy to clipboard"
                />
                <ArtifactAction
                  icon={DownloadIcon}
                  label="Download"
                  onClick={handleDownload}
                  tooltip="Download file"
                />
              </ArtifactActions>
            </ArtifactHeader>
            <ArtifactContent className="p-0 flex-1 min-h-0">
              <CodeBlock
                className="border-none rounded-none"
                code={content}
                language={language}
                showLineNumbers
              />
            </ArtifactContent>
          </Artifact>
        )}
      </div>

      {/* Prompt Input Area */}
      <div className="grid shrink-0 gap-4 border-t pt-4">
        <Suggestions className="px-4">
          {suggestions.map((suggestion) => (
            <SuggestionItem key={suggestion} onClick={handleSuggestionClick} suggestion={suggestion} />
          ))}
        </Suggestions>
        <div className="w-full px-4 pb-4">
          <PromptInput
            onSubmit={handlePromptSubmit}
          >
            <PromptInputBody>
              <PromptInputTextarea
                onChange={handlePromptChange}
                value={promptText}
                placeholder="Ask about this file..."
              />
            </PromptInputBody>
            <PromptInputFooter>
              <PromptInputTools>
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
              <PromptInputSubmit disabled={!promptText.trim()} status={undefined} />
            </PromptInputFooter>
          </PromptInput>
        </div>
      </div>
    </div>
  );
}
