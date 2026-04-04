import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useAgentStore } from './agent-store';
import { fetchAgents, fetchConfig } from '@/lib/chat-api';
import type { AgentInfo, TokenUsage } from '@/lib/chat-api';
import { toast } from 'sonner';

vi.mock('@/lib/chat-api', () => ({
  fetchAgents: vi.fn(),
  fetchConfig: vi.fn(),
}));

vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// ─── Fixtures ────────────────────────────────────────────────────────────────

const ZERO_USAGE: TokenUsage = {
  total_tokens: 0,
  input_tokens: 0,
  output_tokens: 0,
  reasoning_tokens: 0,
  cache_tokens: 0,
  context_window: 0,
  context_utilization: 0,
};

const INITIAL_STATE = {
  agents: [],
  agentGroups: [],
  selectedAgent: '',
  sessionAgentId: null,
  tokenUsage: ZERO_USAGE,
  isLoading: false,
  agentSelectorOpen: false,
};

/** Minimal valid config response — loadAgents calls fetchConfig in parallel but does not use the result. */
const STUB_CONFIG = {
  api_url: '',
  context_window: 0,
  rag_enabled: false,
  web_sounds: true,
};

const makeAgent = (overrides: Partial<AgentInfo> = {}): AgentInfo => ({
  id: 'agent-1',
  name: 'Agent One',
  description: 'A test agent',
  model: 'openai/gpt-4o',
  enabled: true,
  use_tools: false,
  permissions: { allow: [], deny: [] },
  ...overrides,
});

/**
 * Mocks the two API calls that loadAgents fires in parallel.
 * Only the fetchAgents result is used by the store logic.
 */
