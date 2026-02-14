import { create } from 'zustand';
import { listSessions, deleteSession as apiDeleteSession, updateSessionTitle as apiUpdateSessionTitle, type SessionListItem } from '@/lib/chat-api';
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

  // Actions
  loadSessions: () => Promise<void>;
  setActiveSession: (sessionId: string | null) => void;
  selectSession: (sessionId: string) => void;
  startNewChat: () => void;
  refreshSessions: () => Promise<void>;
  deleteSession: (sessionId: string) => Promise<boolean>;
  updateSessionTitle: (sessionId: string, title: string) => Promise<boolean>;
}

export const useSessionStore = create<SessionStore>((set, get) => ({
  // Initial state
  sessions: [],
  activeSessionId: null,
  isLoading: false,

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
        // Update local state
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
}));
