import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, afterEach, beforeAll } from 'vitest';
import userEvent from '@testing-library/user-event';
import Logs from './logs';

// ─── JSDOM polyfills required by Radix UI ────────────────────────────────────

beforeAll(() => {
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn(() => false);
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
  window.HTMLElement.prototype.setPointerCapture = vi.fn();
});

// ─── Fixtures ────────────────────────────────────────────────────────────────

const BASE_LOGS = [
  { id: 1, timestamp: 1700000000, level: 'info', target: 'app::server', message: 'Server started', session_id: null },
  {
    id: 2,
    timestamp: 1700000001,
    level: 'error',
    target: 'app::handler',
    message: 'Request failed',
    session_id: 'abc',
  },
  { id: 3, timestamp: 1700000002, level: 'warn', target: 'app::cache', message: 'Cache miss', session_id: null },
  { id: 4, timestamp: 1700000003, level: 'debug', target: 'app::db', message: 'Query executed', session_id: null },
  { id: 5, timestamp: 1700000004, level: 'trace', target: 'app::router', message: 'Route matched', session_id: null },
];

const makeResponse = (overrides: Record<string, unknown> = {}) => ({
  logs: BASE_LOGS,
  total: 5,
  page: 1,
  page_size: 50,
  total_pages: 1,
  ...overrides,
});

// ─── Helpers ─────────────────────────────────────────────────────────────────

const ok = (data: object) => ({ ok: true, json: async () => data });
const nok = () => ({ ok: false });

type MockResponse = ReturnType<typeof ok> | ReturnType<typeof nok>;

