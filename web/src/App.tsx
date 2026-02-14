import { BrowserRouter, Routes, Route, useLocation, useNavigate } from 'react-router-dom';
import ChatBot from './components/app/chatbot';
import Logs from './components/app/logs';
import { FileViewer } from './components/app/file-viewer';
import { AppSidebar } from './components/app/app-sidebar';
import { FilesSidebar } from './components/app/files-sidebar';
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar';
import { Separator } from '@/components/ui/separator';
import { Button } from './components/ui/button';
import { FileText, MessageSquare, FolderTree } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useSessionStore } from '@/stores/session-store';
import { useChatStore } from '@/stores/chat-store';
import { useModelStore } from '@/stores/model-store';

function AppContent() {
  const location = useLocation();
  const navigate = useNavigate();
  
  // Zustand stores
  const { sessions, activeSessionId, loadSessions, selectSession, startNewChat } = useSessionStore();
  const { clearMessages } = useChatStore();
  const { resetTokenUsage } = useModelStore();

  // State for right sidebar (files panel)
  const [showFilesPanel, setShowFilesPanel] = useState(false);

  useEffect(() => {
    void loadSessions();
  }, [loadSessions]);

  const handleSessionSelect = (sessionId: string) => {
    // Only load history if switching to a different session
    if (sessionId !== activeSessionId) {
      selectSession(sessionId);
      if (location.pathname !== '/') {
        navigate('/');
      }
    }
  };

  const handleNewChat = () => {
    startNewChat();
    clearMessages();
    resetTokenUsage();
    if (location.pathname !== '/') {
      navigate('/');
    }
  };

  const isLogsPage = location.pathname === '/logs';

  return (
    <SidebarProvider className="h-full">
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
          <div className="flex flex-1 items-center gap-2 px-4">
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
          {!isLogsPage && (
            <>
              <Separator orientation="vertical" className="h-4" />
              <Button
                variant="ghost"
                size="icon"
                className="mr-4"
                onClick={() => setShowFilesPanel(!showFilesPanel)}
                title="Toggle workspace files"
              >
                <FolderTree className={showFilesPanel ? 'text-primary' : ''} />
              </Button>
            </>
          )}
        </header>
        <div className="flex flex-1 overflow-hidden min-h-0">
          <div className="flex flex-1 flex-col overflow-hidden p-4">
            <Routes>
              <Route path="/" element={<ChatBot />} />
              <Route path="/logs" element={<Logs />} />
              <Route path="/workspace/files/*" element={<FileViewer />} />
            </Routes>
          </div>
          {!isLogsPage && showFilesPanel && (
            <div className="border-l w-80 shrink-0">
              <FilesSidebar />
            </div>
          )}
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

function App() {
  return (
    <BrowserRouter>
      <div className="h-full">
        <AppContent />
      </div>
    </BrowserRouter>
  );
}

export default App;
