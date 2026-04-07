import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, afterEach, beforeAll } from 'vitest';
import { AgentStatsCard } from './agent-stats';
import type { AgentTokenStats } from '@/lib/chat-api';

beforeAll(() => {
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn(() => false);
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
  window.HTMLElement.prototype.setPointerCapture = vi.fn();
});

const makeAgentStats = (overrides: Partial<Record<keyof AgentTokenStats, unknown>> = {}) => ({
  agent_id: 'test-agent',
  total_sessions: 10,
  total_tokens: 50000,
  input_tokens: 30000,
  output_tokens: 20000,
  reasoning_tokens: 5000,
  cache_tokens: 10000,
  total_cost_usd: 0.1234,
  avg_cost_per_session: 0.0123,
  first_used_at: 1700000000,
  last_used_at: 1700100000,
  ...overrides,
});

const makeResponse = (agents: Array<Record<string, unknown>> = [makeAgentStats()]) => ({
  agents,
});

const ok = (data: object) => ({ ok: true, json: async () => data });
const nok = (status = 500) => ({ ok: false, status });

type MockResponse = ReturnType<typeof ok> | ReturnType<typeof nok>;

const stubFetch = (...responses: MockResponse[]) => {
  const mock = vi.fn();
  responses.forEach((r) => mock.mockResolvedValueOnce(r));
  vi.stubGlobal('fetch', mock);
  return mock;
};

