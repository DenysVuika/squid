import { create } from 'zustand';
import { listSessions, type SessionListItem } from '@/lib/chat-api';
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
}));
