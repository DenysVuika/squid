import { create } from 'zustand';
import { listSessions, deleteSession as apiDeleteSession, updateSessionTitle as apiUpdateSessionTitle, subscribeToSessionUpdates, type SessionListItem } from '@/lib/chat-api';
import { toast } from 'sonner';

export interface ChatSession {
  id: string;
  title: string;
  preview?: string;
  created_at: string;
  updated_at: string;
  message_count: number;
}

interface SessionStore {
  // State
  sessions: ChatSession[];
  activeSessionId: string | null;
  isLoading: boolean;
  sseConnection: EventSource | null;

  // Actions
  loadSessions: () => Promise<void>;
  setActiveSession: (sessionId: string | null) => void;
  selectSession: (sessionId: string) => void;
  startNewChat: () => void;
  refreshSessions: () => Promise<void>;
  deleteSession: (sessionId: string) => Promise<boolean>;
  updateSessionTitle: (sessionId: string, title: string) => Promise<boolean>;
  updateSession: (session: SessionListItem) => void;
  removeSession: (sessionId: string) => void;
  startSSE: () => void;
  stopSSE: () => void;
}

export const useSessionStore = create<SessionStore>((set, get) => ({
  // Initial state
  sessions: [],
  activeSessionId: null,
  isLoading: false,
  sseConnection: null,

  // Load sessions from API
  loadSessions: async () => {
    set({ isLoading: true });
    try {
      const { sessions: fetchedSessions } = await listSessions('');
      
      const chatSessions: ChatSession[] = fetchedSessions.map((session: SessionListItem) => ({
        id: session.session_id,
        title: session.title || session.preview || 'New Chat',
        preview: session.preview || undefined,
        created_at: new Date(session.created_at).toISOString(),
        updated_at: new Date(session.updated_at).toISOString(),
        message_count: session.message_count,
      }));

      set({ sessions: chatSessions, isLoading: false });
    } catch (error) {
      console.error('Failed to load sessions:', error);
      toast.error('Failed to load sessions');
      set({ isLoading: false });
    }
  },

  // Set active session without loading
  setActiveSession: (sessionId: string | null) => {
    set({ activeSessionId: sessionId });
    if (sessionId) {
      localStorage.setItem('squid_session_id', sessionId);
    } else {
      localStorage.removeItem('squid_session_id');
    }
  },

  // Select and load a session
  selectSession: (sessionId: string) => {
    set({ activeSessionId: sessionId });
    localStorage.setItem('squid_session_id', sessionId);
  },

  // Start a new chat
  startNewChat: () => {
    set({ activeSessionId: null });
    localStorage.removeItem('squid_session_id');
  },

  // Refresh sessions (alias for loadSessions)
  refreshSessions: async () => {
    await get().loadSessions();
  },

  // Delete a session
  deleteSession: async (sessionId: string) => {
    try {
      const success = await apiDeleteSession('', sessionId);
      if (success) {
        // Remove from local state
        set((state) => ({
          sessions: state.sessions.filter((s) => s.id !== sessionId),
        }));
        
        // If deleted session was active, clear active session
        if (get().activeSessionId === sessionId) {
          get().startNewChat();
        }
        
        toast.success('Session deleted');
        return true;
      } else {
        toast.error('Failed to delete session');
        return false;
      }
    } catch (error) {
      console.error('Failed to delete session:', error);
      toast.error('Failed to delete session');
      return false;
    }
  },

  // Update session title
  updateSessionTitle: async (sessionId: string, title: string) => {
    try {
      const success = await apiUpdateSessionTitle('', sessionId, title);
      if (success) {
        // SSE will handle the update, but we update optimistically
        set((state) => ({
          sessions: state.sessions.map((s) =>
            s.id === sessionId ? { ...s, title } : s
          ),
        }));

        toast.success('Session renamed');
        return true;
      } else {
        toast.error('Failed to rename session');
        return false;
      }
    } catch (error) {
      console.error('Failed to update session title:', error);
      toast.error('Failed to rename session');
      return false;
    }
  },

  // Update a session in the store (from SSE)
  updateSession: (updatedSession: SessionListItem) => {
    set((state) => {
      const existingIndex = state.sessions.findIndex((s) => s.id === updatedSession.session_id);
      const chatSession: ChatSession = {
        id: updatedSession.session_id,
        title: updatedSession.title || updatedSession.preview || 'New Chat',
        preview: updatedSession.preview || undefined,
        created_at: new Date(updatedSession.created_at).toISOString(),
        updated_at: new Date(updatedSession.updated_at).toISOString(),
        message_count: updatedSession.message_count,
      };

      if (existingIndex >= 0) {
        // Update existing session
        const newSessions = [...state.sessions];
        newSessions[existingIndex] = chatSession;
        return { sessions: newSessions };
      } else {
        // Add new session
        return { sessions: [chatSession, ...state.sessions] };
      }
    });
  },

  // Remove a session from the store (from SSE)
  removeSession: (sessionId: string) => {
    set((state) => ({
      sessions: state.sessions.filter((s) => s.id !== sessionId),
      activeSessionId: state.activeSessionId === sessionId ? null : state.activeSessionId,
    }));
  },

  // Start SSE connection for live updates
  startSSE: () => {
    const { sseConnection, updateSession, removeSession } = get();

    // Don't create duplicate connections
    if (sseConnection) {
      console.log('Session SSE connection already active');
      return;
    }

    console.log('Starting SSE connection for sessions...');
    const eventSource = subscribeToSessionUpdates('', {
      onSessionUpdate: (session) => {
        updateSession(session);
      },
      onSessionDeleted: (sessionId) => {
        removeSession(sessionId);
      },
      onError: (error) => {
        console.error('Session SSE error:', error);
        // Reconnect after 5 seconds
        setTimeout(() => {
          const store = get();
          if (store.sseConnection) {
            store.sseConnection.close();
            set({ sseConnection: null });
            store.startSSE();
          }
        }, 5000);
      },
    });

    set({ sseConnection: eventSource });
  },

  // Stop SSE connection
  stopSSE: () => {
    const { sseConnection } = get();

    if (sseConnection) {
      console.log('Stopping SSE connection for sessions');
      sseConnection.close();
      set({ sseConnection: null });
    }
  },
}));
