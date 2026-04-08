import { create } from 'zustand';
import { fetchJobs, subscribeToJobUpdates, type JobInfo } from '@/lib/chat-api';
import { toast } from 'sonner';

interface JobStore {
  // State
  jobs: JobInfo[];
  selectedJob: number | null;
  isLoading: boolean;
  sseConnection: EventSource | null;
  pollIntervalId: number | null;

  // Actions
  loadJobs: () => Promise<void>;
  setSelectedJob: (jobId: number | null) => void;
  updateJob: (job: JobInfo) => void;
  removeJob: (jobId: number) => void;
  startSSE: () => void;
  stopSSE: () => void;
}

export const useJobStore = create<JobStore>((set, get) => ({
  // Initial state
  jobs: [],
  selectedJob: null,
  isLoading: false,
  sseConnection: null,
  pollIntervalId: null,

  // Load jobs from API
  loadJobs: async () => {
    set({ isLoading: true });
    try {
      const jobs = await fetchJobs('');
      set({ jobs, isLoading: false });
    } catch (error) {
      console.error('Failed to load jobs:', error);
      toast.error('Failed to load jobs', {
        description: 'Could not reach the jobs API.',
        duration: 3000,
      });
      set({ isLoading: false });
    }
  },

  // Set selected job
  setSelectedJob: (jobId: number | null) => {
    set({ selectedJob: jobId });
  },

  // Update a job in the store (from SSE)
  updateJob: (updatedJob: JobInfo) => {
    set((state) => {
      const existingIndex = state.jobs.findIndex((j) => j.id === updatedJob.id);
      if (existingIndex >= 0) {
        // Update existing job
        const newJobs = [...state.jobs];
        newJobs[existingIndex] = updatedJob;
        return { jobs: newJobs };
      } else {
        // Add new job
        return { jobs: [...state.jobs, updatedJob] };
      }
    });
  },

  // Remove a job from the store
  removeJob: (jobId: number) => {
    set((state) => ({
      jobs: state.jobs.filter((j) => j.id !== jobId),
      selectedJob: state.selectedJob === jobId ? null : state.selectedJob,
    }));
  },

  // Start SSE connection for live updates
  startSSE: () => {
    const { sseConnection, pollIntervalId, updateJob, removeJob, loadJobs } = get();

    // Don't create duplicate connections
    if (sseConnection) {
      console.log('SSE connection already active');
      return;
    }

    console.log('Starting SSE connection for jobs...');
    const eventSource = subscribeToJobUpdates('', {
      onJobUpdate: (job) => {
        updateJob(job);
      },
      onJobDeleted: (jobId) => {
        removeJob(jobId);
      },
      onError: (error) => {
        console.error('Job SSE error:', error);
        // Reconnect after 5 seconds
        setTimeout(() => {
          const store = get();
          if (store.sseConnection) {
            store.sseConnection.close();
            set({ sseConnection: null });
            store.startSSE();
          }
        }, 5000);
      },
    });

    set({ sseConnection: eventSource });

    // Start periodic polling to catch CLI-created jobs
    // Poll every 30 seconds to refresh the job list
    if (!pollIntervalId) {
      const intervalId = window.setInterval(() => {
        console.log('[Jobs] Periodic refresh...');
        void loadJobs();
      }, 30000); // 30 seconds

      set({ pollIntervalId: intervalId });
    }
  },

  // Stop SSE connection
  stopSSE: () => {
    const { sseConnection, pollIntervalId } = get();

    if (sseConnection) {
      console.log('Stopping SSE connection for jobs');
      sseConnection.close();
      set({ sseConnection: null });
    }

    if (pollIntervalId) {
      console.log('Stopping job polling interval');
      window.clearInterval(pollIntervalId);
      set({ pollIntervalId: null });
    }
  },
}));
