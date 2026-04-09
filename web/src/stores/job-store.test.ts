import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useJobStore } from './job-store';
import { fetchJobs, subscribeToJobUpdates, type JobInfo } from '@/lib/chat-api';
import { toast } from 'sonner';

vi.mock('@/lib/chat-api', () => ({
  fetchJobs: vi.fn(),
  subscribeToJobUpdates: vi.fn(),
}));

vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// ─── Fixtures ────────────────────────────────────────────────────────────────

const makeJob = (overrides: Partial<JobInfo> = {}): JobInfo => ({
  id: 1,
  name: 'Test Job',
  schedule_type: 'cron',
  cron_expression: '0 9 * * *',
  status: 'pending',
  priority: 0,
  is_active: true,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  last_run: null,
  next_run: '2024-01-02T09:00:00Z',
  run_count: 0,
  payload: {
    agent_id: 'test-agent',
    message: 'Test message',
  },
  ...overrides,
});

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('useJobStore', () => {
  beforeEach(() => {
    useJobStore.setState({
      jobs: [],
      selectedJob: null,
      isLoading: false,
      sseConnection: null,
      pollIntervalId: null,
    });
    vi.clearAllMocks();
  });

  afterEach(() => {
    // Clean up any intervals
    const state = useJobStore.getState();
    if (state.pollIntervalId) {
      window.clearInterval(state.pollIntervalId);
    }
    if (state.sseConnection) {
      state.sseConnection.close();
    }
  });

  // ── loadJobs ───────────────────────────────────────────────────────────────

  describe('loadJobs', () => {
    it('loads jobs from API and updates state', async () => {
      const jobs = [makeJob({ id: 1 }), makeJob({ id: 2, name: 'Job 2' })];
      vi.mocked(fetchJobs).mockResolvedValue(jobs);

      await useJobStore.getState().loadJobs();

      expect(fetchJobs).toHaveBeenCalledWith('');
      expect(useJobStore.getState().jobs).toEqual(jobs);
      expect(useJobStore.getState().isLoading).toBe(false);
    });

    it('sets isLoading to true during loading', async () => {
      vi.mocked(fetchJobs).mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve([]), 100))
      );

      const loadPromise = useJobStore.getState().loadJobs();
      expect(useJobStore.getState().isLoading).toBe(true);

      await loadPromise;
      expect(useJobStore.getState().isLoading).toBe(false);
    });

    it('shows error toast when loading fails', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      vi.mocked(fetchJobs).mockRejectedValue(new Error('Network error'));

      await useJobStore.getState().loadJobs();

      expect(toast.error).toHaveBeenCalledWith('Failed to load jobs', {
        description: 'Could not reach the jobs API.',
        duration: 3000,
      });
      expect(useJobStore.getState().isLoading).toBe(false);

      consoleSpy.mockRestore();
    });
  });

  // ── setSelectedJob ─────────────────────────────────────────────────────────

  describe('setSelectedJob', () => {
    it('updates selectedJob in the store', () => {
      useJobStore.getState().setSelectedJob(5);
      expect(useJobStore.getState().selectedJob).toBe(5);
    });

    it('can set selectedJob to null', () => {
      useJobStore.getState().setSelectedJob(5);
      useJobStore.getState().setSelectedJob(null);
      expect(useJobStore.getState().selectedJob).toBeNull();
    });
  });

  // ── updateJob ──────────────────────────────────────────────────────────────

  describe('updateJob', () => {
    it('updates an existing job in the store', () => {
      const job1 = makeJob({ id: 1, name: 'Job 1' });
      const job2 = makeJob({ id: 2, name: 'Job 2' });
      useJobStore.setState({ jobs: [job1, job2] });

      const updatedJob = makeJob({ id: 1, name: 'Updated Job 1', status: 'running' });
      useJobStore.getState().updateJob(updatedJob);

      const jobs = useJobStore.getState().jobs;
      expect(jobs).toHaveLength(2);
      expect(jobs[0]).toEqual(updatedJob);
      expect(jobs[1]).toEqual(job2);
    });

    it('adds a new job if it does not exist', () => {
      const job1 = makeJob({ id: 1, name: 'Job 1' });
      useJobStore.setState({ jobs: [job1] });

      const newJob = makeJob({ id: 3, name: 'New Job' });
      useJobStore.getState().updateJob(newJob);

      const jobs = useJobStore.getState().jobs;
      expect(jobs).toHaveLength(2);
      expect(jobs[1]).toEqual(newJob);
    });
  });

  // ── removeJob ──────────────────────────────────────────────────────────────

  describe('removeJob', () => {
    it('removes a job from the store', () => {
      const job1 = makeJob({ id: 1, name: 'Job 1' });
      const job2 = makeJob({ id: 2, name: 'Job 2' });
      useJobStore.setState({ jobs: [job1, job2] });

      useJobStore.getState().removeJob(1);

      const jobs = useJobStore.getState().jobs;
      expect(jobs).toHaveLength(1);
      expect(jobs[0]).toEqual(job2);
    });

    it('clears selectedJob if the removed job was selected', () => {
      const job1 = makeJob({ id: 1, name: 'Job 1' });
      useJobStore.setState({ jobs: [job1], selectedJob: 1 });

      useJobStore.getState().removeJob(1);

      expect(useJobStore.getState().selectedJob).toBeNull();
    });

    it('keeps selectedJob if a different job was removed', () => {
      const job1 = makeJob({ id: 1, name: 'Job 1' });
      const job2 = makeJob({ id: 2, name: 'Job 2' });
      useJobStore.setState({ jobs: [job1, job2], selectedJob: 2 });

      useJobStore.getState().removeJob(1);

      expect(useJobStore.getState().selectedJob).toBe(2);
    });
  });

  // ── startSSE ───────────────────────────────────────────────────────────────

  describe('startSSE', () => {
    it('creates an SSE connection', () => {
      const mockEventSource = {
        close: vi.fn(),
      } as unknown as EventSource;

      vi.mocked(subscribeToJobUpdates).mockReturnValue(mockEventSource);

      useJobStore.getState().startSSE();

      expect(subscribeToJobUpdates).toHaveBeenCalledWith('', expect.any(Object));
      expect(useJobStore.getState().sseConnection).toBe(mockEventSource);
    });

    it('does not create duplicate connections', () => {
      const mockEventSource = {
        close: vi.fn(),
      } as unknown as EventSource;

      vi.mocked(subscribeToJobUpdates).mockReturnValue(mockEventSource);

      useJobStore.getState().startSSE();
      useJobStore.getState().startSSE();

      expect(subscribeToJobUpdates).toHaveBeenCalledTimes(1);
    });

    it('starts periodic polling interval', () => {
      const mockEventSource = {
        close: vi.fn(),
      } as unknown as EventSource;

      vi.mocked(subscribeToJobUpdates).mockReturnValue(mockEventSource);

      useJobStore.getState().startSSE();

      expect(useJobStore.getState().pollIntervalId).not.toBeNull();
    });

    it('calls updateJob callback on job update', () => {
      const mockEventSource = {
        close: vi.fn(),
      } as unknown as EventSource;

      let onJobUpdate: ((job: JobInfo) => void) | undefined;
      vi.mocked(subscribeToJobUpdates).mockImplementation((_, callbacks) => {
        onJobUpdate = callbacks.onJobUpdate;
        return mockEventSource;
      });

      useJobStore.getState().startSSE();

      const updatedJob = makeJob({ id: 1, status: 'running' });
      onJobUpdate?.(updatedJob);

      expect(useJobStore.getState().jobs).toContainEqual(updatedJob);
    });

    it('calls removeJob callback on job deleted', () => {
      const mockEventSource = {
        close: vi.fn(),
      } as unknown as EventSource;

      let onJobDeleted: ((jobId: number) => void) | undefined;
      vi.mocked(subscribeToJobUpdates).mockImplementation((_, callbacks) => {
        onJobDeleted = callbacks.onJobDeleted;
        return mockEventSource;
      });

      const job1 = makeJob({ id: 1 });
      useJobStore.setState({ jobs: [job1] });
      useJobStore.getState().startSSE();

      onJobDeleted?.(1);

      expect(useJobStore.getState().jobs).toHaveLength(0);
    });
  });

  // ── stopSSE ────────────────────────────────────────────────────────────────

  describe('stopSSE', () => {
    it('closes SSE connection', () => {
      const mockEventSource = {
        close: vi.fn(),
      } as unknown as EventSource;

      useJobStore.setState({ sseConnection: mockEventSource });

      useJobStore.getState().stopSSE();

      expect(mockEventSource.close).toHaveBeenCalled();
      expect(useJobStore.getState().sseConnection).toBeNull();
    });

    it('clears polling interval', () => {
      const intervalId = window.setInterval(() => {}, 1000);
      useJobStore.setState({ pollIntervalId: intervalId });

      const clearIntervalSpy = vi.spyOn(window, 'clearInterval');

      useJobStore.getState().stopSSE();

      expect(clearIntervalSpy).toHaveBeenCalledWith(intervalId);
      expect(useJobStore.getState().pollIntervalId).toBeNull();
    });

    it('handles missing connection gracefully', () => {
      useJobStore.setState({ sseConnection: null, pollIntervalId: null });

      expect(() => useJobStore.getState().stopSSE()).not.toThrow();
    });
  });
});
