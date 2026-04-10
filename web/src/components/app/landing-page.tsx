import { Button } from '@/components/ui/button';
import { Plus, MessageSquare, History } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useSessionStore } from '@/stores/session-store';
import { useEffect } from 'react';

export function LandingPage() {
  const navigate = useNavigate();
  const { sessions } = useSessionStore();

  const recentSessions = sessions.slice(0, 5);

  // Keyboard shortcut: Ctrl+N to start new chat
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
        e.preventDefault();
        navigate('/new');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [navigate]);

  return (
    <div className="flex flex-1 items-center justify-center p-8">
      <div className="max-w-2xl w-full space-y-8">
        <div className="text-center space-y-4">
          <h1 className="text-4xl font-bold tracking-tight">Welcome to Squid 🦑</h1>
          <p className="text-lg text-muted-foreground">
            Your AI-powered coding assistant with RAG, agents, and scheduled jobs
          </p>
        </div>

        <div className="flex justify-center">
          <Button
            size="lg"
            className="h-24 w-full max-w-md flex-col gap-2"
            onClick={() => navigate('/new')}
          >
            <Plus className="h-6 w-6" />
            <span className="text-base">Start New Chat</span>
          </Button>
        </div>

        {recentSessions.length > 0 && (
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
              <History className="h-4 w-4" />
              <span>Recent Sessions</span>
            </div>
            <div className="space-y-2">
              {recentSessions.map((session) => (
                <Button
                  key={session.id}
                  variant="ghost"
                  className="w-full justify-start h-auto py-3 px-4"
                  onClick={() => navigate(`/chat/${session.id}`)}
                >
                  <MessageSquare className="h-4 w-4 mr-3 shrink-0" />
                  <div className="flex-1 text-left truncate">
                    <div className="font-medium truncate">{session.title}</div>
                    {session.preview && (
                      <div className="text-sm text-muted-foreground truncate">
                        {session.preview}
                      </div>
                    )}
                  </div>
                  <div className="text-xs text-muted-foreground shrink-0 ml-4">
                    {new Date(session.updated_at).toLocaleDateString()}
                  </div>
                </Button>
              ))}
            </div>
          </div>
        )}

        <div className="text-center text-sm text-muted-foreground">
          <p>
            Press{' '}
            <kbd className="px-2 py-1 bg-muted rounded">
              {navigator.platform.indexOf('Mac') > -1 ? '⌘' : 'Ctrl'}+N
            </kbd>{' '}
            to start a new chat
          </p>
        </div>
      </div>
    </div>
  );
}
