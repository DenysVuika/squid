import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useConfigStore } from './config-store';
import { fetchConfig } from '@/lib/chat-api';

vi.mock('@/lib/chat-api', () => ({
  fetchConfig: vi.fn(),
}));

// ─── Fixtures ────────────────────────────────────────────────────────────────

const INITIAL_STATE = {
  ragEnabled: false,
  webSounds: true,
  apiUrl: '',
  contextWindow: 0,
  isLoading: false,
  isLoaded: false,
};

const MOCK_CONFIG = {
  api_url: 'http://localhost:8080',
  context_window: 8192,
  rag_enabled: true,
  web_sounds: true,
  audio_enabled: false,
};

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('useConfigStore', () => {
  beforeEach(() => {
    // Merge initial values back in – preserves the loadConfig action
    useConfigStore.setState(INITIAL_STATE);
    vi.clearAllMocks();
    // Suppress expected console output so test output stays clean
    vi.spyOn(console, 'log').mockImplementation(() => {});
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  // ── Initial state ──────────────────────────────────────────────────────────

  it('has the correct initial state', () => {
    const { ragEnabled, apiUrl, contextWindow, isLoading, isLoaded } =
      useConfigStore.getState();

    expect(ragEnabled).toBe(false);
    expect(apiUrl).toBe('');
    expect(contextWindow).toBe(0);
    expect(isLoading).toBe(false);
    expect(isLoaded).toBe(false);
  });

  // ── In-flight ─────────────────────────────────────────────────────────────

  it('sets isLoading=true while the request is in-flight', () => {
    // A promise that never resolves lets us inspect the intermediate state
    vi.mocked(fetchConfig).mockReturnValueOnce(new Promise(() => {}));

    useConfigStore.getState().loadConfig();

    expect(useConfigStore.getState().isLoading).toBe(true);
  });

  it('does not set isLoaded while still loading', () => {
    vi.mocked(fetchConfig).mockReturnValueOnce(new Promise(() => {}));

    useConfigStore.getState().loadConfig();

    expect(useConfigStore.getState().isLoaded).toBe(false);
  });

  // ── Success ───────────────────────────────────────────────────────────────

  it('maps all config fields into state on success', async () => {
    vi.mocked(fetchConfig).mockResolvedValueOnce(MOCK_CONFIG);

    await useConfigStore.getState().loadConfig();

    const { ragEnabled, apiUrl, contextWindow } = useConfigStore.getState();
    expect(ragEnabled).toBe(true);
    expect(apiUrl).toBe('http://localhost:8080');
    expect(contextWindow).toBe(8192);
  });

  it('clears isLoading and sets isLoaded=true on success', async () => {
    vi.mocked(fetchConfig).mockResolvedValueOnce(MOCK_CONFIG);

    await useConfigStore.getState().loadConfig();

    const { isLoading, isLoaded } = useConfigStore.getState();
    expect(isLoading).toBe(false);
    expect(isLoaded).toBe(true);
  });

  it('correctly stores ragEnabled=false when the API returns false', async () => {
    vi.mocked(fetchConfig).mockResolvedValueOnce({ ...MOCK_CONFIG, rag_enabled: false });

    await useConfigStore.getState().loadConfig();

    expect(useConfigStore.getState().ragEnabled).toBe(false);
  });

  it('calls fetchConfig exactly once with an empty string', async () => {
    vi.mocked(fetchConfig).mockResolvedValueOnce(MOCK_CONFIG);

    await useConfigStore.getState().loadConfig();

    expect(vi.mocked(fetchConfig)).toHaveBeenCalledOnce();
    expect(vi.mocked(fetchConfig)).toHaveBeenCalledWith('');
  });

  it('logs the loaded config summary to the console', async () => {
    vi.mocked(fetchConfig).mockResolvedValueOnce(MOCK_CONFIG);
    const consoleSpy = vi.spyOn(console, 'log');

    await useConfigStore.getState().loadConfig();

    expect(consoleSpy).toHaveBeenCalledWith(
      '📋 Configuration loaded:',
      expect.objectContaining({
        ragEnabled: true,
        contextWindow: 8192,
      }),
    );
  });

  // ── Failure ───────────────────────────────────────────────────────────────

  it('clears isLoading and sets isLoaded=true on error', async () => {
    vi.mocked(fetchConfig).mockRejectedValueOnce(new Error('Network error'));

    await useConfigStore.getState().loadConfig();

    const { isLoading, isLoaded } = useConfigStore.getState();
    expect(isLoading).toBe(false);
    expect(isLoaded).toBe(true);
  });

  it('forces ragEnabled=false on error', async () => {
    // Seed ragEnabled=true so we can confirm the error handler resets it
    useConfigStore.setState({ ragEnabled: true });
    vi.mocked(fetchConfig).mockRejectedValueOnce(new Error('Network error'));

    await useConfigStore.getState().loadConfig();

    expect(useConfigStore.getState().ragEnabled).toBe(false);
  });

  it('does not overwrite apiUrl or contextWindow on error', async () => {
    // Load a successful config first
    vi.mocked(fetchConfig).mockResolvedValueOnce(MOCK_CONFIG);
    await useConfigStore.getState().loadConfig();

    // Subsequent call fails
    vi.mocked(fetchConfig).mockRejectedValueOnce(new Error('Network error'));
    await useConfigStore.getState().loadConfig();

    const { apiUrl, contextWindow } = useConfigStore.getState();
    expect(apiUrl).toBe('http://localhost:8080');
    expect(contextWindow).toBe(8192);
  });

  it('logs the error to the console', async () => {
    const err = new Error('Something broke');
    vi.mocked(fetchConfig).mockRejectedValueOnce(err);
    const consoleSpy = vi.spyOn(console, 'error');

    await useConfigStore.getState().loadConfig();

    expect(consoleSpy).toHaveBeenCalledWith('Failed to load config:', err);
  });

  // ── Consecutive calls ─────────────────────────────────────────────────────

  it('reflects the latest config after being called multiple times', async () => {
    vi.mocked(fetchConfig)
      .mockResolvedValueOnce(MOCK_CONFIG)
      .mockResolvedValueOnce({ ...MOCK_CONFIG, context_window: 4096 });

    await useConfigStore.getState().loadConfig();
    expect(useConfigStore.getState().contextWindow).toBe(8192);

    await useConfigStore.getState().loadConfig();
    expect(useConfigStore.getState().contextWindow).toBe(4096);
  });

  it('recovers to a loaded state after a failure is followed by a success', async () => {
    vi.mocked(fetchConfig)
      .mockRejectedValueOnce(new Error('First attempt failed'))
      .mockResolvedValueOnce(MOCK_CONFIG);

    await useConfigStore.getState().loadConfig();
    expect(useConfigStore.getState().isLoaded).toBe(true);
    expect(useConfigStore.getState().ragEnabled).toBe(false);

    await useConfigStore.getState().loadConfig();
    expect(useConfigStore.getState().ragEnabled).toBe(true);
  });
});
