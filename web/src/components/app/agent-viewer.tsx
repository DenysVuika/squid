import { useParams } from 'react-router-dom';
import { useEffect, useState, useCallback } from 'react';
import { Loader2, Bot, CopyIcon } from 'lucide-react';
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
import { fetchAgentContent, type AgentContentResponse } from '@/lib/chat-api';
import { toast } from 'sonner';

export function AgentViewer() {
  const { id: agentId } = useParams<{ id: string }>();
  const [agentData, setAgentData] = useState<AgentContentResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchContent = async () => {
      if (!agentId) {
        setError('No agent ID provided');
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        setError(null);
        const data = await fetchAgentContent('', agentId);
        setAgentData(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load agent content');
      } finally {
        setLoading(false);
      }
    };

    void fetchContent();
  }, [agentId]);

  const handleCopy = useCallback(async () => {
    if (!agentData?.content) return;
    try {
      await navigator.clipboard.writeText(agentData.content);
      toast.success('Copied to clipboard');
    } catch {
      toast.error('Failed to copy to clipboard');
    }
  }, [agentData?.content]);

  if (loading) {
    return (
      <div className="flex items-center justify-center flex-1">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center flex-1">
        <div className="text-sm text-destructive">Error: {error}</div>
      </div>
    );
  }

  if (!agentData) {
    return (
      <div className="flex items-center justify-center flex-1">
        <div className="text-sm text-muted-foreground">Agent not found</div>
      </div>
    );
  }

  return (
    <div className="flex flex-1 w-full flex-col overflow-hidden min-h-0 p-4">
      <Artifact className="flex-1 flex flex-col min-h-0">
        <ArtifactHeader>
          <div>
            <ArtifactTitle>
              <div className="flex items-center gap-2">
                <Bot className="h-4 w-4" />
                <span>{agentData.name}</span>
              </div>
            </ArtifactTitle>
            <ArtifactDescription>Agent: {agentData.id}</ArtifactDescription>
          </div>
          <ArtifactActions>
            <ArtifactAction icon={CopyIcon} label="Copy" onClick={handleCopy} tooltip="Copy to clipboard" />
          </ArtifactActions>
        </ArtifactHeader>
        <ArtifactContent className="p-0 flex-1 min-h-0 overflow-auto">
          <div className="agent-viewer-wrap">
            <CodeBlock
              className="border-none rounded-none"
              code={agentData.content}
              language="markdown"
              showLineNumbers
            />
          </div>
        </ArtifactContent>
      </Artifact>
    </div>
  );
}