const stubFetch = (...responses: MockResponse[]) => {
  const mock = vi.fn();
  responses.forEach((r) => mock.mockResolvedValueOnce(r));
  vi.stubGlobal('fetch', mock);
  return mock;
};

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('Logs', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  // ── Loading state ──────────────────────────────────────────────────────────

  it('shows a loading spinner on initial render', async () => {
    stubFetch(ok(makeResponse()));
    render(<Logs />);
    expect(screen.getByRole('status')).toBeInTheDocument();
    // Drain pending state updates so React doesn't warn about act()
    await waitFor(() => expect(screen.queryByRole('status')).not.toBeInTheDocument());
  });

  it('hides the spinner after data has loaded', async () => {
    stubFetch(ok(makeResponse()));
    render(<Logs />);
    await waitFor(() => expect(screen.queryByRole('status')).not.toBeInTheDocument());
  });

  // ── Data rendering ─────────────────────────────────────────────────────────

  it('renders all log messages', async () => {
    stubFetch(ok(makeResponse()));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());
    expect(screen.getByText('Request failed')).toBeInTheDocument();
    expect(screen.getByText('Cache miss')).toBeInTheDocument();
    expect(screen.getByText('Query executed')).toBeInTheDocument();
    expect(screen.getByText('Route matched')).toBeInTheDocument();
  });

  it('renders the target for each log entry', async () => {
    stubFetch(ok(makeResponse()));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('app::server')).toBeInTheDocument());
    expect(screen.getByText('app::handler')).toBeInTheDocument();
    expect(screen.getByText('app::cache')).toBeInTheDocument();
    expect(screen.getByText('app::db')).toBeInTheDocument();
    expect(screen.getByText('app::router')).toBeInTheDocument();
  });

  it('renders all table column headers', async () => {
    stubFetch(ok(makeResponse()));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Timestamp')).toBeInTheDocument());
    expect(screen.getByText('Level')).toBeInTheDocument();
    expect(screen.getByText('Target')).toBeInTheDocument();
    expect(screen.getByText('Message')).toBeInTheDocument();
  });

  it('renders the page heading', async () => {
    stubFetch(ok(makeResponse()));
    render(<Logs />);
    await waitFor(() => expect(screen.getByText('Application Logs')).toBeInTheDocument());
  });

  // ── Total count display ────────────────────────────────────────────────────

  it('shows plural entry count in the header', async () => {
    stubFetch(ok(makeResponse({ total: 5 })));
    render(<Logs />);
    await waitFor(() => expect(screen.getByText('Showing 5 log entries')).toBeInTheDocument());
  });

  it('shows singular entry count when total is 1', async () => {
    stubFetch(ok(makeResponse({ logs: [BASE_LOGS[0]], total: 1 })));
    render(<Logs />);
    await waitFor(() => expect(screen.getByText('Showing 1 log entry')).toBeInTheDocument());
  });

  it('shows "No logs found" in the header when total is 0', async () => {
    stubFetch(ok(makeResponse({ logs: [], total: 0 })));
    render(<Logs />);
    await waitFor(() => expect(screen.getByText('No logs found')).toBeInTheDocument());
  });

  // ── Empty state ────────────────────────────────────────────────────────────

  it('shows the empty-state message when the logs array is empty', async () => {
    stubFetch(ok(makeResponse({ logs: [], total: 0 })));
    render(<Logs />);
    await waitFor(() => expect(screen.getByText('No logs found with the current filters')).toBeInTheDocument());
  });

  // ── Error state ────────────────────────────────────────────────────────────

  it('shows an error message when the server responds with a non-ok status', async () => {
    stubFetch(nok());
    render(<Logs />);
    await waitFor(() => expect(screen.getByText('Failed to fetch logs')).toBeInTheDocument());
  });

  it('shows a Retry button alongside the error message', async () => {
    stubFetch(nok());
    render(<Logs />);
    await waitFor(() => expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument());
  });

  it('shows the thrown error message on a network failure', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValueOnce(new Error('Network error')));
    render(<Logs />);
    await waitFor(() => expect(screen.getByText('Network error')).toBeInTheDocument());
  });

  it('re-fetches when the Retry button is clicked', async () => {
    const fetchMock = stubFetch(nok(), ok(makeResponse()));
    render(<Logs />);

    await waitFor(() => expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: /retry/i }));

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  // ── Badge variants ─────────────────────────────────────────────────────────

  it.each([
    ['ERROR', 'destructive'],
    ['WARN', 'outline'],
    ['INFO', 'default'],
    ['DEBUG', 'secondary'],
    ['TRACE', 'secondary'],
  ])('renders the %s badge with data-variant="%s"', async (levelText, expectedVariant) => {
    stubFetch(ok(makeResponse()));
    render(<Logs />);

    await waitFor(() => {
      const badge = screen.getByText(levelText);
      expect(badge).toHaveAttribute('data-variant', expectedVariant);
    });
  });

  // ── Pagination button state ────────────────────────────────────────────────

  it('disables the Previous button on the first page', async () => {
    stubFetch(ok(makeResponse({ page: 1, total_pages: 3, total: 150 })));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());
    expect(screen.getByRole('button', { name: /previous/i })).toBeDisabled();
  });

  it('disables the Next button on the last page', async () => {
    stubFetch(ok(makeResponse({ page: 1, total_pages: 1 })));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());
    expect(screen.getByRole('button', { name: /next/i })).toBeDisabled();
  });

  it('enables the Next button when more pages are available', async () => {
    stubFetch(ok(makeResponse({ page: 1, total_pages: 3, total: 150 })));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());
    expect(screen.getByRole('button', { name: /next/i })).not.toBeDisabled();
  });

  it('enables the Previous button after moving to the second page', async () => {
    stubFetch(
      ok(makeResponse({ page: 1, total_pages: 3, total: 150 })),
      ok(makeResponse({ page: 2, total_pages: 3, total: 150 }))
    );
    render(<Logs />);

    await waitFor(() => expect(screen.getByRole('button', { name: /next/i })).not.toBeDisabled());
    fireEvent.click(screen.getByRole('button', { name: /next/i }));

    await waitFor(() => expect(screen.getByRole('button', { name: /previous/i })).not.toBeDisabled());
  });

  // ── Pagination info text ───────────────────────────────────────────────────

  it('shows the correct pagination info text', async () => {
    stubFetch(ok(makeResponse({ page: 1, total_pages: 4, total: 200 })));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Page 1 of 4 (200 total entries)')).toBeInTheDocument());
  });

  it('uses singular "entry" in pagination info when total is 1', async () => {
    stubFetch(ok(makeResponse({ logs: [BASE_LOGS[0]], page: 1, total_pages: 1, total: 1 })));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Page 1 of 1 (1 total entry)')).toBeInTheDocument());
  });

  // ── Fetch URL parameters ───────────────────────────────────────────────────

  it('fetches with default page=1 and page_size=50', async () => {
    const fetchMock = stubFetch(ok(makeResponse()));
    render(<Logs />);

    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/logs?page=1&page_size=50'));
  });

  it('fetches the next page when Next is clicked', async () => {
    const fetchMock = stubFetch(
      ok(makeResponse({ page: 1, total_pages: 3, total: 150 })),
      ok(makeResponse({ page: 2, total_pages: 3, total: 150 }))
    );
    render(<Logs />);

    await waitFor(() => expect(screen.getByRole('button', { name: /next/i })).not.toBeDisabled());
    fireEvent.click(screen.getByRole('button', { name: /next/i }));

    await waitFor(() => expect(fetchMock).toHaveBeenLastCalledWith('/api/logs?page=2&page_size=50'));
  });

  it('fetches the previous page when Previous is clicked', async () => {
    const fetchMock = stubFetch(
      ok(makeResponse({ page: 1, total_pages: 3, total: 150 })),
      ok(makeResponse({ page: 2, total_pages: 3, total: 150 })),
      ok(makeResponse({ page: 1, total_pages: 3, total: 150 }))
    );
    render(<Logs />);

    await waitFor(() => expect(screen.getByRole('button', { name: /next/i })).not.toBeDisabled());
    fireEvent.click(screen.getByRole('button', { name: /next/i }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));

    fireEvent.click(screen.getByRole('button', { name: /previous/i }));
    await waitFor(() => expect(fetchMock).toHaveBeenLastCalledWith('/api/logs?page=1&page_size=50'));
  });

  it('does not go below page 1 when Previous is clicked on the first page', async () => {
    const fetchMock = stubFetch(ok(makeResponse({ page: 1, total_pages: 3 })));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: /previous/i }));

    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it('does not go past the last page when Next is clicked on the last page', async () => {
    const fetchMock = stubFetch(ok(makeResponse({ page: 1, total_pages: 1 })));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: /next/i }));

    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  // ── Level filter ───────────────────────────────────────────────────────────

  it('appends the level query param when a level filter is chosen', async () => {
    const user = userEvent.setup();
    const fetchMock = stubFetch(ok(makeResponse()), ok(makeResponse()));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());

    // The first combobox is the Level select
    const [levelTrigger] = screen.getAllByRole('combobox');
    await user.click(levelTrigger);

    const errorOption = await screen.findByRole('option', { name: 'Error' });
    await user.click(errorOption);

    await waitFor(() => expect(fetchMock).toHaveBeenLastCalledWith('/api/logs?page=1&page_size=50&level=error'));
  });

  it('omits the level param when "All" is selected', async () => {
    const user = userEvent.setup();
    const fetchMock = stubFetch(ok(makeResponse()), ok(makeResponse()), ok(makeResponse()));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());

    // Select "Error" first
    const [levelTrigger] = screen.getAllByRole('combobox');
    await user.click(levelTrigger);
    await user.click(await screen.findByRole('option', { name: 'Error' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));

    // Then switch back to "All"
    await user.click(levelTrigger);
    await user.click(await screen.findByRole('option', { name: 'All' }));

    await waitFor(() => expect(fetchMock).toHaveBeenLastCalledWith('/api/logs?page=1&page_size=50'));
  });

  it('resets to page 1 when the level filter changes', async () => {
    const user = userEvent.setup();
    const fetchMock = stubFetch(
      ok(makeResponse({ page: 1, total_pages: 3, total: 150 })),
      ok(makeResponse({ page: 2, total_pages: 3, total: 150 })),
      ok(makeResponse({ page: 1, total_pages: 1, total: 2 }))
    );
    render(<Logs />);

    await waitFor(() => expect(screen.getByRole('button', { name: /next/i })).not.toBeDisabled());
    fireEvent.click(screen.getByRole('button', { name: /next/i }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));

    const [levelTrigger] = screen.getAllByRole('combobox');
    await user.click(levelTrigger);
    await user.click(await screen.findByRole('option', { name: 'Warn' }));

    await waitFor(() => expect(fetchMock).toHaveBeenLastCalledWith('/api/logs?page=1&page_size=50&level=warn'));
  });

  // ── Page size filter ───────────────────────────────────────────────────────

  it('updates the page_size param when a different page size is selected', async () => {
    const user = userEvent.setup();
    const fetchMock = stubFetch(ok(makeResponse()), ok(makeResponse()));
    render(<Logs />);

    await waitFor(() => expect(screen.getByText('Server started')).toBeInTheDocument());

    // The second combobox is the "Per page" select
    const [, pageSizeTrigger] = screen.getAllByRole('combobox');
    await user.click(pageSizeTrigger);

    const option25 = await screen.findByRole('option', { name: '25' });
    await user.click(option25);

    await waitFor(() => expect(fetchMock).toHaveBeenLastCalledWith('/api/logs?page=1&page_size=25'));
  });

  it('resets to page 1 when the page size changes', async () => {
    const user = userEvent.setup();
    const fetchMock = stubFetch(
      ok(makeResponse({ page: 1, total_pages: 5, total: 250 })),
      ok(makeResponse({ page: 2, total_pages: 5, total: 250 })),
      ok(makeResponse({ page: 1, total_pages: 3, total: 250 }))
    );
    render(<Logs />);

    await waitFor(() => expect(screen.getByRole('button', { name: /next/i })).not.toBeDisabled());
    fireEvent.click(screen.getByRole('button', { name: /next/i }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));

    const [, pageSizeTrigger] = screen.getAllByRole('combobox');
    await user.click(pageSizeTrigger);

    const option100 = await screen.findByRole('option', { name: '100' });
    await user.click(option100);

    await waitFor(() => expect(fetchMock).toHaveBeenLastCalledWith('/api/logs?page=1&page_size=100'));
  });
});
