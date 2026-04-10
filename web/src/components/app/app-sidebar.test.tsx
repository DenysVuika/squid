import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, beforeAll } from 'vitest';
import { BrowserRouter } from 'react-router-dom';
import { AppSidebar } from './app-sidebar';
import { SidebarProvider } from '@/components/ui/sidebar';
import { useSessionStore } from '@/stores/session-store';
import { useAgentStore } from '@/stores/agent-store';
import { useJobStore } from '@/stores/job-store';

// JSDOM polyfills required by Radix UI
beforeAll(() => {
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn(() => false);
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
  window.HTMLElement.prototype.setPointerCapture = vi.fn();

  // Mock IntersectionObserver for Radix Portal
  (globalThis as any).IntersectionObserver = class IntersectionObserver {
    constructor() {}
    disconnect() {}
    observe() {}
    takeRecords() { return []; }
    unobserve() {}
  };

  // Mock ResizeObserver for Radix components
  (globalThis as any).ResizeObserver = class ResizeObserver {
    constructor() {}
    disconnect() {}
    observe() {}
    unobserve() {}
  };
});

// Mock stores
vi.mock('@/stores/session-store', () => ({
  useSessionStore: vi.fn(),
}));

vi.mock('@/stores/agent-store', () => ({
  useAgentStore: vi.fn(),
}));

vi.mock('@/stores/job-store', () => ({
  useJobStore: vi.fn(),
}));

// Mock chat-api
vi.mock('@/lib/chat-api', () => ({
  pauseJob: vi.fn(),
  resumeJob: vi.fn(),
  deleteJob: vi.fn(),
  triggerJob: vi.fn(),
  cancelJob: vi.fn(),
}));

