import { useCallback, useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { toast } from 'sonner';
import { FileText, Trash2, Upload, RefreshCw, Database } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface DocumentSummary {
  id: number;
  filename: string;
  file_size: number;
  updated_at: number;
}

interface RagStats {
  doc_count: number;
  chunk_count: number;
  embedding_count: number;
  avg_chunks_per_doc: number;
}

interface DocumentManagerProps {
  apiUrl?: string;
  onDocumentChange?: () => void;
}

export function DocumentManager({ apiUrl = '', onDocumentChange }: DocumentManagerProps) {
  const [documents, setDocuments] = useState<DocumentSummary[]>([]);
  const [stats, setStats] = useState<RagStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false);
  const [filename, setFilename] = useState('');
  const [content, setContent] = useState('');
  const [fileInputKey, setFileInputKey] = useState(Date.now());

  const endpoint = useCallback((path: string) => {
    return apiUrl ? `${apiUrl}${path}` : path;
  }, [apiUrl]);

  const loadDocuments = useCallback(async () => {
    setLoading(true);
    try {
      const response = await fetch(endpoint('/api/rag/documents'));
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      setDocuments(data.documents || []);
    } catch (error) {
      console.error('Failed to load documents:', error);
      toast.error('Failed to load documents', {
        description: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setLoading(false);
    }
  }, [endpoint]);

  const loadStats = useCallback(async () => {
    try {
      const response = await fetch(endpoint('/api/rag/stats'));
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      setStats(data);
    } catch (error) {
      console.error('Failed to load stats:', error);
    }
  }, [endpoint]);

  useEffect(() => {
    void loadDocuments();
    void loadStats();
  }, [loadDocuments, loadStats]);

  const handleFileSelect = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setFilename(file.name);
      const reader = new FileReader();
      reader.onload = (e) => {
        const text = e.target?.result as string;
        setContent(text);
      };
      reader.readAsText(file);
    }
  }, []);

  const handleUpload = useCallback(async () => {
    if (!filename || !content) {
      toast.error('Missing information', {
        description: 'Please provide both filename and content',
      });
      return;
    }

    setUploading(true);
    try {
      const response = await fetch(endpoint('/api/rag/upload'), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ filename, content }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const result = await response.json();
      toast.success('Document uploaded', {
        description: result.message,
      });

      setUploadDialogOpen(false);
      setFilename('');
      setContent('');
      setFileInputKey(Date.now());
      void loadDocuments();
      void loadStats();
      onDocumentChange?.();
    } catch (error) {
      console.error('Failed to upload document:', error);
      toast.error('Upload failed', {
        description: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setUploading(false);
    }
  }, [filename, content, endpoint, loadDocuments, loadStats, onDocumentChange]);

  const handleDelete = useCallback(
    async (docFilename: string) => {
      if (!confirm(`Are you sure you want to delete "${docFilename}"?`)) {
        return;
      }

      try {
        const response = await fetch(endpoint(`/api/rag/documents/${encodeURIComponent(docFilename)}`), {
          method: 'DELETE',
        });

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const result = await response.json();
        toast.success('Document deleted', {
          description: result.message,
        });

        void loadDocuments();
        void loadStats();
        onDocumentChange?.();
      } catch (error) {
        console.error('Failed to delete document:', error);
        toast.error('Delete failed', {
          description: error instanceof Error ? error.message : String(error),
        });
      }
    },
    [endpoint, loadDocuments, loadStats, onDocumentChange]
  );

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
  };

  const formatDate = (timestamp: number): string => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Database className="h-5 w-5" />
              RAG Documents
            </CardTitle>
            <CardDescription>Manage your indexed documents for semantic search</CardDescription>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={() => { void loadDocuments(); void loadStats(); }}>
              <RefreshCw className="h-4 w-4 mr-2" />
              Refresh
            </Button>
            <Dialog open={uploadDialogOpen} onOpenChange={setUploadDialogOpen}>
              <DialogTrigger asChild>
                <Button size="sm">
                  <Upload className="h-4 w-4 mr-2" />
                  Upload
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Upload Document</DialogTitle>
                  <DialogDescription>
                    Upload a document to be indexed for semantic search
                  </DialogDescription>
                </DialogHeader>
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label htmlFor="file-upload">Select File</Label>
                    <Input
                      id="file-upload"
                      key={fileInputKey}
                      type="file"
                      accept=".txt,.md,.rst,.js,.ts,.py,.rs,.go,.java,.cpp,.c,.h,.hpp,.json,.yaml,.yml,.toml,.xml,.html,.css"
                      onChange={handleFileSelect}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="filename">Filename</Label>
                    <Input
                      id="filename"
                      placeholder="document.md"
                      value={filename}
                      onChange={(e) => setFilename(e.target.value)}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="content">Content</Label>
                    <Textarea
                      id="content"
                      placeholder="Enter or paste document content..."
                      value={content}
                      onChange={(e) => setContent(e.target.value)}
                      rows={10}
                      className="font-mono text-sm"
                    />
                  </div>
                  <div className="flex justify-end gap-2">
                    <Button
                      variant="outline"
                      onClick={() => {
                        setUploadDialogOpen(false);
                        setFilename('');
                        setContent('');
                        setFileInputKey(Date.now());
                      }}
                    >
                      Cancel
                    </Button>
                    <Button onClick={() => { void handleUpload(); }} disabled={uploading || !filename || !content}>
                      {uploading ? 'Uploading...' : 'Upload'}
                    </Button>
                  </div>
                </div>
              </DialogContent>
            </Dialog>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {stats && (
          <div className="grid grid-cols-4 gap-4 mb-4">
            <div className="bg-muted rounded-lg p-3">
              <div className="text-2xl font-bold">{stats.doc_count}</div>
              <div className="text-xs text-muted-foreground">Documents</div>
            </div>
            <div className="bg-muted rounded-lg p-3">
              <div className="text-2xl font-bold">{stats.chunk_count}</div>
              <div className="text-xs text-muted-foreground">Chunks</div>
            </div>
            <div className="bg-muted rounded-lg p-3">
              <div className="text-2xl font-bold">{stats.embedding_count}</div>
              <div className="text-xs text-muted-foreground">Embeddings</div>
            </div>
            <div className="bg-muted rounded-lg p-3">
              <div className="text-2xl font-bold">{stats.avg_chunks_per_doc.toFixed(1)}</div>
              <div className="text-xs text-muted-foreground">Avg Chunks/Doc</div>
            </div>
          </div>
        )}

        {loading ? (
          <div className="flex items-center justify-center py-8">
            <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : documents.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <FileText className="h-12 w-12 mx-auto mb-2 opacity-50" />
            <p>No documents indexed yet</p>
            <p className="text-sm">Upload a document to get started</p>
          </div>
        ) : (
          <div className="space-y-2">
            {documents.map((doc) => (
              <div
                key={doc.id}
                className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors"
              >
                <div className="flex items-center gap-3 flex-1">
                  <FileText className="h-4 w-4 text-muted-foreground" />
                  <div className="flex-1 min-w-0">
                    <div className="font-medium truncate">{doc.filename}</div>
                    <div className="text-xs text-muted-foreground">
                      {formatFileSize(doc.file_size)} • Updated {formatDate(doc.updated_at)}
                    </div>
                  </div>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => { void handleDelete(doc.filename); }}
                  className="text-destructive hover:text-destructive hover:bg-destructive/10"
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
