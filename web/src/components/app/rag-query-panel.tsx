import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { BookOpen, Sparkles } from 'lucide-react';

interface RagSource {
  filename: string;
  text: string;
  relevance: number;
}

interface RagQueryPanelProps {
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  sources?: RagSource[];
  isQuerying?: boolean;
}

export function RagQueryPanel({ enabled, onToggle, sources, isQuerying }: RagQueryPanelProps) {
  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Sparkles className="h-5 w-5" />
            <CardTitle>RAG Mode</CardTitle>
          </div>
          <div className="flex items-center gap-2">
            <Switch
              id="rag-mode"
              checked={enabled}
              onCheckedChange={onToggle}
            />
            <Label htmlFor="rag-mode" className="cursor-pointer">
              {enabled ? 'Enabled' : 'Disabled'}
            </Label>
          </div>
        </div>
        <CardDescription>
          {enabled
            ? 'Queries will be enhanced with relevant context from your documents'
            : 'Enable to search your documents for relevant context'}
        </CardDescription>
      </CardHeader>
      {enabled && sources && sources.length > 0 && (
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-center gap-2 text-sm font-medium">
              <BookOpen className="h-4 w-4" />
              Retrieved Sources ({sources.length})
            </div>
            <div className="space-y-2">
              {sources.map((source, idx) => (
                <div
                  key={idx}
                  className="p-3 border rounded-lg bg-muted/30 hover:bg-muted/50 transition-colors"
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="font-medium text-sm truncate flex-1">{source.filename}</div>
                    <Badge variant="secondary" className="ml-2">
                      {(source.relevance * 100).toFixed(0)}%
                    </Badge>
                  </div>
                  <div className="text-xs text-muted-foreground line-clamp-3">
                    {source.text}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      )}
      {enabled && isQuerying && (
        <CardContent>
          <div className="text-sm text-muted-foreground flex items-center gap-2">
            <div className="animate-pulse">🔍</div>
            Searching documents for relevant context...
          </div>
        </CardContent>
      )}
    </Card>
  );
}
