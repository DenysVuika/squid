import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { fetchAllAgentStats, type AgentTokenStats } from '@/lib/chat-api';
import { Brain, TrendingUp, DollarSign, MessageSquare, Clock } from 'lucide-react';

interface AgentStatsCardProps {
  apiUrl: string;
}

const formatNumber = (num: number): string => {
  if (num >= 1000000) {
    return `${(num / 1000000).toFixed(1)}M`;
  }
  if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}K`;
  }
  return num.toString();
};

const formatCurrency = (amount: number): string => {
  return `$${amount.toFixed(4)}`;
};

const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString();
};

const AgentStatsItem = ({ stats }: { stats: AgentTokenStats }) => {
  return (
    <Card className="mb-4">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg flex items-center gap-2">
            <Brain className="h-5 w-5" />
            {stats.agent_id}
          </CardTitle>
          <Badge variant="secondary">{stats.total_sessions} sessions</Badge>
        </div>
        <CardDescription>
          Lifetime token usage statistics
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="flex flex-col">
            <div className="flex items-center gap-2 text-sm text-muted-foreground mb-1">
              <MessageSquare className="h-4 w-4" />
              <span>Total Tokens</span>
            </div>
            <div className="text-2xl font-bold">{formatNumber(stats.total_tokens)}</div>
            <div className="text-xs text-muted-foreground">
              {formatNumber(stats.input_tokens)} in / {formatNumber(stats.output_tokens)} out
            </div>
          </div>

          <div className="flex flex-col">
            <div className="flex items-center gap-2 text-sm text-muted-foreground mb-1">
              <DollarSign className="h-4 w-4" />
              <span>Total Cost</span>
            </div>
            <div className="text-2xl font-bold">{formatCurrency(stats.total_cost_usd)}</div>
            <div className="text-xs text-muted-foreground">
              {formatCurrency(stats.avg_cost_per_session)} per session
            </div>
          </div>

          <div className="flex flex-col">
            <div className="flex items-center gap-2 text-sm text-muted-foreground mb-1">
              <TrendingUp className="h-4 w-4" />
              <span>Cache Hit Rate</span>
            </div>
            <div className="text-2xl font-bold">
              {stats.total_tokens > 0 
                ? `${((stats.cache_tokens / stats.total_tokens) * 100).toFixed(1)}%`
                : '0%'
              }
            </div>
            <div className="text-xs text-muted-foreground">
              {formatNumber(stats.cache_tokens)} cached
            </div>
          </div>

          <div className="flex flex-col">
            <div className="flex items-center gap-2 text-sm text-muted-foreground mb-1">
              <Clock className="h-4 w-4" />
              <span>Usage Period</span>
            </div>
            <div className="text-sm font-semibold">
              {formatDate(stats.first_used_at)}
            </div>
            <div className="text-xs text-muted-foreground">
              to {formatDate(stats.last_used_at)}
            </div>
          </div>
        </div>

        {stats.reasoning_tokens > 0 && (
          <div className="mt-4 pt-4 border-t">
            <div className="text-sm text-muted-foreground mb-2">
              Reasoning tokens: {formatNumber(stats.reasoning_tokens)} 
              ({stats.total_tokens > 0 ? ((stats.reasoning_tokens / stats.total_tokens) * 100).toFixed(1) : 0}% of total)
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
};

export const AgentStatsCard = ({ apiUrl }: AgentStatsCardProps) => {
  const [stats, setStats] = useState<AgentTokenStats[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadStats = async () => {
      try {
        setLoading(true);
        const data = await fetchAllAgentStats(apiUrl);
        setStats(data.agents);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load agent statistics');
      } finally {
        setLoading(false);
      }
    };

    loadStats();
  }, [apiUrl]);

  if (loading) {
    return (
      <Card>
        <CardContent className="py-8">
          <div className="text-center text-muted-foreground">Loading agent statistics...</div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardContent className="py-8">
          <div className="text-center text-red-500">{error}</div>
        </CardContent>
      </Card>
    );
  }

  if (stats.length === 0) {
    return (
      <Card>
        <CardContent className="py-8">
          <div className="text-center text-muted-foreground">
            No agent statistics available yet. Start using agents to see usage data.
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div>
      {stats.map((agentStats) => (
        <AgentStatsItem key={agentStats.agent_id} stats={agentStats} />
      ))}
    </div>
  );
};
