import { useParams } from 'react-router-dom';
import { useEffect, useState, useCallback, useMemo } from 'react';
import { Loader2, FileIcon, CopyIcon, DownloadIcon } from 'lucide-react';
import type { BundledLanguage } from 'shiki';
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
  PromptInputFooter,
  PromptInputSubmit,
  PromptInputTextarea,
  PromptInputTools,
} from '@/components/ai-elements/prompt-input';
import { toast } from 'sonner';

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

  const handlePromptSubmit = useCallback(() => {
    // Placeholder for future implementation
    console.log('Prompt submitted:', promptText);
  }, [promptText]);

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
