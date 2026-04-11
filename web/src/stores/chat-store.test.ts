import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useChatStore, type MessageType, type ToolApproval } from './chat-store';
import { streamChat, loadSession, sendToolApproval } from '@/lib/chat-api';
import type { StreamHandlers, TokenUsage, AgentInfo, SessionData } from '@/lib/chat-api';
import { toast } from 'sonner';
import { useSessionStore } from './session-store';
import { useAgentStore } from './agent-store';

vi.mock('@/lib/chat-api', () => ({
  streamChat: vi.fn(),
  loadSession: vi.fn(),
  sendToolApproval: vi.fn(),
}));

vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
  },
}));

vi.mock('./session-store', () => ({
  useSessionStore: { getState: vi.fn() },
}));

vi.mock('./agent-store', () => ({
  useAgentStore: { getState: vi.fn() },
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
  messages: [] as MessageType[],
  status: 'ready' as const,
  streamingMessageId: null,
  streamingContentRef: '',
  streamingReasoningRef: '',
  isReasoningStreaming: false,
  abortController: null,
  useWebSearch: false,
  useRag: false,
  useTools: false,
  pendingApprovals: new Map<string, ToolApproval>(),
  toolApprovalDecisions: new Map<string, { approval_id: string; approved: boolean; timestamp: number }>(),
};

const MSG_ID = 'assistant-test-id';

const makeMessage = (id = MSG_ID, content = '', overrides: Partial<MessageType> = {}): MessageType => ({
  from: 'assistant',
  key: `key-${id}`,
  versions: [{ id, content }],
  ...overrides,
});

const makeApproval = (overrides: Partial<ToolApproval> = {}): ToolApproval => ({
  approval_id: 'approval-1',
  tool_name: 'read_file',
  tool_args: { path: '/etc/passwd' },
  tool_description: 'Read file contents',
  message_id: MSG_ID,
  contentBeforeApproval: 'Content before approval',
  ...overrides,
});

const makeSessionData = (overrides: Partial<SessionData> = {}): SessionData => ({
  session_id: 'session-123',
  messages: [],
  created_at: 1_700_000_000_000,
  updated_at: 1_700_000_060_000,
  title: 'Test Session',
  agent_id: null,
  token_usage: { ...ZERO_USAGE },
  cost_usd: 0,
  ...overrides,
});

// ─── Per-test mock state objects ──────────────────────────────────────────────

let mockSessionState: {
  activeSessionId: string | null;
  setActiveSession: ReturnType<typeof vi.fn>;
  refreshSessions: ReturnType<typeof vi.fn>;
};

let mockAgentState: {
  selectedAgent: string;
  tokenUsage: TokenUsage;
  agents: AgentInfo[];
  updateTokenUsage: ReturnType<typeof vi.fn>;
  setSessionAgentId: ReturnType<typeof vi.fn>;
  setSelectedAgent: ReturnType<typeof vi.fn>;
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Seed a single assistant message into the store. */
const seedMessage = (id = MSG_ID, content = '', overrides: Partial<MessageType> = {}) => {
  useChatStore.setState({ messages: [makeMessage(id, content, overrides)] });
};

/**
 * Replace the streamChat mock for one call.
 * The provided function is invoked with the handlers object so individual
 * callbacks can be fired from test code.
 */
const mockStream = (fn: (h: StreamHandlers) => void | Promise<void>) => {
  vi.mocked(streamChat).mockImplementationOnce(async (_url, _msg, handlers) => {
    await fn(handlers);
  });
};

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('useChatStore', () => {
  beforeEach(() => {
    useChatStore.setState(INITIAL_STATE);
    localStorage.clear();
    vi.clearAllMocks();

    mockSessionState = {
      activeSessionId: null,
      setActiveSession: vi.fn(),
      refreshSessions: vi.fn().mockResolvedValue(undefined),
    };

    mockAgentState = {
      selectedAgent: 'agent-1',
      tokenUsage: { ...ZERO_USAGE },
      agents: [],
      updateTokenUsage: vi.fn(),
      setSessionAgentId: vi.fn(),
      setSelectedAgent: vi.fn(),
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    vi.mocked(useSessionStore.getState).mockReturnValue(mockSessionState as any);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    vi.mocked(useAgentStore.getState).mockReturnValue(mockAgentState as any);

    vi.spyOn(console, 'log').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  // ── Initial state ──────────────────────────────────────────────────────────

  describe('initial state', () => {
    it('has an empty messages array', () => {
      expect(useChatStore.getState().messages).toEqual([]);
    });

    it('has status "ready"', () => {
      expect(useChatStore.getState().status).toBe('ready');
    });

    it('has no streaming message id', () => {
      expect(useChatStore.getState().streamingMessageId).toBeNull();
    });

    it('has an empty streaming content ref', () => {
      expect(useChatStore.getState().streamingContentRef).toBe('');
    });

    it('has an empty streaming reasoning ref', () => {
      expect(useChatStore.getState().streamingReasoningRef).toBe('');
    });

    it('is not reasoning-streaming', () => {
      expect(useChatStore.getState().isReasoningStreaming).toBe(false);
    });

    it('has no abort controller', () => {
      expect(useChatStore.getState().abortController).toBeNull();
    });

    it('has web search disabled', () => {
      expect(useChatStore.getState().useWebSearch).toBe(false);
    });

    it('has RAG disabled', () => {
      expect(useChatStore.getState().useRag).toBe(false);
    });

    it('has tools disabled', () => {
      expect(useChatStore.getState().useTools).toBe(false);
    });

    it('has an empty pending approvals map', () => {
      expect(useChatStore.getState().pendingApprovals.size).toBe(0);
    });

    it('has an empty tool approval decisions map', () => {
      expect(useChatStore.getState().toolApprovalDecisions.size).toBe(0);
    });
  });

  // ── Simple setters ─────────────────────────────────────────────────────────

  describe('setStatus', () => {
    it.each(['submitted', 'streaming', 'ready', 'error'] as const)('sets status to "%s"', (s) => {
      useChatStore.getState().setStatus(s);
      expect(useChatStore.getState().status).toBe(s);
    });
  });

  describe('setStreamingMessageId', () => {
    it('sets a non-null message id', () => {
      useChatStore.getState().setStreamingMessageId('msg-42');
      expect(useChatStore.getState().streamingMessageId).toBe('msg-42');
    });

    it('clears the message id when called with null', () => {
      useChatStore.setState({ streamingMessageId: 'msg-42' });
      useChatStore.getState().setStreamingMessageId(null);
      expect(useChatStore.getState().streamingMessageId).toBeNull();
    });
  });

  describe('updateStreamingContent', () => {
    it('replaces the streaming content ref', () => {
      useChatStore.getState().updateStreamingContent('some streamed text');
      expect(useChatStore.getState().streamingContentRef).toBe('some streamed text');
    });
  });

  describe('setIsReasoningStreaming', () => {
    it('sets to true', () => {
      useChatStore.getState().setIsReasoningStreaming(true);
      expect(useChatStore.getState().isReasoningStreaming).toBe(true);
    });

    it('sets to false', () => {
      useChatStore.setState({ isReasoningStreaming: true });
      useChatStore.getState().setIsReasoningStreaming(false);
      expect(useChatStore.getState().isReasoningStreaming).toBe(false);
    });
  });

  // ── clearMessages ──────────────────────────────────────────────────────────

  describe('clearMessages', () => {
    beforeEach(() => {
      useChatStore.setState({
        messages: [makeMessage()],
        status: 'streaming',
        streamingMessageId: MSG_ID,
        streamingContentRef: 'partial',
        streamingReasoningRef: 'reasoning',
        isReasoningStreaming: true,
      });
    });

    it('empties the messages array', () => {
      useChatStore.getState().clearMessages();
      expect(useChatStore.getState().messages).toEqual([]);
    });

    it('resets status to "ready"', () => {
      useChatStore.getState().clearMessages();
      expect(useChatStore.getState().status).toBe('ready');
    });

    it('clears the streaming message id', () => {
      useChatStore.getState().clearMessages();
      expect(useChatStore.getState().streamingMessageId).toBeNull();
    });

    it('clears streaming content, reasoning refs, and the reasoning-streaming flag', () => {
      useChatStore.getState().clearMessages();
      expect(useChatStore.getState().streamingContentRef).toBe('');
      expect(useChatStore.getState().streamingReasoningRef).toBe('');
      expect(useChatStore.getState().isReasoningStreaming).toBe(false);
    });
  });

  // ── Toggles ────────────────────────────────────────────────────────────────

  describe('toggleWebSearch', () => {
    it('enables web search when it is off', () => {
      useChatStore.getState().toggleWebSearch();
      expect(useChatStore.getState().useWebSearch).toBe(true);
    });

    it('disables web search when it is on', () => {
      useChatStore.setState({ useWebSearch: true });
      useChatStore.getState().toggleWebSearch();
      expect(useChatStore.getState().useWebSearch).toBe(false);
    });
  });

  describe('toggleRag', () => {
    it('enables RAG when it is off', () => {
      useChatStore.getState().toggleRag();
      expect(useChatStore.getState().useRag).toBe(true);
    });

    it('disables RAG when it is on', () => {
      useChatStore.setState({ useRag: true });
      useChatStore.getState().toggleRag();
      expect(useChatStore.getState().useRag).toBe(false);
    });
  });

  describe('toggleTools', () => {
    it('enables tools when they are off', () => {
      useChatStore.getState().toggleTools();
      expect(useChatStore.getState().useTools).toBe(true);
    });

    it('disables tools when they are on', () => {
      useChatStore.setState({ useTools: true });
      useChatStore.getState().toggleTools();
      expect(useChatStore.getState().useTools).toBe(false);
    });
  });

  // ── updateMessageContent ───────────────────────────────────────────────────

  describe('updateMessageContent', () => {
    beforeEach(() => {
      seedMessage(MSG_ID, 'original');
    });

    it('updates the content of the matching version', () => {
      useChatStore.getState().updateMessageContent(MSG_ID, 'updated');
      expect(useChatStore.getState().messages[0].versions[0].content).toBe('updated');
    });

    it('does not affect messages with a different version id', () => {
      useChatStore.setState({
        messages: [makeMessage(MSG_ID, 'first'), makeMessage('other-id', 'second')],
      });
      useChatStore.getState().updateMessageContent(MSG_ID, 'changed');
      const other = useChatStore.getState().messages.find((m) => m.versions.some((v) => v.id === 'other-id'));
      expect(other?.versions[0].content).toBe('second');
    });

    it('is a no-op for an unknown message id', () => {
      useChatStore.getState().updateMessageContent('nonexistent', 'x');
      expect(useChatStore.getState().messages[0].versions[0].content).toBe('original');
    });
  });

  // ── addUserMessage ─────────────────────────────────────────────────────────

  describe('addUserMessage', () => {
    beforeEach(() => {
      vi.mocked(streamChat).mockResolvedValue(undefined);
    });

    it('immediately adds a user message with the given content', () => {
      vi.useFakeTimers();
      useChatStore.getState().addUserMessage('Hello');
      const messages = useChatStore.getState().messages;
      expect(messages).toHaveLength(1);
      expect(messages[0].from).toBe('user');
      expect(messages[0].versions[0].content).toBe('Hello');
    });

    it('does not add the assistant placeholder before the 500 ms delay', () => {
      vi.useFakeTimers();
      useChatStore.getState().addUserMessage('Hello');
      vi.advanceTimersByTime(499);
      expect(useChatStore.getState().messages).toHaveLength(1);
    });

    it('adds an empty assistant placeholder after the 500 ms delay', async () => {
      vi.useFakeTimers();
      useChatStore.getState().addUserMessage('Hello');
      vi.advanceTimersByTime(500);
      await Promise.resolve(); // flush microtask for the async streamResponse
      const messages = useChatStore.getState().messages;
      expect(messages).toHaveLength(2);
      expect(messages[1].from).toBe('assistant');
      expect(messages[1].versions[0].content).toBe('');
    });
  });

  // ── addPendingApproval ─────────────────────────────────────────────────────

  describe('addPendingApproval', () => {
    it('adds the approval to the pending approvals map', () => {
      const approval = makeApproval();
      useChatStore.getState().addPendingApproval(approval);
      expect(useChatStore.getState().pendingApprovals.get('approval-1')).toEqual(approval);
    });

    it('attaches the approval to the message whose version id matches message_id', () => {
      seedMessage(MSG_ID);
      useChatStore.getState().addPendingApproval(makeApproval({ message_id: MSG_ID }));
      const toolApprovals = useChatStore.getState().messages[0].toolApprovals;
      expect(toolApprovals).toHaveLength(1);
      expect(toolApprovals![0].approval_id).toBe('approval-1');
    });

    it('accumulates multiple approvals on the same message', () => {
      seedMessage(MSG_ID);
      useChatStore.getState().addPendingApproval(makeApproval({ approval_id: 'a1', message_id: MSG_ID }));
      useChatStore.getState().addPendingApproval(makeApproval({ approval_id: 'a2', message_id: MSG_ID }));
      expect(useChatStore.getState().messages[0].toolApprovals).toHaveLength(2);
    });
  });

  // ── clearApproval ──────────────────────────────────────────────────────────

  describe('clearApproval', () => {
    it('removes the approval from the pending approvals map', () => {
      useChatStore.setState({
        pendingApprovals: new Map([['approval-1', makeApproval()]]),
      });
      useChatStore.getState().clearApproval('approval-1');
      expect(useChatStore.getState().pendingApprovals.has('approval-1')).toBe(false);
    });

    it('is a no-op when the id does not exist in the map', () => {
      useChatStore.getState().clearApproval('nonexistent');
      expect(useChatStore.getState().pendingApprovals.size).toBe(0);
    });
  });

  // ── respondToApproval ──────────────────────────────────────────────────────

  describe('respondToApproval', () => {
    const APPROVAL = makeApproval();

    beforeEach(() => {
      useChatStore.setState({
        pendingApprovals: new Map([['approval-1', APPROVAL]]),
      });
    });

    it('logs an error and returns early when the approval id is unknown', async () => {
      const consoleSpy = vi.spyOn(console, 'error');
      await useChatStore.getState().respondToApproval('missing', true, false);
      expect(vi.mocked(sendToolApproval)).not.toHaveBeenCalled();
      expect(consoleSpy).toHaveBeenCalledWith('Approval not found:', 'missing');
    });

    it('calls sendToolApproval with the correct arguments', async () => {
      vi.mocked(sendToolApproval).mockResolvedValueOnce(true);
      await useChatStore.getState().respondToApproval('approval-1', true, true, 'session');
      expect(vi.mocked(sendToolApproval)).toHaveBeenCalledWith('', 'approval-1', true, true, 'session');
    });

    it('records the decision in toolApprovalDecisions on success', async () => {
      vi.mocked(sendToolApproval).mockResolvedValueOnce(true);
      await useChatStore.getState().respondToApproval('approval-1', true, false);
      const decision = useChatStore.getState().toolApprovalDecisions.get('approval-1');
      expect(decision).toBeDefined();
      expect(decision?.approved).toBe(true);
    });

    it('shows a success toast when the tool execution is approved', async () => {
      vi.mocked(sendToolApproval).mockResolvedValueOnce(true);
      await useChatStore.getState().respondToApproval('approval-1', true, false);
      expect(vi.mocked(toast.success)).toHaveBeenCalledWith(
        'Tool execution approved',
        expect.objectContaining({ description: expect.stringContaining('read_file') })
      );
    });

    it('shows an info toast when the tool execution is rejected', async () => {
      vi.mocked(sendToolApproval).mockResolvedValueOnce(true);
      await useChatStore.getState().respondToApproval('approval-1', false, false);
      expect(vi.mocked(toast.info)).toHaveBeenCalledWith(
        'Tool execution rejected',
        expect.objectContaining({ description: expect.stringContaining('read_file') })
      );
    });

    it('shows an error toast when the API call reports failure', async () => {
      vi.mocked(sendToolApproval).mockResolvedValueOnce(false);
      await useChatStore.getState().respondToApproval('approval-1', true, false);
      expect(vi.mocked(toast.error)).toHaveBeenCalledWith('Failed to send approval', expect.anything());
    });
  });

  // ── stopStreaming ──────────────────────────────────────────────────────────

  describe('stopStreaming', () => {
    it('calls abort() on the active controller', () => {
      const controller = new AbortController();
      const abortSpy = vi.spyOn(controller, 'abort');
      useChatStore.setState({ abortController: controller });
      useChatStore.getState().stopStreaming();
      expect(abortSpy).toHaveBeenCalledOnce();
    });

    it('resets status, streamingMessageId, and abortController after aborting', () => {
      const controller = new AbortController();
      useChatStore.setState({
        abortController: controller,
        status: 'streaming',
        streamingMessageId: MSG_ID,
      });
      useChatStore.getState().stopStreaming();
      const { status, streamingMessageId, abortController } = useChatStore.getState();
      expect(status).toBe('ready');
      expect(streamingMessageId).toBeNull();
      expect(abortController).toBeNull();
    });

    it('does nothing when there is no active abort controller', () => {
      expect(() => useChatStore.getState().stopStreaming()).not.toThrow();
      expect(useChatStore.getState().status).toBe('ready');
    });
  });

  // ── loadSessionHistory ─────────────────────────────────────────────────────

  describe('loadSessionHistory', () => {
    it('shows an error toast and returns without changing messages when session is null', async () => {
      vi.mocked(loadSession).mockResolvedValueOnce(null);
      await useChatStore.getState().loadSessionHistory('missing-id');
      expect(vi.mocked(toast.error)).toHaveBeenCalledWith('Session not found');
      expect(useChatStore.getState().messages).toHaveLength(0);
    });

    // Note: Active session is now managed by URL in App.tsx, not by loadSessionHistory

    it('maps session messages to UI messages preserving role and content', async () => {
      vi.mocked(loadSession).mockResolvedValueOnce(
        makeSessionData({
          messages: [
            { role: 'user', content: 'Hi', sources: [], timestamp: 1_700_000_000 },
            { role: 'assistant', content: 'Hello!', sources: [], timestamp: 1_700_000_001 },
          ],
        })
      );
      await useChatStore.getState().loadSessionHistory('sess-1');
      const messages = useChatStore.getState().messages;
      expect(messages).toHaveLength(2);
      expect(messages[0].from).toBe('user');
      expect(messages[0].versions[0].content).toBe('Hi');
      expect(messages[1].from).toBe('assistant');
      expect(messages[1].versions[0].content).toBe('Hello!');
    });

    it('maps non-empty sources and sets href to "#"', async () => {
      vi.mocked(loadSession).mockResolvedValueOnce(
        makeSessionData({
          messages: [
            {
              role: 'assistant',
              content: 'See sources',
              timestamp: 1_700_000_000,
              sources: [{ title: 'Doc A', content: 'body text' }],
            },
          ],
        })
      );
      await useChatStore.getState().loadSessionHistory('sess-1');
      const sources = useChatStore.getState().messages[0].sources;
      expect(sources).toHaveLength(1);
      expect(sources![0].title).toBe('Doc A');
      expect(sources![0].href).toBe('#');
    });

    it('leaves sources undefined when the message has no sources', async () => {
      vi.mocked(loadSession).mockResolvedValueOnce(
        makeSessionData({
          messages: [{ role: 'user', content: 'Hi', sources: [], timestamp: 1_700_000_000 }],
        })
      );
      await useChatStore.getState().loadSessionHistory('sess-1');
      expect(useChatStore.getState().messages[0].sources).toBeUndefined();
    });

    it('maps a reasoning thinking step', async () => {
      vi.mocked(loadSession).mockResolvedValueOnce(
        makeSessionData({
          messages: [
            {
              role: 'assistant',
              content: 'Answer',
              sources: [],
              timestamp: 1_700_000_000,
              thinking_steps: [{ step_type: 'reasoning', step_order: 0, content: 'I think therefore I am' }],
            },
          ],
        })
      );
      await useChatStore.getState().loadSessionHistory('sess-1');
      const steps = useChatStore.getState().messages[0].thinkingSteps;
      expect(steps).toHaveLength(1);
      expect(steps![0]).toEqual({ type: 'reasoning', content: 'I think therefore I am' });
    });

    it('maps a completed tool thinking step', async () => {
      vi.mocked(loadSession).mockResolvedValueOnce(
        makeSessionData({
          messages: [
            {
              role: 'assistant',
              content: 'Done',
              sources: [],
              timestamp: 1_700_000_000,
              thinking_steps: [
                {
                  step_type: 'tool',
                  step_order: 0,
                  tool_name: 'read_file',
                  tool_arguments: { path: '/tmp/x' },
                  tool_result: 'file content',
                },
              ],
            },
          ],
        })
      );
      await useChatStore.getState().loadSessionHistory('sess-1');
      const step = useChatStore.getState().messages[0].thinkingSteps![0];
      expect(step.type).toBe('tool');
      if (step.type === 'tool') {
        expect(step.name).toBe('read_file');
        expect(step.status).toBe('completed');
        expect(step.result).toBe('file content');
      }
    });

    it('sets the tool step status to "error" when tool_error is present', async () => {
      vi.mocked(loadSession).mockResolvedValueOnce(
        makeSessionData({
          messages: [
            {
              role: 'assistant',
              content: 'Err',
              sources: [],
              timestamp: 1_700_000_000,
              thinking_steps: [
                {
                  step_type: 'tool',
                  step_order: 0,
                  tool_name: 'exec',
                  tool_arguments: {},
                  tool_error: 'Permission denied',
                },
              ],
            },
          ],
        })
      );
      await useChatStore.getState().loadSessionHistory('sess-1');
      const step = useChatStore.getState().messages[0].thinkingSteps![0];
      expect(step.type).toBe('tool');
      if (step.type === 'tool') {
        expect(step.status).toBe('error');
        expect(step.error).toBe('Permission denied');
      }
    });

    it('sets status to "ready" after loading', async () => {
      useChatStore.setState({ status: 'streaming' });
      vi.mocked(loadSession).mockResolvedValueOnce(makeSessionData());
      await useChatStore.getState().loadSessionHistory('sess-1');
      expect(useChatStore.getState().status).toBe('ready');
    });

    it('updates token usage from the session data', async () => {
      const usage = { ...ZERO_USAGE, total_tokens: 200, input_tokens: 100, output_tokens: 100 };
      vi.mocked(loadSession).mockResolvedValueOnce(makeSessionData({ token_usage: usage }));
      await useChatStore.getState().loadSessionHistory('sess-1');
      expect(mockAgentState.updateTokenUsage).toHaveBeenCalledWith(usage);
    });

    it('sets the session agent id on the agent store', async () => {
      vi.mocked(loadSession).mockResolvedValueOnce(makeSessionData({ agent_id: 'agent-from-session' }));
      await useChatStore.getState().loadSessionHistory('sess-1');
      expect(mockAgentState.setSessionAgentId).toHaveBeenCalledWith('agent-from-session');
    });

    it('restores the selected agent when the agent exists in the agents list', async () => {
      mockAgentState.agents = [
        {
          id: 'agent-from-session',
          name: 'Session Agent',
          description: '',
          model: 'openai/gpt-4o',
          enabled: true,
          use_tools: false,
          permissions: { allow: [], deny: [] },
        },
      ];
      vi.mocked(loadSession).mockResolvedValueOnce(makeSessionData({ agent_id: 'agent-from-session' }));
      await useChatStore.getState().loadSessionHistory('sess-1');
      expect(mockAgentState.setSelectedAgent).toHaveBeenCalledWith('agent-from-session');
    });

    it('does not call setSelectedAgent when agents list is empty', async () => {
      mockAgentState.agents = [];
      vi.mocked(loadSession).mockResolvedValueOnce(makeSessionData({ agent_id: 'agent-from-session' }));
      await useChatStore.getState().loadSessionHistory('sess-1');
      expect(mockAgentState.setSelectedAgent).not.toHaveBeenCalled();
    });
  });

  // ── streamResponse ─────────────────────────────────────────────────────────

  describe('streamResponse', () => {
    beforeEach(() => {
      seedMessage(MSG_ID, '');
    });

    it('sets status to "streaming" and records streamingMessageId synchronously', async () => {
      let resolve!: () => void;
      vi.mocked(streamChat).mockImplementationOnce(() => new Promise<void>((r) => (resolve = r)));
      const p = useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(useChatStore.getState().status).toBe('streaming');
      expect(useChatStore.getState().streamingMessageId).toBe(MSG_ID);
      resolve();
      await p;
    });

    it('passes the active session id to streamChat', async () => {
      mockSessionState.activeSessionId = 'active-session';
      mockStream(() => {});
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      const [, msgArg] = vi.mocked(streamChat).mock.calls[0];
      expect(msgArg.session_id).toBe('active-session');
    });

    it('passes useRag and useTools to streamChat when enabled', async () => {
      useChatStore.setState({ useRag: true, useTools: true });
      mockStream(() => {});
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      const [, msgArg] = vi.mocked(streamChat).mock.calls[0];
      expect(msgArg.use_rag).toBe(true);
      expect(msgArg.use_tools).toBe(true);
    });

    // ── onSession ────────────────────────────────────────────────────────────

    it('calls setActiveSession when onSession fires', async () => {
      mockStream((h) => h.onSession?.('new-session-id'));
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(mockSessionState.setActiveSession).toHaveBeenCalledWith('new-session-id');
    });

    // ── onContent (plain text) ───────────────────────────────────────────────

    it('updates the message content with plain streamed text', async () => {
      mockStream((h) => h.onContent('Hello world'));
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(useChatStore.getState().messages[0].versions[0].content).toBe('Hello world');
    });

    it('accumulates text across multiple onContent calls', async () => {
      mockStream((h) => {
        h.onContent('Hello ');
        h.onContent('world');
      });
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(useChatStore.getState().messages[0].versions[0].content).toBe('Hello world');
    });

    // ── onContent (<think> parsing) ──────────────────────────────────────────

    it('extracts a complete <think> block into a reasoning thinking step', async () => {
      mockStream((h) => h.onContent('<think>I reasoned about this</think>The answer'));
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      const msg = useChatStore.getState().messages[0];
      expect(msg.versions[0].content).toBe('The answer');
      expect(msg.thinkingSteps).toHaveLength(1);
      expect(msg.thinkingSteps![0]).toEqual({
        type: 'reasoning',
        content: 'I reasoned about this',
      });
    });

    it('preserves content before and after a <think> block in the display content', async () => {
      mockStream((h) => h.onContent('Prefix<think>reasoning</think>Suffix'));
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(useChatStore.getState().messages[0].versions[0].content).toBe('PrefixSuffix');
    });

    it('sets isReasoningStreaming=true when an unclosed <think> tag is encountered', async () => {
      mockStream((h) => h.onContent('<think>Thinking in progress'));
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(useChatStore.getState().isReasoningStreaming).toBe(true);
    });

    // ── onSources ────────────────────────────────────────────────────────────

    it('attaches sources to the matching message with href="#"', async () => {
      mockStream((h) => h.onSources?.([{ title: 'Doc A', content: 'body' }]));
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      const sources = useChatStore.getState().messages[0].sources;
      expect(sources).toHaveLength(1);
      expect(sources![0].title).toBe('Doc A');
      expect(sources![0].href).toBe('#');
      expect(sources![0].content).toBe('body');
    });

    // ── onError ──────────────────────────────────────────────────────────────

    it('updates the message content with the error text', async () => {
      mockStream((h) => h.onError?.('Something went wrong'));
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(useChatStore.getState().messages[0].versions[0].content).toBe('Error: Something went wrong');
    });

    it('resets status, streamingMessageId, and abortController after an error', async () => {
      mockStream((h) => h.onError?.('boom'));
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      const { status, streamingMessageId, abortController } = useChatStore.getState();
      expect(status).toBe('ready');
      expect(streamingMessageId).toBeNull();
      expect(abortController).toBeNull();
    });

    it('shows a toast error when onError fires', async () => {
      mockStream((h) => h.onError?.('Something went wrong'));
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
        'Failed to get response',
        expect.objectContaining({ description: 'Something went wrong' })
      );
    });

    // ── onDone ───────────────────────────────────────────────────────────────

    it('resets streaming state when onDone fires', async () => {
      mockStream(async (h) => {
        h.onContent('final content');
        await h.onDone?.();
      });
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      const { status, streamingMessageId, abortController, streamingContentRef, isReasoningStreaming } =
        useChatStore.getState();
      expect(status).toBe('ready');
      expect(streamingMessageId).toBeNull();
      expect(abortController).toBeNull();
      expect(streamingContentRef).toBe('');
      expect(isReasoningStreaming).toBe(false);
    });

    it('calls refreshSessions after onDone', async () => {
      mockStream(async (h) => {
        await h.onDone?.();
      });
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(mockSessionState.refreshSessions).toHaveBeenCalledOnce();
    });

    it('loads the session token usage when activeSessionId is set', async () => {
      mockSessionState.activeSessionId = 'active-session';
      const usage = { ...ZERO_USAGE, total_tokens: 42 };
      vi.mocked(loadSession).mockResolvedValueOnce(makeSessionData({ token_usage: usage }));
      mockStream(async (h) => {
        await h.onDone?.();
      });
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(vi.mocked(loadSession)).toHaveBeenCalledWith('', 'active-session');
      expect(mockAgentState.updateTokenUsage).toHaveBeenCalledWith(usage);
    });

    it('does not call loadSession when there is no active session', async () => {
      mockSessionState.activeSessionId = null;
      mockStream(async (h) => {
        await h.onDone?.();
      });
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(vi.mocked(loadSession)).not.toHaveBeenCalled();
    });

    // ── AbortError ───────────────────────────────────────────────────────────

    it('sets the message to the streamed content so far when the request is aborted', async () => {
      vi.mocked(streamChat).mockImplementationOnce(async () => {
        // Simulate content already received before the abort
        useChatStore.setState({ streamingContentRef: 'partial response' });
        const err = new Error('Aborted');
        err.name = 'AbortError';
        throw err;
      });
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(useChatStore.getState().messages[0].versions[0].content).toBe('partial response');
    });

    it('falls back to "Response stopped by user." when no content was streamed before abort', async () => {
      vi.mocked(streamChat).mockImplementationOnce(async () => {
        const err = new Error('Aborted');
        err.name = 'AbortError';
        throw err;
      });
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(useChatStore.getState().messages[0].versions[0].content).toBe('Response stopped by user.');
    });

    it('resets streaming state after an AbortError', async () => {
      vi.mocked(streamChat).mockImplementationOnce(async () => {
        const err = new Error('Aborted');
        err.name = 'AbortError';
        throw err;
      });
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      const { status, streamingMessageId, abortController } = useChatStore.getState();
      expect(status).toBe('ready');
      expect(streamingMessageId).toBeNull();
      expect(abortController).toBeNull();
    });

    it('shows a toast and logs on a non-abort error thrown by streamChat', async () => {
      const err = new Error('Network failure');
      vi.mocked(streamChat).mockRejectedValueOnce(err);
      const consoleSpy = vi.spyOn(console, 'error');
      await useChatStore.getState().streamResponse(MSG_ID, 'Hello');
      expect(vi.mocked(toast.error)).toHaveBeenCalledWith(
        'Failed to send message',
        expect.objectContaining({ description: 'Network failure' })
      );
      expect(consoleSpy).toHaveBeenCalledWith('Chat error:', err);
    });
  });
});