describe('AgentStatsCard', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('shows loading message on initial render', async () => {
    stubFetch(ok(makeResponse()));
    render(<AgentStatsCard apiUrl="" />);
    expect(screen.getByText('Loading agent statistics...')).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByText('Loading agent statistics...')).not.toBeInTheDocument());
  });

  it('hides loading message after data has loaded', async () => {
    stubFetch(ok(makeResponse()));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.queryByText('Loading agent statistics...')).not.toBeInTheDocument());
  });

  it('shows error message when fetch fails with non-ok response', async () => {
    stubFetch(nok());
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('Failed to fetch agent statistics: HTTP 500')).toBeInTheDocument());
  });

  it('shows error message when fetch throws a network error', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValueOnce(new Error('Network error')));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('Network error')).toBeInTheDocument());
  });

  it('displays the thrown error message correctly', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValueOnce(new Error('Custom error message')));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('Custom error message')).toBeInTheDocument());
  });

  it('shows empty state message when agents array is empty', async () => {
    stubFetch(ok(makeResponse([])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() =>
      expect(screen.getByText('No agent statistics available yet. Start using agents to see usage data.')).toBeInTheDocument()
    );
  });

  it('renders all agent stat cards when multiple agents exist', async () => {
    const agents = [
      makeAgentStats({ agent_id: 'agent-1' }),
      makeAgentStats({ agent_id: 'agent-2' }),
      makeAgentStats({ agent_id: 'agent-3' }),
    ];
    stubFetch(ok(makeResponse(agents)));
    render(<AgentStatsCard apiUrl="" />);

    await waitFor(() => expect(screen.getByText('agent-1')).toBeInTheDocument());
    expect(screen.getByText('agent-2')).toBeInTheDocument();
    expect(screen.getByText('agent-3')).toBeInTheDocument();
  });

  it('renders agent ID for each stat card', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ agent_id: 'my-custom-agent' })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('my-custom-agent')).toBeInTheDocument());
  });

  it('renders session count badge', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ total_sessions: 42 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('42 sessions')).toBeInTheDocument());
  });

  it('shows total tokens with proper formatting', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ total_tokens: 50000 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('50.0K')).toBeInTheDocument());
  });

  it('shows input/output token breakdown', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ input_tokens: 30000, output_tokens: 20000 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('30.0K in / 20.0K out')).toBeInTheDocument());
  });

  it('shows total cost with proper formatting', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ total_cost_usd: 0.1234 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('$0.1234')).toBeInTheDocument());
  });

  it('shows average cost per session', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ avg_cost_per_session: 0.0123 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('$0.0123 per session')).toBeInTheDocument());
  });

  it('shows cache hit rate percentage', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ cache_tokens: 10000, total_tokens: 50000 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('20.0%')).toBeInTheDocument());
  });

  it('shows cached token count', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ cache_tokens: 10000 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('10.0K cached')).toBeInTheDocument());
  });

  it('shows usage period with formatted dates', async () => {
    const firstDate = new Date(1700000000 * 1000).toLocaleDateString();
    const lastDate = new Date(1700100000 * 1000).toLocaleDateString();

    stubFetch(ok(makeResponse([
      makeAgentStats({ first_used_at: 1700000000, last_used_at: 1700100000 }),
    ])));
    render(<AgentStatsCard apiUrl="" />);

    await waitFor(() => expect(screen.getByText(firstDate)).toBeInTheDocument());
    expect(screen.getByText(`to ${lastDate}`)).toBeInTheDocument();
  });

  it('shows reasoning tokens section when reasoning_tokens > 0', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ reasoning_tokens: 5000, total_tokens: 50000 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText(/Reasoning tokens: 5.0K/)).toBeInTheDocument());
  });

  it('hides reasoning tokens section when reasoning_tokens === 0', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ reasoning_tokens: 0 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.queryByText(/Reasoning tokens:/)).not.toBeInTheDocument());
  });

  it('formats numbers >= 1,000,000 as "X.XM"', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ total_tokens: 1500000 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('1.5M')).toBeInTheDocument());
  });

  it('formats numbers >= 1,000 as "X.XK"', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ total_tokens: 2500 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('2.5K')).toBeInTheDocument());
  });

  it('formats numbers < 1,000 as string', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ total_tokens: 42 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('42')).toBeInTheDocument());
  });

  it('calls fetchAllAgentStats with the correct apiUrl prop', async () => {
    const fetchMock = stubFetch(ok(makeResponse()));
    render(<AgentStatsCard apiUrl="https://example.com" />);
    await waitFor(() => expect(screen.queryByText('Loading agent statistics...')).not.toBeInTheDocument());
    expect(fetchMock).toHaveBeenCalledWith('https://example.com/api/agents/stats');
  });

  it('handles zero values gracefully without division by zero errors', async () => {
    stubFetch(ok(makeResponse([
      makeAgentStats({
        total_tokens: 0,
        input_tokens: 0,
        output_tokens: 0,
        reasoning_tokens: 0,
        cache_tokens: 0,
        total_cost_usd: 0,
        avg_cost_per_session: 0,
        total_sessions: 0,
      }),
    ])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('0')).toBeInTheDocument());
    expect(screen.getByText('0%')).toBeInTheDocument();
    expect(screen.getByText('$0.0000')).toBeInTheDocument();
  });

  it('handles very large token numbers correctly', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ total_tokens: 999999999 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('1000.0M')).toBeInTheDocument());
  });

  it('renders correctly with minimal data (all zeros)', async () => {
    stubFetch(ok(makeResponse([
      makeAgentStats({
        total_tokens: 0,
        input_tokens: 0,
        output_tokens: 0,
        reasoning_tokens: 0,
        cache_tokens: 0,
        total_cost_usd: 0,
        avg_cost_per_session: 0,
        total_sessions: 0,
        first_used_at: 0,
        last_used_at: 0,
      }),
    ])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText('test-agent')).toBeInTheDocument());
  });

  it('calculates reasoning token percentage correctly', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ reasoning_tokens: 5000, total_tokens: 50000 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText(/10\.0% of total/)).toBeInTheDocument());
  });

  it('shows 0% for reasoning token percentage when total_tokens is 0', async () => {
    stubFetch(ok(makeResponse([makeAgentStats({ reasoning_tokens: 1000, total_tokens: 0 })])));
    render(<AgentStatsCard apiUrl="" />);
    await waitFor(() => expect(screen.getByText(/0% of total/)).toBeInTheDocument());
  });
});
