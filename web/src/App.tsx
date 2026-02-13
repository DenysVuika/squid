import { BrowserRouter, Routes, Route, useLocation, useNavigate } from 'react-router-dom';
import ChatBot from './components/app/chatbot';
import Logs from './components/app/logs';
import { AppSidebar } from './components/app/app-sidebar';
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar';
import { Separator } from '@/components/ui/separator';
import { Button } from './components/ui/button';
import { FileText, MessageSquare } from 'lucide-react';
import { useState, useEffect, useCallback } from 'react';

interface SessionListItem {
  session_id: string;
  message_count: number;
  created_at: string;
  updated_at: string;
  preview?: string;
  title?: string;
  model_id?: string;
  token_usage: {
    total_tokens: number;
    input_tokens: number;
    output_tokens: number;
    reasoning_tokens: number;
    cache_tokens: number;
    context_window: number;
    context_utilization: number;
  };
  cost_usd?: number;
}

interface ChatSession {
  id: string;
  title: string;
  isActive?: boolean;
}

function AppContent() {
  const location = useLocation();
  const navigate = useNavigate();
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  const loadSessions = useCallback(async () => {
    try {
      const response = await fetch('/api/sessions');
      if (response.ok) {
        const data: { sessions: SessionListItem[] } = await response.json();
        const chatSessions: ChatSession[] = (data.sessions || []).map((session) => ({
          id: session.session_id,
          title: session.title || session.preview || 'New Chat',
        }));
        setSessions(chatSessions);
      }
    } catch (error) {
      console.error('Failed to load sessions:', error);
    }
  }, []);

  useEffect(() => {
    loadSessions();
  }, [refreshTrigger, loadSessions]);

  const handleSessionSelect = (sessionId: string) => {
    setActiveSessionId(sessionId);
    if (location.pathname !== '/') {
      navigate('/');
    }
  };

  const handleNewChat = () => {
    setActiveSessionId(null);
    if (location.pathname !== '/') {
      navigate('/');
    }
  };

  const isLogsPage = location.pathname === '/logs';

  return (
    <SidebarProvider>
      {!isLogsPage && (
        <AppSidebar
          sessions={sessions}
          onSessionSelect={handleSessionSelect}
          onNewChat={handleNewChat}
          activeSessionId={activeSessionId || undefined}
        />
      )}
      <SidebarInset className="flex flex-col overflow-hidden">
        <header className="flex h-16 shrink-0 items-center gap-2 border-b">
          <div className="flex items-center gap-2 px-4">
            {!isLogsPage && (
              <>
                <SidebarTrigger className="-ml-1" />
                <Separator orientation="vertical" className="mr-2 h-4" />
              </>
            )}
            {isLogsPage && (
              <>
                <button
                  onClick={() => navigate('/')}
                  className="flex items-center gap-2 hover:opacity-80 transition-opacity"
                >
                  <span className="text-2xl">🦑</span>
                  <span className="font-bold text-xl">Squid</span>
                </button>
                <Separator orientation="vertical" className="mx-2 h-4" />
              </>
            )}
            <div className="flex gap-2">
              {isLogsPage ? (
                <Button variant="ghost" className="flex items-center gap-2" onClick={() => navigate('/')}>
                  <MessageSquare className="h-4 w-4" />
                  Back to Chat
                </Button>
              ) : (
                <Button variant="ghost" className="flex items-center gap-2" onClick={() => navigate('/logs')}>
                  <FileText className="h-4 w-4" />
                  Logs
                </Button>
              )}
            </div>
          </div>
        </header>
        <div className="flex flex-1 flex-col overflow-hidden min-h-0 p-4">
          <Routes>
            <Route
              path="/"
              element={
                <ChatBot
                  selectedSessionId={activeSessionId}
                  onSessionChange={(sessionId) => {
                    setActiveSessionId(sessionId);
                    setRefreshTrigger((prev) => prev + 1);
                  }}
                />
              }
            />
            <Route path="/logs" element={<Logs />} />
          </Routes>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

function App() {
  return (
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  );
}

export default App;
