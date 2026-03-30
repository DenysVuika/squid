import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useSessionStore, type ChatSession } from './session-store';
import {
  listSessions,
  deleteSession as apiDeleteSession,
  updateSessionTitle as apiUpdateSessionTitle,
} from '@/lib/chat-api';
import type { SessionListItem } from '@/lib/chat-api';
import { toast } from 'sonner';

vi.mock('@/lib/chat-api', () => ({
  listSessions: vi.fn(),
  deleteSession: vi.fn(),
  updateSessionTitle: vi.fn(),
}));

vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// ─── Fixtures ────────────────────────────────────────────────────────────────

const CREATED_AT_MS = 1_700_000_000_000;
const UPDATED_AT_MS = 1_700_000_060_000;

const TOKEN_USAGE = {
  total_tokens: 0,
  input_tokens: 0,
  output_tokens: 0,
  reasoning_tokens: 0,
  cache_tokens: 0,
  context_window: 0,
  context_utilization: 0,
};

const makeApiSession = (overrides: Partial<SessionListItem> = {}): SessionListItem => ({
  session_id: 'session-1',
  message_count: 3,
  created_at: CREATED_AT_MS,
  updated_at: UPDATED_AT_MS,
  preview: 'Hello there',
  title: 'My Chat',
  agent_id: null,
  token_usage: TOKEN_USAGE,
  cost_usd: 0,
  ...overrides,
});

/** Two pre-built ChatSessions for seeding the store directly. */
const SESSION_A: ChatSession = {
  id: 'session-a',
  title: 'Session A',
  preview: 'Preview A',
  created_at: new Date(CREATED_AT_MS).toISOString(),
  updated_at: new Date(UPDATED_AT_MS).toISOString(),
  message_count: 2,
};

