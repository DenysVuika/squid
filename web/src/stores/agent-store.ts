import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { fetchAgents, fetchConfig, type AgentInfo, type TokenUsage } from '@/lib/chat-api';
import { toast } from 'sonner';

interface AgentStore {
  // State
  agents: AgentInfo[];
  agentGroups: string[];
  selectedAgent: string;
  sessionAgentId: string | null;
  tokenUsage: TokenUsage;
  isLoading: boolean;
  agentSelectorOpen: boolean;

  // Actions
  loadAgents: () => Promise<void>;
  setSelectedAgent: (agentId: string) => void;
  setSessionAgentId: (agentId: string | null) => void;
  updateTokenUsage: (usage: Partial<TokenUsage>) => void;
  resetTokenUsage: () => void;
  setAgentSelectorOpen: (open: boolean) => void;
  getAgentModelForPricing: () => string;
}

const initialTokenUsage: TokenUsage = {
  total_tokens: 0,
  input_tokens: 0,
  output_tokens: 0,
  reasoning_tokens: 0,
  cache_tokens: 0,
  context_window: 0,
  context_utilization: 0,
};

export const useAgentStore = create<AgentStore>()(
  persist(
    (set, get) => ({
      // Initial state
      agents: [],
      agentGroups: [],
      selectedAgent: '',
      sessionAgentId: null,
      tokenUsage: initialTokenUsage,
      isLoading: false,
      agentSelectorOpen: false,

      // Load available agents from API
      loadAgents: async () => {
        const currentSelectedAgent = get().selectedAgent;
        set({ isLoading: true });
        try {
          // Fetch both agents and config in parallel
          const [{ agents: fetchedAgents, default_agent }] = await Promise.all([
            fetchAgents(''),
            fetchConfig(''),
          ]);

          if (fetchedAgents.length > 0) {
            // Extract unique providers (from model strings, e.g., "anthropic/claude" -> "anthropic")
            const providers = Array.from(
              new Set(
                fetchedAgents
                  .map((a) => {
                    const parts = a.model.split('/');
                    return parts.length > 1 ? parts[0] : 'local';
                  })
                  .filter(Boolean)
              )
            ).sort();

            // Check if current selection is still valid
            const isCurrentAgentValid =
              currentSelectedAgent && fetchedAgents.some((a) => a.id === currentSelectedAgent);

            // Use persisted agent if valid, else backend's default agent, or fallback to first available
            const agentToSelect = isCurrentAgentValid
              ? currentSelectedAgent
              : (fetchedAgents.find((a) => a.id === default_agent) || fetchedAgents[0])?.id || '';

            set({
              agents: fetchedAgents,
              agentGroups: providers,
              selectedAgent: agentToSelect,
              isLoading: false,
            });

            if (agentToSelect && !isCurrentAgentValid) {
              const defaultAgent = fetchedAgents.find((a) => a.id === agentToSelect);
              if (defaultAgent) {
                console.log(`🤖 Default agent: ${defaultAgent.name} (${defaultAgent.id})`);
              }
            } else if (isCurrentAgentValid) {
              console.log(`🤖 Restored persisted agent: ${currentSelectedAgent}`);
            }
          } else {
            // No agents available - show warning
            console.warn('⚠️ No agents available. Make sure backend is configured.');
            toast.error('No agents available', {
              description: 'Check your squid.config.json agents configuration.',
              duration: 5000,
            });
            set({ isLoading: false });
          }
        } catch (error) {
          console.error('Failed to load agents:', error);
          toast.error('Failed to connect to API', {
            description: 'Could not reach the agents API. Check if backend is running.',
            duration: 5000,
          });
          set({ isLoading: false });
        }
      },

      // Set selected agent
      setSelectedAgent: (agentId: string) => {
        const { agents } = get();
        const selectedAgentData = agents.find((a) => a.id === agentId);

        set({
          selectedAgent: agentId,
          agentSelectorOpen: false,
        });

        // Note: context window is now tied to the agent's model
        // We could fetch model info if needed, but for now we'll rely on usage updates
        if (selectedAgentData) {
          console.log(`🤖 Selected agent: ${selectedAgentData.name} (model: ${selectedAgentData.model})`);
        }
      },

      // Set session agent ID (for loaded sessions)
      setSessionAgentId: (agentId: string | null) => {
        set({ sessionAgentId: agentId });
      },

      // Update token usage (partial update)
      updateTokenUsage: (usage: Partial<TokenUsage>) => {
        set((state) => ({
          tokenUsage: {
            ...state.tokenUsage,
            ...usage,
          },
        }));
      },

      // Reset token usage to initial state
      resetTokenUsage: () => {
        set({ tokenUsage: initialTokenUsage });
      },

      // Set agent selector open state
      setAgentSelectorOpen: (open: boolean) => {
        set({ agentSelectorOpen: open });
      },

      // Get model ID for pricing calculations (from current agent's model)
      getAgentModelForPricing: () => {
        const { agents, selectedAgent, sessionAgentId } = get();
        const currentAgentId = sessionAgentId || selectedAgent;

        if (!currentAgentId) {
          return 'gpt-4o';
        }

        const agentData = agents.find((a) => a.id === currentAgentId);
        return agentData?.model || currentAgentId;
      },
    }),
    {
      name: 'agent-storage',
      partialize: (state) => ({
        selectedAgent: state.selectedAgent,
      }),
    }
  )
);