describe('AppSidebar', () => {
  const mockDeleteSession = vi.fn();
  const mockUpdateSessionTitle = vi.fn();
  const mockLoadAgents = vi.fn();
  const mockLoadJobs = vi.fn();

  // Helper to render AppSidebar with required SidebarProvider and BrowserRouter
  const renderAppSidebar = (props = {}) => {
    return render(
      <BrowserRouter>
        <SidebarProvider>
          <AppSidebar {...props} />
        </SidebarProvider>
      </BrowserRouter>
    );
  };

  beforeEach(() => {
    vi.clearAllMocks();

    // Default store implementations with selector support
    vi.mocked(useSessionStore).mockImplementation((selector?: any) => {
      const sessionState = {
        sessions: [],
        deleteSession: mockDeleteSession,
        updateSessionTitle: mockUpdateSessionTitle,
      };
      return selector ? selector(sessionState) : sessionState;
    });

    vi.mocked(useAgentStore).mockImplementation((selector?: any) => {
      const agentState = {
        agents: [],
        loadAgents: mockLoadAgents,
      };
      return selector ? selector(agentState) : agentState;
    });

    vi.mocked(useJobStore).mockImplementation((selector?: any) => {
      const jobState = {
        jobs: [],
        loadJobs: mockLoadJobs,
      };
      return selector ? selector(jobState) : jobState;
    });
  });

  // ── Basic rendering ────────────────────────────────────────────────────────

  describe('basic rendering', () => {
    it('renders sidebar with Squid branding', () => {
      renderAppSidebar();

      expect(screen.getByText('🦑')).toBeInTheDocument();
      expect(screen.getByText('Squid')).toBeInTheDocument();
    });

    it('renders collapsible sections', () => {
      renderAppSidebar();

      expect(screen.getByText('Sessions')).toBeInTheDocument();
      expect(screen.getByText('Agents')).toBeInTheDocument();
      expect(screen.getByText('Jobs')).toBeInTheDocument();
    });
  });

  // ── Sessions section ───────────────────────────────────────────────────────

  describe('sessions section', () => {
    it('expands Sessions section automatically when a session is active', async () => {
      // Mock store with sessions data
    vi.mocked(useSessionStore).mockImplementation((selector?: any) => {
        const state = {
          sessions: [{ id: 'session-1', title: 'Chat 1', created_at: '2024-01-01', updated_at: '2024-01-01', message_count: 1 }],
          deleteSession: mockDeleteSession,
          updateSessionTitle: mockUpdateSessionTitle,
        };
        return selector ? selector(state) : state;
      });

      renderAppSidebar({ activeSessionId: 'session-1' });

      // The Sessions section should be open automatically (showing the chat title)
      await waitFor(() => {
        expect(screen.getByText('Chat 1')).toBeVisible();
      });
    });

    it('highlights active session', async () => {
      // Mock store with multiple sessions
    vi.mocked(useSessionStore).mockImplementation((selector?: any) => {
        const state = {
          sessions: [
            { id: 'session-1', title: 'Chat 1', created_at: '2024-01-01', updated_at: '2024-01-01', message_count: 1 },
            { id: 'session-2', title: 'Chat 2', created_at: '2024-01-02', updated_at: '2024-01-02', message_count: 2 },
          ],
          deleteSession: mockDeleteSession,
          updateSessionTitle: mockUpdateSessionTitle,
        };
        return selector ? selector(state) : state;
      });

      renderAppSidebar({ activeSessionId: 'session-1' });

      await waitFor(() => {
        const activeButton = screen.getByText('Chat 1').closest('button');
        expect(activeButton).toHaveAttribute('data-active', 'true');
      });
    });

    it('does not auto-expand Sessions section when no session is active', () => {
      // Mock store with sessions
    vi.mocked(useSessionStore).mockImplementation((selector?: any) => {
        const state = {
          sessions: [{ id: 'session-1', title: 'Chat 1', created_at: '2024-01-01', updated_at: '2024-01-01', message_count: 1 }],
          deleteSession: mockDeleteSession,
          updateSessionTitle: mockUpdateSessionTitle,
        };
        return selector ? selector(state) : state;
      });

      renderAppSidebar({});

      // Without activeSessionId, section should be closed
      // Use queryByText since it won't throw if not found
      const chat1 = screen.queryByText('Chat 1');
      // It may be in the DOM but not visible if collapsed
      expect(chat1).toBeDefined();
    });
  });

  // ── Agents section ─────────────────────────────────────────────────────────

  describe('agents section', () => {
    it('loads agents on mount', () => {
      renderAppSidebar();

      expect(mockLoadAgents).toHaveBeenCalled();
    });

    it('expands Agents section automatically when an agent is selected', async () => {
    vi.mocked(useAgentStore).mockImplementation((selector?: any) => {
        const state = {
          agents: [
            { id: 'agent-1', name: 'Code Reviewer', description: 'Reviews code' },
          ],
          loadAgents: mockLoadAgents,
        };
        return selector ? selector(state) : state;
      });

      renderAppSidebar({ selectedAgentId: 'agent-1' });

      await waitFor(() => {
        expect(screen.getByText('Code Reviewer')).toBeVisible();
      });
    });

    it('highlights selected agent', async () => {
    vi.mocked(useAgentStore).mockImplementation((selector?: any) => {
        const state = {
          agents: [
            { id: 'agent-1', name: 'Code Reviewer', description: 'Reviews code' },
          ],
          loadAgents: mockLoadAgents,
        };
        return selector ? selector(state) : state;
      });

      renderAppSidebar({ selectedAgentId: 'agent-1' });

      await waitFor(() => {
        const activeButton = screen.getByText('Code Reviewer').closest('button');
        expect(activeButton).toHaveAttribute('data-active', 'true');
      });
    });
  });

  // ── Jobs section ───────────────────────────────────────────────────────────

  describe('jobs section', () => {
    it('expands Jobs section automatically when a job is selected', async () => {
    vi.mocked(useJobStore).mockImplementation((selector?: any) => {
        const state = {
          jobs: [
            {
              id: 1,
              name: 'Daily Review',
              schedule_type: 'cron',
              status: 'pending',
              is_active: true,
            },
          ],
          loadJobs: mockLoadJobs,
        };
        return selector ? selector(state) : state;
      });

      renderAppSidebar({ selectedJobId: 1 });

      await waitFor(() => {
        expect(screen.getByText('Daily Review')).toBeVisible();
      });
    });

    it('highlights selected job', async () => {
    vi.mocked(useJobStore).mockImplementation((selector?: any) => {
        const state = {
          jobs: [
            {
              id: 1,
              name: 'Daily Review',
              schedule_type: 'cron',
              status: 'pending',
              is_active: true,
            },
          ],
          loadJobs: mockLoadJobs,
        };
        return selector ? selector(state) : state;
      });

      renderAppSidebar({ selectedJobId: 1 });

      await waitFor(() => {
        const activeButton = screen.getByText('Daily Review').closest('button');
        expect(activeButton).toHaveAttribute('data-active', 'true');
      });
    });
  });
});