const SESSION_B: ChatSession = {
  id: 'session-b',
  title: 'Session B',
  preview: 'Preview B',
  created_at: new Date(CREATED_AT_MS).toISOString(),
  updated_at: new Date(UPDATED_AT_MS).toISOString(),
  message_count: 5,
};

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('useSessionStore', () => {
  beforeEach(() => {
    useSessionStore.setState({ sessions: [], activeSessionId: null, isLoading: false });
    localStorage.clear();
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  // ── Initial state ──────────────────────────────────────────────────────────

  describe('initial state', () => {
    it('starts with an empty sessions list', () => {
      expect(useSessionStore.getState().sessions).toEqual([]);
    });

    it('starts with no active session', () => {
      expect(useSessionStore.getState().activeSessionId).toBeNull();
    });

    it('starts not loading', () => {
      expect(useSessionStore.getState().isLoading).toBe(false);
    });
  });

  // ── loadSessions ───────────────────────────────────────────────────────────

  describe('loadSessions', () => {
    it('sets isLoading=true while the request is in-flight', () => {
      vi.mocked(listSessions).mockReturnValueOnce(new Promise(() => {}));

      useSessionStore.getState().loadSessions();

      expect(useSessionStore.getState().isLoading).toBe(true);
    });

    it('calls listSessions with an empty string', async () => {
      vi.mocked(listSessions).mockResolvedValueOnce({ sessions: [], total: 0 });

      await useSessionStore.getState().loadSessions();

      expect(vi.mocked(listSessions)).toHaveBeenCalledOnce();
      expect(vi.mocked(listSessions)).toHaveBeenCalledWith('');
    });

    it('clears isLoading after a successful fetch', async () => {
      vi.mocked(listSessions).mockResolvedValueOnce({ sessions: [], total: 0 });

      await useSessionStore.getState().loadSessions();

      expect(useSessionStore.getState().isLoading).toBe(false);
    });

    it('populates sessions from the API response', async () => {
      vi.mocked(listSessions).mockResolvedValueOnce({
        sessions: [makeApiSession(), makeApiSession({ session_id: 'session-2', title: 'Second' })],
        total: 2,
      });

      await useSessionStore.getState().loadSessions();

      expect(useSessionStore.getState().sessions).toHaveLength(2);
    });

    it('maps session_id → id', async () => {
      vi.mocked(listSessions).mockResolvedValueOnce({
        sessions: [makeApiSession({ session_id: 'abc-123' })],
        total: 1,
      });

      await useSessionStore.getState().loadSessions();

      expect(useSessionStore.getState().sessions[0].id).toBe('abc-123');
    });

    it('maps message_count correctly', async () => {
      vi.mocked(listSessions).mockResolvedValueOnce({
        sessions: [makeApiSession({ message_count: 42 })],
        total: 1,
      });

      await useSessionStore.getState().loadSessions();

      expect(useSessionStore.getState().sessions[0].message_count).toBe(42);
    });

    it('converts numeric created_at to an ISO string', async () => {
      vi.mocked(listSessions).mockResolvedValueOnce({
        sessions: [makeApiSession({ created_at: CREATED_AT_MS })],
        total: 1,
      });

      await useSessionStore.getState().loadSessions();

      expect(useSessionStore.getState().sessions[0].created_at).toBe(
        new Date(CREATED_AT_MS).toISOString(),
      );
    });

    it('converts numeric updated_at to an ISO string', async () => {
      vi.mocked(listSessions).mockResolvedValueOnce({
        sessions: [makeApiSession({ updated_at: UPDATED_AT_MS })],
        total: 1,
      });

      await useSessionStore.getState().loadSessions();

      expect(useSessionStore.getState().sessions[0].updated_at).toBe(
        new Date(UPDATED_AT_MS).toISOString(),
      );
    });

    describe('title fallback logic', () => {
      it('uses title when both title and preview are present', async () => {
        vi.mocked(listSessions).mockResolvedValueOnce({
          sessions: [makeApiSession({ title: 'My Title', preview: 'My Preview' })],
          total: 1,
        });

        await useSessionStore.getState().loadSessions();

        expect(useSessionStore.getState().sessions[0].title).toBe('My Title');
      });

      it('falls back to preview when title is null', async () => {
        vi.mocked(listSessions).mockResolvedValueOnce({
          sessions: [makeApiSession({ title: null, preview: 'Only Preview' })],
          total: 1,
        });

        await useSessionStore.getState().loadSessions();

        expect(useSessionStore.getState().sessions[0].title).toBe('Only Preview');
      });

      it('falls back to "New Chat" when both title and preview are null', async () => {
        vi.mocked(listSessions).mockResolvedValueOnce({
          sessions: [makeApiSession({ title: null, preview: null })],
          total: 1,
        });

        await useSessionStore.getState().loadSessions();

        expect(useSessionStore.getState().sessions[0].title).toBe('New Chat');
      });

      it('falls back to "New Chat" when title is null and preview is empty string', async () => {
        vi.mocked(listSessions).mockResolvedValueOnce({
          sessions: [makeApiSession({ title: null, preview: '' })],
          total: 1,
        });

        await useSessionStore.getState().loadSessions();

        expect(useSessionStore.getState().sessions[0].title).toBe('New Chat');
      });
    });

    describe('preview field mapping', () => {
      it('stores the preview string when present', async () => {
        vi.mocked(listSessions).mockResolvedValueOnce({
          sessions: [makeApiSession({ preview: 'Some preview text' })],
          total: 1,
        });

        await useSessionStore.getState().loadSessions();

        expect(useSessionStore.getState().sessions[0].preview).toBe('Some preview text');
      });

      it('stores undefined (not null) when preview is null', async () => {
        vi.mocked(listSessions).mockResolvedValueOnce({
          sessions: [makeApiSession({ preview: null })],
          total: 1,
        });

        await useSessionStore.getState().loadSessions();

        expect(useSessionStore.getState().sessions[0].preview).toBeUndefined();
      });
    });

    describe('error handling', () => {
      it('clears isLoading after a failed fetch', async () => {
        vi.mocked(listSessions).mockRejectedValueOnce(new Error('Network error'));

        await useSessionStore.getState().loadSessions();

        expect(useSessionStore.getState().isLoading).toBe(false);
      });

      it('keeps sessions unchanged on error', async () => {
        useSessionStore.setState({ sessions: [SESSION_A] });
        vi.mocked(listSessions).mockRejectedValueOnce(new Error('Network error'));

        await useSessionStore.getState().loadSessions();

        expect(useSessionStore.getState().sessions).toEqual([SESSION_A]);
      });

      it('shows a toast error on failure', async () => {
        vi.mocked(listSessions).mockRejectedValueOnce(new Error('Network error'));

        await useSessionStore.getState().loadSessions();

        expect(vi.mocked(toast.error)).toHaveBeenCalledWith('Failed to load sessions');
      });

      it('logs the error to the console', async () => {
        const err = new Error('Boom');
        vi.mocked(listSessions).mockRejectedValueOnce(err);
        const consoleSpy = vi.spyOn(console, 'error');

        await useSessionStore.getState().loadSessions();

        expect(consoleSpy).toHaveBeenCalledWith('Failed to load sessions:', err);
      });
    });
  });

  // ── setActiveSession ───────────────────────────────────────────────────────

  describe('setActiveSession', () => {
    it('updates activeSessionId in the store', () => {
      useSessionStore.getState().setActiveSession('session-x');

      expect(useSessionStore.getState().activeSessionId).toBe('session-x');
    });

    it('persists the session id to localStorage', () => {
      useSessionStore.getState().setActiveSession('session-x');

      expect(localStorage.getItem('squid_session_id')).toBe('session-x');
    });

    it('sets activeSessionId to null', () => {
      useSessionStore.setState({ activeSessionId: 'session-x' });

      useSessionStore.getState().setActiveSession(null);

      expect(useSessionStore.getState().activeSessionId).toBeNull();
    });

    it('removes the key from localStorage when called with null', () => {
      localStorage.setItem('squid_session_id', 'session-x');

      useSessionStore.getState().setActiveSession(null);

      expect(localStorage.getItem('squid_session_id')).toBeNull();
    });
  });

  // ── selectSession ──────────────────────────────────────────────────────────

  describe('selectSession', () => {
    it('updates activeSessionId in the store', () => {
      useSessionStore.getState().selectSession('session-y');

      expect(useSessionStore.getState().activeSessionId).toBe('session-y');
    });

    it('persists the session id to localStorage', () => {
      useSessionStore.getState().selectSession('session-y');

      expect(localStorage.getItem('squid_session_id')).toBe('session-y');
    });

    it('overwrites any previously stored session id', () => {
      useSessionStore.getState().selectSession('session-first');
      useSessionStore.getState().selectSession('session-second');

      expect(localStorage.getItem('squid_session_id')).toBe('session-second');
      expect(useSessionStore.getState().activeSessionId).toBe('session-second');
    });
  });

  // ── startNewChat ───────────────────────────────────────────────────────────

  describe('startNewChat', () => {
    it('clears activeSessionId', () => {
      useSessionStore.setState({ activeSessionId: 'session-z' });

      useSessionStore.getState().startNewChat();

      expect(useSessionStore.getState().activeSessionId).toBeNull();
    });

    it('removes the session id from localStorage', () => {
      localStorage.setItem('squid_session_id', 'session-z');

      useSessionStore.getState().startNewChat();

      expect(localStorage.getItem('squid_session_id')).toBeNull();
    });

    it('does not affect the sessions list', () => {
      useSessionStore.setState({ sessions: [SESSION_A, SESSION_B], activeSessionId: 'session-a' });

      useSessionStore.getState().startNewChat();

      expect(useSessionStore.getState().sessions).toHaveLength(2);
    });
  });

  // ── refreshSessions ────────────────────────────────────────────────────────

  describe('refreshSessions', () => {
    it('delegates to loadSessions and calls the API', async () => {
      vi.mocked(listSessions).mockResolvedValueOnce({ sessions: [], total: 0 });

      await useSessionStore.getState().refreshSessions();

      expect(vi.mocked(listSessions)).toHaveBeenCalledOnce();
      expect(vi.mocked(listSessions)).toHaveBeenCalledWith('');
    });

    it('updates sessions just like loadSessions would', async () => {
      vi.mocked(listSessions).mockResolvedValueOnce({
        sessions: [makeApiSession({ session_id: 'refreshed', title: 'Refreshed' })],
        total: 1,
      });

      await useSessionStore.getState().refreshSessions();

      expect(useSessionStore.getState().sessions[0].id).toBe('refreshed');
    });
  });

  // ── deleteSession ──────────────────────────────────────────────────────────

  describe('deleteSession', () => {
    beforeEach(() => {
      useSessionStore.setState({ sessions: [SESSION_A, SESSION_B], activeSessionId: null });
    });

    it('returns true when the API reports success', async () => {
      vi.mocked(apiDeleteSession).mockResolvedValueOnce(true);

      const result = await useSessionStore.getState().deleteSession('session-a');

      expect(result).toBe(true);
    });

    it('removes the deleted session from local state', async () => {
      vi.mocked(apiDeleteSession).mockResolvedValueOnce(true);

      await useSessionStore.getState().deleteSession('session-a');

      const ids = useSessionStore.getState().sessions.map((s) => s.id);
      expect(ids).not.toContain('session-a');
      expect(ids).toContain('session-b');
    });

    it('calls the API with the correct arguments', async () => {
      vi.mocked(apiDeleteSession).mockResolvedValueOnce(true);

      await useSessionStore.getState().deleteSession('session-a');

      expect(vi.mocked(apiDeleteSession)).toHaveBeenCalledWith('', 'session-a');
    });

    it('shows a success toast when deletion succeeds', async () => {
      vi.mocked(apiDeleteSession).mockResolvedValueOnce(true);

      await useSessionStore.getState().deleteSession('session-a');

      expect(vi.mocked(toast.success)).toHaveBeenCalledWith('Session deleted');
    });

    it('clears the active session when the active session is deleted', async () => {
      useSessionStore.setState({ activeSessionId: 'session-a' });
      localStorage.setItem('squid_session_id', 'session-a');
      vi.mocked(apiDeleteSession).mockResolvedValueOnce(true);

      await useSessionStore.getState().deleteSession('session-a');

      expect(useSessionStore.getState().activeSessionId).toBeNull();
      expect(localStorage.getItem('squid_session_id')).toBeNull();
    });

    it('does not clear the active session when a different session is deleted', async () => {
      useSessionStore.setState({ activeSessionId: 'session-b' });
      vi.mocked(apiDeleteSession).mockResolvedValueOnce(true);

      await useSessionStore.getState().deleteSession('session-a');

      expect(useSessionStore.getState().activeSessionId).toBe('session-b');
    });

    it('returns false when the API reports failure', async () => {
      vi.mocked(apiDeleteSession).mockResolvedValueOnce(false);

      const result = await useSessionStore.getState().deleteSession('session-a');

      expect(result).toBe(false);
    });

    it('does not modify sessions when the API reports failure', async () => {
      vi.mocked(apiDeleteSession).mockResolvedValueOnce(false);

      await useSessionStore.getState().deleteSession('session-a');

      expect(useSessionStore.getState().sessions).toHaveLength(2);
    });

    it('shows an error toast when the API reports failure', async () => {
      vi.mocked(apiDeleteSession).mockResolvedValueOnce(false);

      await useSessionStore.getState().deleteSession('session-a');

      expect(vi.mocked(toast.error)).toHaveBeenCalledWith('Failed to delete session');
    });

    it('returns false when the API throws', async () => {
      vi.mocked(apiDeleteSession).mockRejectedValueOnce(new Error('Network error'));

      const result = await useSessionStore.getState().deleteSession('session-a');

      expect(result).toBe(false);
    });

    it('shows an error toast when the API throws', async () => {
      vi.mocked(apiDeleteSession).mockRejectedValueOnce(new Error('Network error'));

      await useSessionStore.getState().deleteSession('session-a');

      expect(vi.mocked(toast.error)).toHaveBeenCalledWith('Failed to delete session');
    });

    it('logs the error to the console when the API throws', async () => {
      const err = new Error('Network error');
      vi.mocked(apiDeleteSession).mockRejectedValueOnce(err);
      const consoleSpy = vi.spyOn(console, 'error');

      await useSessionStore.getState().deleteSession('session-a');

      expect(consoleSpy).toHaveBeenCalledWith('Failed to delete session:', err);
    });

    it('does not modify sessions when the API throws', async () => {
      vi.mocked(apiDeleteSession).mockRejectedValueOnce(new Error('Network error'));

      await useSessionStore.getState().deleteSession('session-a');

      expect(useSessionStore.getState().sessions).toHaveLength(2);
    });
  });

  // ── updateSessionTitle ─────────────────────────────────────────────────────

  describe('updateSessionTitle', () => {
    beforeEach(() => {
      useSessionStore.setState({ sessions: [SESSION_A, SESSION_B] });
    });

    it('returns true when the API reports success', async () => {
      vi.mocked(apiUpdateSessionTitle).mockResolvedValueOnce(true);

      const result = await useSessionStore.getState().updateSessionTitle('session-a', 'New Title');

      expect(result).toBe(true);
    });

    it('updates the title of the matching session in local state', async () => {
      vi.mocked(apiUpdateSessionTitle).mockResolvedValueOnce(true);

      await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      const session = useSessionStore.getState().sessions.find((s) => s.id === 'session-a');
      expect(session?.title).toBe('Renamed A');
    });

    it('does not affect other sessions', async () => {
      vi.mocked(apiUpdateSessionTitle).mockResolvedValueOnce(true);

      await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      const sessionB = useSessionStore.getState().sessions.find((s) => s.id === 'session-b');
      expect(sessionB?.title).toBe(SESSION_B.title);
    });

    it('calls the API with the correct arguments', async () => {
      vi.mocked(apiUpdateSessionTitle).mockResolvedValueOnce(true);

      await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      expect(vi.mocked(apiUpdateSessionTitle)).toHaveBeenCalledWith('', 'session-a', 'Renamed A');
    });

    it('shows a success toast when renaming succeeds', async () => {
      vi.mocked(apiUpdateSessionTitle).mockResolvedValueOnce(true);

      await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      expect(vi.mocked(toast.success)).toHaveBeenCalledWith('Session renamed');
    });

    it('returns false when the API reports failure', async () => {
      vi.mocked(apiUpdateSessionTitle).mockResolvedValueOnce(false);

      const result = await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      expect(result).toBe(false);
    });

    it('does not modify sessions when the API reports failure', async () => {
      vi.mocked(apiUpdateSessionTitle).mockResolvedValueOnce(false);

      await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      const session = useSessionStore.getState().sessions.find((s) => s.id === 'session-a');
      expect(session?.title).toBe(SESSION_A.title);
    });

    it('shows an error toast when the API reports failure', async () => {
      vi.mocked(apiUpdateSessionTitle).mockResolvedValueOnce(false);

      await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      expect(vi.mocked(toast.error)).toHaveBeenCalledWith('Failed to rename session');
    });

    it('returns false when the API throws', async () => {
      vi.mocked(apiUpdateSessionTitle).mockRejectedValueOnce(new Error('Network error'));

      const result = await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      expect(result).toBe(false);
    });

    it('does not modify sessions when the API throws', async () => {
      vi.mocked(apiUpdateSessionTitle).mockRejectedValueOnce(new Error('Network error'));

      await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      const session = useSessionStore.getState().sessions.find((s) => s.id === 'session-a');
      expect(session?.title).toBe(SESSION_A.title);
    });

    it('shows an error toast when the API throws', async () => {
      vi.mocked(apiUpdateSessionTitle).mockRejectedValueOnce(new Error('Network error'));

      await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      expect(vi.mocked(toast.error)).toHaveBeenCalledWith('Failed to rename session');
    });

    it('logs the error to the console when the API throws', async () => {
      const err = new Error('Network error');
      vi.mocked(apiUpdateSessionTitle).mockRejectedValueOnce(err);
      const consoleSpy = vi.spyOn(console, 'error');

      await useSessionStore.getState().updateSessionTitle('session-a', 'Renamed A');

      expect(consoleSpy).toHaveBeenCalledWith('Failed to update session title:', err);
    });
  });
});
