import type { BundledLanguage } from 'shiki';
import { FileIcon } from 'lucide-react';
import {
  CodeBlock,
  CodeBlockActions,
  CodeBlockCopyButton,
  CodeBlockFilename,
  CodeBlockHeader,
  CodeBlockTitle,
} from '@/components/ai-elements/code-block';

interface SourceContentSidebarProps {
  title: string;
  content: string;
  language: BundledLanguage;
  onClose: () => void;
}

export const SourceContentSidebar = ({ title, content, language, onClose }: SourceContentSidebarProps) => {
  return (
    <div className="fixed right-0 top-0 h-full w-[600px] border-l bg-background shadow-lg z-50 flex flex-col">
      <div className="flex items-center justify-between border-b p-4">
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold truncate">{title}</h3>
          <p className="text-xs text-muted-foreground">
            {content.length.toLocaleString()} characters
          </p>
        </div>
        <button
          onClick={onClose}
          className="ml-2 rounded-md p-2 hover:bg-accent"
          type="button"
          aria-label="Close sidebar"
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
          code={content}
          language={language}
          showLineNumbers={true}
          className="rounded-none border-0 border-t"
        >
          <CodeBlockHeader>
            <CodeBlockTitle>
              <FileIcon size={14} />
              <CodeBlockFilename>{title}</CodeBlockFilename>
            </CodeBlockTitle>
            <CodeBlockActions>
              <CodeBlockCopyButton />
            </CodeBlockActions>
          </CodeBlockHeader>
        </CodeBlock>
      </div>
    </div>
  );
};
