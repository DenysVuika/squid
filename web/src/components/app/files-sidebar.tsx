import * as React from 'react';
import { Files, Loader2 } from 'lucide-react';
import {
  FileTree,
  FileTreeFile,
  FileTreeFolder,
} from '@/components/ai-elements/file-tree';

interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[];
}

export function FilesSidebar() {
  const [files, setFiles] = React.useState<FileNode[]>([]);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);
  const [selectedPath, setSelectedPath] = React.useState<string | undefined>();

  const handleFileSelect = React.useCallback((path: string) => {
    setSelectedPath(path);
  }, []);

  React.useEffect(() => {
    const fetchFiles = async () => {
      try {
        setLoading(true);
        setError(null);
        const response = await fetch('/api/workspace/files');
        if (!response.ok) {
          throw new Error('Failed to fetch files');
        }
        const data = await response.json();
        setFiles(data.files || []);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    void fetchFiles();
  }, []);

  const renderFileTree = (nodes: FileNode[]) => {
    return nodes.map((node) => {
      if (node.is_dir && node.children) {
        return (
          <FileTreeFolder key={node.path} name={node.name} path={node.path}>
            {renderFileTree(node.children)}
          </FileTreeFolder>
        );
      }
      return <FileTreeFile key={node.path} name={node.name} path={node.path} />;
    });
  };

  // Build initial expanded paths for all directories
  const buildExpandedPaths = (): Set<string> => {
    // Don't auto-expand any directories - let users expand them manually
    return new Set<string>();
  };

  const defaultExpanded = React.useMemo(
    () => buildExpandedPaths(),
    []
  );

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="border-b p-4">
        <div className="flex items-center gap-2">
          <Files className="h-5 w-5" />
          <span className="font-semibold text-lg">Files</span>
        </div>
      </div>
      <div className="flex-1 overflow-auto p-4">
        {loading && (
          <div className="flex items-center justify-center py-4">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}
        {error && (
          <div className="text-sm text-destructive py-4">
            Error: {error}
          </div>
        )}
        {!loading && !error && files.length === 0 && (
          <div className="text-sm text-muted-foreground py-4">
            No files found
          </div>
        )}
            {!loading && !error && files.length > 0 && (
              <FileTree
                defaultExpanded={defaultExpanded}
                // @ts-expect-error - FileTree has type conflict between HTML onSelect and custom onSelect
                onSelect={handleFileSelect}
                selectedPath={selectedPath}
              >
                {renderFileTree(files)}
              </FileTree>
            )}
      </div>
    </div>
  );
}
