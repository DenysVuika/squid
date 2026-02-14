import type { BundledLanguage } from 'shiki';
import { FileIcon, XIcon } from 'lucide-react';
import {
  CodeBlock,
  CodeBlockActions,
  CodeBlockCopyButton,
  CodeBlockFilename,
  CodeBlockHeader,
  CodeBlockTitle,
} from '@/components/ai-elements/code-block';
import { Button } from '@/components/ui/button';

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
        <Button
          onClick={onClose}
          variant="ghost"
          size="icon"
          className="ml-2"
          aria-label="Close sidebar"
        >
          <XIcon size={16} />
        </Button>
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
