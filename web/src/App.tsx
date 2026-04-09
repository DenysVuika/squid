import { BrowserRouter, Routes, Route, useLocation, useNavigate } from 'react-router-dom';
import ChatBot from './components/app/chatbot';
import Logs from './components/app/logs';
import { FileViewer } from './components/app/file-viewer';
import { AgentViewer } from './components/app/agent-viewer';
import JobDetails from './components/app/job-details';
import { AppSidebar } from './components/app/app-sidebar';
import { FilesSidebar } from './components/app/files-sidebar';
import { DocumentManager } from './components/app/document-manager';
import { AgentStatsCard } from './components/app/agent-stats';
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar';
import { Separator } from '@/components/ui/separator';
import { Button } from './components/ui/button';
import { MessageSquare, Files, Database } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useSessionStore } from '@/stores/session-store';
import { useChatStore } from '@/stores/chat-store';
import { useAgentStore } from '@/stores/agent-store';
import { useConfigStore } from '@/stores/config-store';
import { useJobStore } from '@/stores/job-store';

function AppContent() {
  const location = useLocation();
  const navigate = useNavigate();

  // Zustand stores
  const { sessions, activeSessionId, loadSessions, selectSession, startNewChat } = useSessionStore();
  const { clearMessages } = useChatStore();
  const { resetTokenUsage } = useAgentStore();
  const { ragEnabled, isLoaded, loadConfig } = useConfigStore();
  const { selectedJob, setSelectedJob } = useJobStore();

  // State for right sidebar (files panel)
  const [showFilesPanel, setShowFilesPanel] = useState(false);
  const [showRagPanel, setShowRagPanel] = useState(false);

  // Derive selected agent from URL instead of storing in state
  const selectedAgentId = location.pathname.startsWith('/agents/')
    ? location.pathname.split('/')[2]
    : null;

  // Derive selected job from URL and sync with store
  const urlJobId = location.pathname.startsWith('/jobs/')
    ? parseInt(location.pathname.split('/')[2], 10)
    : null;

  // Sync job store with URL
  useEffect(() => {
    if (urlJobId && !isNaN(urlJobId) && urlJobId !== selectedJob) {
      setSelectedJob(urlJobId);
    }
  }, [urlJobId, selectedJob, setSelectedJob]);

  useEffect(() => {
    void loadSessions();
  }, [loadSessions]);

  useEffect(() => {
    void loadConfig();
  }, [loadConfig]);

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

  const handleAgentSelect = (agentId: string) => {
    navigate(`/agents/${agentId}`);
  };

  const handleJobSelect = (jobId: number) => {
    setSelectedJob(jobId);
    navigate(`/jobs/${jobId}`);
  };

  const isLogsPage = location.pathname === '/logs';
  const isAgentStatsPage = location.pathname === '/agent-stats';
  const isAgentViewerPage = location.pathname.startsWith('/agents/');
  const isJobDetailsPage = location.pathname.startsWith('/jobs/');

  return (
    <SidebarProvider className="h-full">
      {(!isLogsPage && !isAgentStatsPage) || isAgentViewerPage || isJobDetailsPage ? (
        <AppSidebar
          sessions={sessions}
          onSessionSelect={handleSessionSelect}
          onNewChat={handleNewChat}
          activeSessionId={activeSessionId || undefined}
          onAgentSelect={handleAgentSelect}
          selectedAgentId={selectedAgentId || undefined}
          onJobSelect={handleJobSelect}
          selectedJobId={selectedJob || undefined}
        />
      ) : null}
      <SidebarInset className="flex flex-col overflow-hidden">
        <header className="flex h-16 shrink-0 items-center gap-2 border-b">
          <div className="flex flex-1 items-center gap-2 px-4">
            {!isLogsPage && !isAgentStatsPage && !isAgentViewerPage && !isJobDetailsPage && (
              <>
                <SidebarTrigger className="-ml-1" />
                <Separator orientation="vertical" className="mr-2 h-4" />
              </>
            )}
            {(isLogsPage || isAgentStatsPage || isAgentViewerPage || isJobDetailsPage) && (
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
              ) : null}
              {isAgentStatsPage ? (
                <Button variant="ghost" className="flex items-center gap-2" onClick={() => navigate('/')}>
                  <MessageSquare className="h-4 w-4" />
                  Back to Chat
                </Button>
              ) : null}
              {isAgentViewerPage ? (
                <Button variant="ghost" className="flex items-center gap-2" onClick={() => navigate('/')}>
                  <MessageSquare className="h-4 w-4" />
                  Back to Chat
                </Button>
              ) : null}
              {isJobDetailsPage ? (
                <Button variant="ghost" className="flex items-center gap-2" onClick={() => navigate('/')}>
                  <MessageSquare className="h-4 w-4" />
                  Back to Chat
                </Button>
              ) : null}
            </div>
          </div>
          {!isLogsPage && !isAgentStatsPage && !isAgentViewerPage && !isJobDetailsPage && (
            <>
              <Separator orientation="vertical" className="h-4" />
              {isLoaded && ragEnabled && (
                <Button
                  variant="ghost"
                  size="icon"
                  className="mr-2"
                  onClick={() => {
                    setShowRagPanel(!showRagPanel);
                    if (!showRagPanel) setShowFilesPanel(false);
                  }}
                  title="Toggle RAG documents"
                >
                  <Database className={showRagPanel ? 'text-primary' : ''} />
                </Button>
              )}
              <Button
                variant="ghost"
                size="icon"
                className="mr-4"
                onClick={() => {
                  setShowFilesPanel(!showFilesPanel);
                  if (!showFilesPanel) setShowRagPanel(false);
                }}
                title="Toggle workspace files"
              >
                <Files className={showFilesPanel ? 'text-primary' : ''} />
              </Button>
            </>
          )}
        </header>
        <div className="flex flex-1 overflow-hidden min-h-0">
          <div className="flex flex-1 flex-col overflow-hidden p-4">
            <Routes>
              <Route path="/" element={<ChatBot />} />
              <Route path="/logs" element={<Logs />} />
              <Route path="/agent-stats" element={<AgentStatsCard apiUrl="" />} />
              <Route path="/agents/:id" element={<AgentViewer />} />
              <Route path="/jobs/:id" element={<JobDetails />} />
              <Route path="/workspace/files/*" element={<FileViewer />} />
            </Routes>
          </div>
          {!isLogsPage && !isAgentViewerPage && !isJobDetailsPage && isLoaded && ragEnabled && showRagPanel && (
            <div className="border-l w-96 shrink-0 overflow-auto p-4">
              <DocumentManager />
            </div>
          )}
          {!isLogsPage && !isAgentViewerPage && !isJobDetailsPage && showFilesPanel && (
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