const mockLoad = (agents: AgentInfo[] = [makeAgent()], defaultAgent = 'agent-1') => {
  vi.mocked(fetchAgents).mockResolvedValueOnce({ agents, default_agent: defaultAgent });
  vi.mocked(fetchConfig).mockResolvedValueOnce(STUB_CONFIG);
};

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('useAgentStore', () => {
  beforeEach(() => {
    // Merge initial values back in — preserves action functions
    useAgentStore.setState(INITIAL_STATE);
    localStorage.clear();
    vi.clearAllMocks();
    // Suppress all console output; individual tests re-spy when needed
    vi.spyOn(console, 'log').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  // ── Initial state ──────────────────────────────────────────────────────────

  describe('initial state', () => {
    it('has an empty agents list', () => {
      expect(useAgentStore.getState().agents).toEqual([]);
    });

    it('has no agent groups', () => {
      expect(useAgentStore.getState().agentGroups).toEqual([]);
    });

    it('has no selected agent', () => {
      expect(useAgentStore.getState().selectedAgent).toBe('');
    });

    it('has no session agent id', () => {
      expect(useAgentStore.getState().sessionAgentId).toBeNull();
    });

    it('has zeroed token usage', () => {
      expect(useAgentStore.getState().tokenUsage).toEqual(ZERO_USAGE);
    });

    it('is not loading', () => {
      expect(useAgentStore.getState().isLoading).toBe(false);
    });

    it('has the agent selector closed', () => {
      expect(useAgentStore.getState().agentSelectorOpen).toBe(false);
    });
  });

  // ── loadAgents ─────────────────────────────────────────────────────────────

  describe('loadAgents', () => {
    it('sets isLoading=true while both requests are in-flight', () => {
      vi.mocked(fetchAgents).mockReturnValueOnce(new Promise(() => {}));
      vi.mocked(fetchConfig).mockReturnValueOnce(new Promise(() => {}));

      useAgentStore.getState().loadAgents();

      expect(useAgentStore.getState().isLoading).toBe(true);
    });

    it('calls fetchAgents and fetchConfig each with an empty string', async () => {
      mockLoad();

      await useAgentStore.getState().loadAgents();

      expect(vi.mocked(fetchAgents)).toHaveBeenCalledWith('');
      expect(vi.mocked(fetchConfig)).toHaveBeenCalledWith('');
    });

    // ── Success with agents ──────────────────────────────────────────────────

    describe('with agents returned', () => {
      it('populates the agents array', async () => {
        const agents = [makeAgent({ id: 'a' }), makeAgent({ id: 'b' })];
        mockLoad(agents, 'a');

        await useAgentStore.getState().loadAgents();

        expect(useAgentStore.getState().agents).toEqual(agents);
      });

      it('clears isLoading after a successful fetch', async () => {
        mockLoad();

        await useAgentStore.getState().loadAgents();

        expect(useAgentStore.getState().isLoading).toBe(false);
      });

      // ── Provider extraction ────────────────────────────────────────────────

      describe('provider extraction', () => {
        it('extracts the provider prefix from a "provider/model" string', async () => {
          mockLoad([makeAgent({ model: 'anthropic/claude-3' })], 'agent-1');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().agentGroups).toEqual(['anthropic']);
        });

        it('uses "local" for models without a slash', async () => {
          mockLoad([makeAgent({ model: 'gpt-4o' })], 'agent-1');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().agentGroups).toEqual(['local']);
        });

        it('deduplicates providers across multiple agents', async () => {
          const agents = [
            makeAgent({ id: 'a1', model: 'openai/gpt-4o' }),
            makeAgent({ id: 'a2', model: 'openai/gpt-3.5' }),
            makeAgent({ id: 'a3', model: 'anthropic/claude-3' }),
          ];
          mockLoad(agents, 'a1');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().agentGroups).toEqual(['anthropic', 'openai']);
        });

        it('sorts providers alphabetically', async () => {
          const agents = [
            makeAgent({ id: 'a1', model: 'zulu/model' }),
            makeAgent({ id: 'a2', model: 'alpha/model' }),
            makeAgent({ id: 'a3', model: 'mango/model' }),
          ];
          mockLoad(agents, 'a1');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().agentGroups).toEqual(['alpha', 'mango', 'zulu']);
        });

        it('mixes "local" with named providers and sorts them', async () => {
          const agents = [
            makeAgent({ id: 'a1', model: 'zulu/model' }),
            makeAgent({ id: 'a2', model: 'no-slash-model' }),
          ];
          mockLoad(agents, 'a1');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().agentGroups).toEqual(['local', 'zulu']);
        });
      });

      // ── Agent selection logic ──────────────────────────────────────────────

      describe('agent selection', () => {
        it('selects default_agent when no agent is currently selected', async () => {
          const agents = [makeAgent({ id: 'default' }), makeAgent({ id: 'other' })];
          mockLoad(agents, 'default');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().selectedAgent).toBe('default');
        });

        it('falls back to the first agent when default_agent is not in the list', async () => {
          const agents = [makeAgent({ id: 'first' }), makeAgent({ id: 'second' })];
          mockLoad(agents, 'missing');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().selectedAgent).toBe('first');
        });

        it('keeps a valid persisted selection instead of switching to the default', async () => {
          useAgentStore.setState({ selectedAgent: 'persisted' });
          const agents = [makeAgent({ id: 'persisted' }), makeAgent({ id: 'default' })];
          mockLoad(agents, 'default');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().selectedAgent).toBe('persisted');
        });

        it('replaces a stale persisted selection with default_agent', async () => {
          useAgentStore.setState({ selectedAgent: 'stale' });
          const agents = [makeAgent({ id: 'default' }), makeAgent({ id: 'other' })];
          mockLoad(agents, 'default');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().selectedAgent).toBe('default');
        });

        it('replaces a stale selection with the first agent when default_agent is also absent', async () => {
          useAgentStore.setState({ selectedAgent: 'stale' });
          const agents = [makeAgent({ id: 'first' })];
          mockLoad(agents, 'also-missing');

          await useAgentStore.getState().loadAgents();

          expect(useAgentStore.getState().selectedAgent).toBe('first');
        });
      });

      // ── Console logging ────────────────────────────────────────────────────

      describe('console logging', () => {
        it('logs "Default agent" when a new agent is auto-selected', async () => {
          const consoleSpy = vi.spyOn(console, 'log');
          mockLoad([makeAgent({ id: 'default', name: 'Default Bot' })], 'default');

          await useAgentStore.getState().loadAgents();

          expect(consoleSpy).toHaveBeenCalledWith(
            expect.stringContaining('Default agent: Default Bot'),
          );
        });

        it('logs "Restored persisted agent" when the current selection is valid and kept', async () => {
          useAgentStore.setState({ selectedAgent: 'persisted' });
          const consoleSpy = vi.spyOn(console, 'log');
          mockLoad(
            [makeAgent({ id: 'persisted' }), makeAgent({ id: 'default' })],
            'default',
          );

          await useAgentStore.getState().loadAgents();

          expect(consoleSpy).toHaveBeenCalledWith(
            expect.stringContaining('Restored persisted agent: persisted'),
          );
        });
      });
    });

    // ── No agents returned ───────────────────────────────────────────────────

    describe('with no agents returned', () => {
      beforeEach(() => {
        vi.mocked(fetchAgents).mockResolvedValueOnce({ agents: [], default_agent: '' });
        vi.mocked(fetchConfig).mockResolvedValueOnce(STUB_CONFIG);
      });

      it('clears isLoading', async () => {
        await useAgentStore.getState().loadAgents();

        expect(useAgentStore.getState().isLoading).toBe(false);
      });

      it('does not populate the agents array', async () => {
        await useAgentStore.getState().loadAgents();

        expect(useAgentStore.getState().agents).toEqual([]);
      });

      it('shows a "No agents available" error toast', async () => {
        await useAgentStore.getState().loadAgents();

        expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
          'No agents available',
          expect.objectContaining({
            description: expect.stringContaining('squid.config.json'),
          }),
        );
      });

      it('logs a warning to the console', async () => {
        const warnSpy = vi.spyOn(console, 'warn');

        await useAgentStore.getState().loadAgents();

        expect(warnSpy).toHaveBeenCalledWith(
          expect.stringContaining('No agents available'),
        );
      });
    });

    // ── API error ────────────────────────────────────────────────────────────

    describe('on API error', () => {
      it('clears isLoading', async () => {
        vi.mocked(fetchAgents).mockRejectedValueOnce(new Error('Network error'));
        vi.mocked(fetchConfig).mockResolvedValueOnce(STUB_CONFIG);

        await useAgentStore.getState().loadAgents();

        expect(useAgentStore.getState().isLoading).toBe(false);
      });

      it('shows a "Failed to connect to API" error toast', async () => {
        vi.mocked(fetchAgents).mockRejectedValueOnce(new Error('Network error'));
        vi.mocked(fetchConfig).mockResolvedValueOnce(STUB_CONFIG);

        await useAgentStore.getState().loadAgents();

        expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
          'Failed to connect to API',
          expect.objectContaining({
            description: expect.stringContaining('backend'),
          }),
        );
      });

      it('logs the error to the console', async () => {
        const err = new Error('Network error');
        vi.mocked(fetchAgents).mockRejectedValueOnce(err);
        vi.mocked(fetchConfig).mockResolvedValueOnce(STUB_CONFIG);
        const consoleSpy = vi.spyOn(console, 'error');

        await useAgentStore.getState().loadAgents();

        expect(consoleSpy).toHaveBeenCalledWith('Failed to load agents:', err);
      });

      it('does not change the agents array', async () => {
        useAgentStore.setState({ agents: [makeAgent()] });
        vi.mocked(fetchAgents).mockRejectedValueOnce(new Error('Network error'));
        vi.mocked(fetchConfig).mockResolvedValueOnce(STUB_CONFIG);

        await useAgentStore.getState().loadAgents();

        expect(useAgentStore.getState().agents).toHaveLength(1);
      });
    });
  });

  // ── setSelectedAgent ───────────────────────────────────────────────────────

  describe('setSelectedAgent', () => {
    it('updates selectedAgent', () => {
      useAgentStore.getState().setSelectedAgent('agent-x');

      expect(useAgentStore.getState().selectedAgent).toBe('agent-x');
    });

    it('always closes the agent selector', () => {
      useAgentStore.setState({ agentSelectorOpen: true });

      useAgentStore.getState().setSelectedAgent('agent-x');

      expect(useAgentStore.getState().agentSelectorOpen).toBe(false);
    });

    it('closes the selector even when the agent id is unknown', () => {
      useAgentStore.setState({ agentSelectorOpen: true, agents: [] });

      useAgentStore.getState().setSelectedAgent('nonexistent');

      expect(useAgentStore.getState().agentSelectorOpen).toBe(false);
    });

    it('logs the selected agent name and model when the agent exists', () => {
      useAgentStore.setState({
        agents: [makeAgent({ id: 'agent-x', name: 'My Agent', model: 'openai/gpt-4o' })],
      });
      const consoleSpy = vi.spyOn(console, 'log');

      useAgentStore.getState().setSelectedAgent('agent-x');

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('My Agent'),
      );
    });

    it('does not log when the agent id is unknown', () => {
      useAgentStore.setState({ agents: [] });
      const consoleSpy = vi.spyOn(console, 'log');

      useAgentStore.getState().setSelectedAgent('nonexistent');

      expect(consoleSpy).not.toHaveBeenCalled();
    });
  });

  // ── setSessionAgentId ──────────────────────────────────────────────────────

  describe('setSessionAgentId', () => {
    it('sets a non-null session agent id', () => {
      useAgentStore.getState().setSessionAgentId('session-agent-1');

      expect(useAgentStore.getState().sessionAgentId).toBe('session-agent-1');
    });

    it('clears the session agent id when called with null', () => {
      useAgentStore.setState({ sessionAgentId: 'session-agent-1' });

      useAgentStore.getState().setSessionAgentId(null);

      expect(useAgentStore.getState().sessionAgentId).toBeNull();
    });
  });

  // ── updateTokenUsage ───────────────────────────────────────────────────────

  describe('updateTokenUsage', () => {
    it('merges a partial update into the existing token usage', () => {
      useAgentStore.getState().updateTokenUsage({ total_tokens: 100, input_tokens: 60 });

      const { tokenUsage } = useAgentStore.getState();
      expect(tokenUsage.total_tokens).toBe(100);
      expect(tokenUsage.input_tokens).toBe(60);
    });

    it('does not overwrite fields absent from the partial update', () => {
      useAgentStore.setState({ tokenUsage: { ...ZERO_USAGE, output_tokens: 40 } });

      useAgentStore.getState().updateTokenUsage({ total_tokens: 100 });

      expect(useAgentStore.getState().tokenUsage.output_tokens).toBe(40);
    });

    it('reflects the most recent value after multiple calls to the same field', () => {
      useAgentStore.getState().updateTokenUsage({ total_tokens: 50 });
      useAgentStore.getState().updateTokenUsage({ total_tokens: 150 });

      expect(useAgentStore.getState().tokenUsage.total_tokens).toBe(150);
    });
  });

  // ── resetTokenUsage ────────────────────────────────────────────────────────

  describe('resetTokenUsage', () => {
    it('resets all token usage fields to zero', () => {
      useAgentStore.setState({
        tokenUsage: {
          total_tokens: 500,
          input_tokens: 300,
          output_tokens: 200,
          reasoning_tokens: 10,
          cache_tokens: 5,
          context_window: 8192,
          context_utilization: 0.61,
        },
      });

      useAgentStore.getState().resetTokenUsage();

      expect(useAgentStore.getState().tokenUsage).toEqual(ZERO_USAGE);
    });
  });

  // ── setAgentSelectorOpen ───────────────────────────────────────────────────

  describe('setAgentSelectorOpen', () => {
    it('opens the agent selector', () => {
      useAgentStore.getState().setAgentSelectorOpen(true);

      expect(useAgentStore.getState().agentSelectorOpen).toBe(true);
    });

    it('closes the agent selector', () => {
      useAgentStore.setState({ agentSelectorOpen: true });

      useAgentStore.getState().setAgentSelectorOpen(false);

      expect(useAgentStore.getState().agentSelectorOpen).toBe(false);
    });
  });

  // ── getAgentModelForPricing ────────────────────────────────────────────────

  describe('getAgentModelForPricing', () => {
    it('returns "gpt-4o" when there is no selected agent and no session agent', () => {
      useAgentStore.setState({ selectedAgent: '', sessionAgentId: null });

      expect(useAgentStore.getState().getAgentModelForPricing()).toBe('gpt-4o');
    });

    it('returns pricing_model when the agent has one', () => {
      useAgentStore.setState({
        agents: [makeAgent({ id: 'a', model: 'openai/gpt-4o', pricing_model: 'gpt-4o-2024-11-20' })],
        selectedAgent: 'a',
        sessionAgentId: null,
      });

      expect(useAgentStore.getState().getAgentModelForPricing()).toBe('gpt-4o-2024-11-20');
    });

    it('falls back to model when pricing_model is absent', () => {
      useAgentStore.setState({
        agents: [makeAgent({ id: 'a', model: 'anthropic/claude-3' })],
        selectedAgent: 'a',
        sessionAgentId: null,
      });

      expect(useAgentStore.getState().getAgentModelForPricing()).toBe('anthropic/claude-3');
    });

    it('falls back to the agent id when the agent is not in the list', () => {
      useAgentStore.setState({ agents: [], selectedAgent: 'unknown-agent', sessionAgentId: null });

      expect(useAgentStore.getState().getAgentModelForPricing()).toBe('unknown-agent');
    });

    it('prefers sessionAgentId over selectedAgent', () => {
      useAgentStore.setState({
        agents: [
          makeAgent({ id: 'session-agent', model: 'session/model' }),
          makeAgent({ id: 'selected-agent', model: 'selected/model' }),
        ],
        selectedAgent: 'selected-agent',
        sessionAgentId: 'session-agent',
      });

      expect(useAgentStore.getState().getAgentModelForPricing()).toBe('session/model');
    });

    it('uses selectedAgent when sessionAgentId is null', () => {
      useAgentStore.setState({
        agents: [makeAgent({ id: 'selected-agent', model: 'selected/model' })],
        selectedAgent: 'selected-agent',
        sessionAgentId: null,
      });

      expect(useAgentStore.getState().getAgentModelForPricing()).toBe('selected/model');
    });
  });
});
