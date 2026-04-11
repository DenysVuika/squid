import { BrowserRouter, Routes, Route, useLocation, useNavigate } from 'react-router-dom';
import ChatBot from './components/app/chatbot';
import Logs from './components/app/logs';
import { FileViewer } from './components/app/file-viewer';
import { AgentViewer } from './components/app/agent-viewer';
import JobDetails from './components/app/job-details';
import { LandingPage } from './components/app/landing-page';
import { AppSidebar } from './components/app/app-sidebar';
import { FilesSidebar } from './components/app/files-sidebar';
import { DocumentManager } from './components/app/document-manager';
import { AgentStatsCard } from './components/app/agent-stats';
import { JobCreateDialog } from './components/app/job-create-dialog';
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar';
import { Separator } from '@/components/ui/separator';
import { Button } from './components/ui/button';
import { Files, Database, Plus, Briefcase } from 'lucide-react';
import { useEffect, useState, useRef, useCallback } from 'react';
import { useSessionStore } from '@/stores/session-store';
import { useChatStore } from '@/stores/chat-store';
import { useAgentStore } from '@/stores/agent-store';
import { useConfigStore } from '@/stores/config-store';
import { useJobStore } from '@/stores/job-store';

function AppContent() {
  const location = useLocation();
  const navigate = useNavigate();

  // Zustand stores
  const { activeSessionId: storeActiveSessionId, loadSessions, selectSession, startNewChat, startSSE: startSessionSSE, stopSSE: stopSessionSSE } = useSessionStore();
  const { clearMessages } = useChatStore();
  const { agents, resetTokenUsage } = useAgentStore();
  const { ragEnabled, isLoaded, loadConfig } = useConfigStore();
  const { selectedJob, setSelectedJob, loadJobs, startSSE: startJobSSE, stopSSE: stopJobSSE } = useJobStore();

  // Derive active session from URL (null for /new route)
  const activeSessionId = location.pathname.startsWith('/chat/')
    ? location.pathname.split('/')[2]
    : null;

  const isNewChatRoute = location.pathname === '/new';

  // Track previous session to detect new session creation vs navigation
  const prevSessionRef = useRef<string | null>(storeActiveSessionId);

  // State for right sidebar (files panel)
  const [showFilesPanel, setShowFilesPanel] = useState(false);
  const [showRagPanel, setShowRagPanel] = useState(false);

  // State for job creation dialog
  const [showJobCreateDialog, setShowJobCreateDialog] = useState(false);

  // Derive selected agent from URL instead of storing in state
  const selectedAgentId = location.pathname.startsWith('/agents/')
    ? location.pathname.split('/')[2]
    : null;

  // Derive selected job from URL and sync with store
  const urlJobId = location.pathname.startsWith('/jobs/')
    ? parseInt(location.pathname.split('/')[2], 10)
    : null;

  // Sync session store when URL session changes
  useEffect(() => {
    if (activeSessionId && activeSessionId !== storeActiveSessionId) {
      // URL has a session, sync it to store
      selectSession(activeSessionId);
      prevSessionRef.current = activeSessionId;
    } else if (isNewChatRoute) {
      // On /new route - always ensure store is cleared
      if (storeActiveSessionId) {
        startNewChat();
      }
      prevSessionRef.current = null;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeSessionId, isNewChatRoute]);

  // Navigate to session URL when a new session is created on /new route
  useEffect(() => {
    const prevSession = prevSessionRef.current;

    if (isNewChatRoute && storeActiveSessionId && prevSession === null) {
      // A new session was just created while on /new route (prev was null, now has value)
      navigate(`/chat/${storeActiveSessionId}`, { replace: true });
      prevSessionRef.current = storeActiveSessionId;
    }
  }, [isNewChatRoute, storeActiveSessionId, navigate]);

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

  // Start SSE connections for real-time updates at app level
  // This ensures connections stay alive regardless of navigation/sidebar visibility
  useEffect(() => {
    // Start job updates SSE
    void loadJobs();
    startJobSSE();

    // Start session updates SSE
    startSessionSSE();

    return () => {
      stopJobSSE();
      stopSessionSSE();
    };
  }, [loadJobs, startJobSSE, stopJobSSE, startSessionSSE, stopSessionSSE]);

  const handleSessionSelect = (sessionId: string) => {
    // Navigate to the session URL
    navigate(`/chat/${sessionId}`);
  };

  const handleNewChat = useCallback(() => {
    startNewChat();
    clearMessages();
    resetTokenUsage();
    navigate('/new');
  }, [startNewChat, clearMessages, resetTokenUsage, navigate]);

  // Global keyboard shortcut: Cmd+K / Ctrl+K to start new chat (works everywhere)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        handleNewChat();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleNewChat]);

  const handleAgentSelect = (agentId: string) => {
    navigate(`/agents/${agentId}`);
  };

  const handleJobSelect = (jobId: number) => {
    setSelectedJob(jobId);
    navigate(`/jobs/${jobId}`);
  };

  // No longer needed - unified layout across all pages

  return (
    <SidebarProvider className="h-full">
      {/* Sidebar is always visible - unified layout across all pages */}
      <AppSidebar
        onSessionSelect={handleSessionSelect}
        activeSessionId={activeSessionId || undefined}
        onAgentSelect={handleAgentSelect}
        selectedAgentId={selectedAgentId || undefined}
        onJobSelect={handleJobSelect}
        selectedJobId={selectedJob || undefined}
      />
      <SidebarInset className="flex flex-col overflow-hidden">
        <header className="flex h-16 shrink-0 items-center gap-2 border-b">
          <div className="flex flex-1 items-center gap-2 px-4">
            {/* Sidebar trigger - always visible for unified layout */}
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 h-4" />

            <div className="flex gap-2">
              {/* Action buttons - available on all pages */}
              <Button
                variant="outline"
                size="sm"
                className="flex items-center gap-2"
                onClick={handleNewChat}
              >
                <Plus className="h-4 w-4" />
                New Chat
              </Button>
              <Button
                variant="outline"
                size="sm"
                className="flex items-center gap-2"
                onClick={() => setShowJobCreateDialog(true)}
              >
                <Briefcase className="h-4 w-4" />
                New Job
              </Button>
            </div>
          </div>
          {/* Right panel toggles - available on all pages */}
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
        </header>
        <div className="flex flex-1 overflow-hidden min-h-0">
          <div className="flex flex-1 flex-col overflow-hidden p-4">
            <Routes>
              <Route path="/" element={<LandingPage />} />
              <Route path="/new" element={<ChatBot key="new-chat" />} />
              <Route path="/chat/:id" element={<ChatBot key={activeSessionId || 'loading'} />} />
              <Route path="/logs" element={<Logs />} />
              <Route path="/agent-stats" element={<AgentStatsCard apiUrl="" />} />
              <Route path="/agents/:id" element={<AgentViewer />} />
              <Route path="/jobs/:id" element={<JobDetails />} />
              <Route path="/workspace/files/*" element={<FileViewer />} />
            </Routes>
          </div>
          {/* Right panels - available on all pages */}
          {isLoaded && ragEnabled && showRagPanel && (
            <div className="border-l w-96 shrink-0 overflow-auto p-4">
              <DocumentManager />
            </div>
          )}
          {showFilesPanel && (
            <div className="border-l w-80 shrink-0">
              <FilesSidebar />
            </div>
          )}
        </div>
      </SidebarInset>

      {/* Job Creation Dialog */}
      <JobCreateDialog
        open={showJobCreateDialog}
        onOpenChange={setShowJobCreateDialog}
        agents={agents}
        onJobCreated={() => {
          // Reload jobs list after creation
          void loadJobs();
        }}
      />
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
