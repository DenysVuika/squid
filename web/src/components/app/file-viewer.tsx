import { useParams } from 'react-router-dom';
import { useEffect, useState, useCallback, useMemo } from 'react';
import { Loader2, FileIcon } from 'lucide-react';
import type { BundledLanguage } from 'shiki';
import {
  CodeBlock,
  CodeBlockActions,
  CodeBlockCopyButton,
  CodeBlockFilename,
  CodeBlockHeader,
  CodeBlockTitle,
} from '@/components/ai-elements/code-block';
import {
  PromptInput,
  PromptInputBody,
  PromptInputFooter,
  PromptInputSubmit,
  PromptInputTextarea,
  PromptInputTools,
} from '@/components/ai-elements/prompt-input';

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
  const [content, setContent] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [promptText, setPromptText] = useState('');

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

  const handlePromptChange = useCallback((event: React.ChangeEvent<HTMLTextAreaElement>) => {
    setPromptText(event.target.value);
  }, []);

  const handlePromptSubmit = useCallback(() => {
    // Placeholder for future implementation
    console.log('Prompt submitted:', promptText);
  }, [promptText]);

  const language = useMemo(() => {
    const result = filePath ? getLanguageFromFilename(filePath) : 'plaintext';
    return result as BundledLanguage;
  }, [filePath]);

  const fileName = useMemo(() => {
    return filePath ? filePath.split('/').pop() || filePath : 'Unknown';
  }, [filePath]);

  return (
    <div className="relative flex flex-1 w-full flex-col overflow-hidden rounded-xl border bg-background min-h-0">
      {/* Header */}
      <div className="flex shrink-0 items-center justify-between border-b bg-white px-4 py-2 dark:bg-gray-950 rounded-t-xl">
        <div className="flex items-center gap-2 min-w-0 flex-1">
          <FileIcon className="h-4 w-4 text-muted-foreground shrink-0" />
          <h2 className="font-semibold truncate">{filePath || 'File Viewer'}</h2>
        </div>
      </div>

      {/* Content Area with proper scrolling */}
      <div className="flex-1 min-h-0 overflow-auto">
        {loading && (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        )}
        {error && (
          <div className="flex items-center justify-center h-full">
            <div className="text-sm text-destructive">
              Error: {error}
            </div>
          </div>
        )}
        {!loading && !error && content && (
          <CodeBlock
            code={content}
            language={language}
            showLineNumbers={true}
            className="rounded-none border-0"
          >
            <CodeBlockHeader>
              <CodeBlockTitle>
                <FileIcon size={14} />
                <CodeBlockFilename>{fileName}</CodeBlockFilename>
              </CodeBlockTitle>
              <CodeBlockActions>
                <CodeBlockCopyButton />
              </CodeBlockActions>
            </CodeBlockHeader>
          </CodeBlock>
        )}
      </div>

      {/* Prompt Input Area - Placeholder */}
      <div className="grid shrink-0 border-t pt-4">
        <div className="w-full px-4 pb-4">
          <PromptInput
            onSubmit={handlePromptSubmit}
          >
            <PromptInputBody>
              <PromptInputTextarea
                onChange={handlePromptChange}
                value={promptText}
                placeholder="Ask about this file... (coming soon)"
                disabled
              />
            </PromptInputBody>
            <PromptInputFooter>
              <PromptInputTools />
              <PromptInputSubmit disabled status={undefined} />
            </PromptInputFooter>
          </PromptInput>
        </div>
      </div>
    </div>
  );
}
