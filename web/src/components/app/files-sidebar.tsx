import * as React from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
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
  const navigate = useNavigate();
  const location = useLocation();
  const [files, setFiles] = React.useState<FileNode[]>([]);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);
  const [selectedPath, setSelectedPath] = React.useState<string | undefined>();
  const [expandedPaths, setExpandedPaths] = React.useState<Set<string>>(new Set());
  
  // Extract file path from current route
  const currentFilePath = React.useMemo(() => {
    if (location.pathname.startsWith('/workspace/files/')) {
      return location.pathname.replace('/workspace/files/', '');
    }
    return undefined;
  }, [location.pathname]);

  // Sync selected path and expand parent directories when URL changes
  React.useEffect(() => {
    if (currentFilePath) {
      setSelectedPath(currentFilePath);
      
      // Expand all parent directories of the current file
      const parts = currentFilePath.split('/');
      let currentPath = '';
      
      // Build paths for each parent directory
      const newExpanded = new Set<string>();
      for (let i = 0; i < parts.length - 1; i++) {
        if (i === 0) {
          currentPath = parts[i];
        } else {
          currentPath += '/' + parts[i];
        }
        newExpanded.add(currentPath);
      }
      
      setExpandedPaths(prev => {
        // Merge with existing expanded paths to preserve user's manual expansions
        const merged = new Set([...prev, ...newExpanded]);
        return merged;
      });
    }
  }, [currentFilePath]);
  
  // Keep track of all file paths (not directories) for quick lookup
  const filePaths = React.useMemo(() => {
    const paths = new Set<string>();
    const collectFilePaths = (nodes: FileNode[]) => {
      nodes.forEach(node => {
        if (!node.is_dir) {
          paths.add(node.path);
        }
        if (node.children) {
          collectFilePaths(node.children);
        }
      });
    };
    collectFilePaths(files);
    return paths;
  }, [files]);

  const handleFileSelect = React.useCallback((path: string) => {
    setSelectedPath(path);
    // Only navigate if this is a file, not a directory
    if (filePaths.has(path)) {
      navigate(`/workspace/files/${path}`);
    }
  }, [navigate, filePaths]);

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

  const handleExpandedChange = React.useCallback((expanded: Set<string>) => {
    setExpandedPaths(expanded);
  }, []);

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
                expanded={expandedPaths}
                onExpandedChange={handleExpandedChange}
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
