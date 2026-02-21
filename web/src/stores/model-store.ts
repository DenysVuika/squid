import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { fetchModels, fetchConfig, type ModelInfo, type TokenUsage } from '@/lib/chat-api';
import { toast } from 'sonner';

interface ModelStore {
  // State
  models: ModelInfo[];
  modelGroups: string[];
  selectedModel: string;
  sessionModelId: string | null;
  tokenUsage: TokenUsage;
  isLoading: boolean;
  modelSelectorOpen: boolean;

  // Actions
  loadModels: () => Promise<void>;
  setSelectedModel: (modelId: string) => void;
  setSessionModelId: (modelId: string | null) => void;
  updateTokenUsage: (usage: Partial<TokenUsage>) => void;
  resetTokenUsage: () => void;
  setModelSelectorOpen: (open: boolean) => void;
  getModelForPricing: () => string;
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

export const useModelStore = create<ModelStore>()(
  persist(
    (set, get) => ({
      // Initial state
      models: [],
      modelGroups: [],
      selectedModel: '',
      sessionModelId: null,
      tokenUsage: initialTokenUsage,
      isLoading: false,
      modelSelectorOpen: false,

  // Load available models from API
  loadModels: async () => {
    const currentSelectedModel = get().selectedModel;
    set({ isLoading: true });
    try {
      // Fetch both models and config in parallel
      const [{ models: fetchedModels }, config] = await Promise.all([
        fetchModels(''),
        fetchConfig(''),
      ]);

      if (fetchedModels.length > 0) {
        // Extract unique providers and sort them
        const providers = Array.from(new Set(fetchedModels.map((m) => m.provider))).sort();

        // Check if current selection is still valid
        const isCurrentModelValid = currentSelectedModel && fetchedModels.some((m) => m.id === currentSelectedModel);

        // Use persisted model if valid, else backend's default model, or fallback to first available
        const modelToSelect = isCurrentModelValid
          ? currentSelectedModel
          : (fetchedModels.find((m) => m.id === config.api_model) || fetchedModels[0])?.id || '';

        set({
          models: fetchedModels,
          modelGroups: providers,
          selectedModel: modelToSelect,
          isLoading: false,
        });

        if (modelToSelect && !isCurrentModelValid) {
          const defaultModel = fetchedModels.find((m) => m.id === modelToSelect);
          if (defaultModel) {
            console.log(`🤖 Default model: ${defaultModel.name} (${defaultModel.id})`);
          }
        } else if (isCurrentModelValid) {
          console.log(`🤖 Restored persisted model: ${currentSelectedModel}`);
        }
      } else {
        // No models available - show warning
        console.warn('⚠️ No models available. Make sure LM Studio or Ollama is running.');
        toast.error('No models available', {
          description: 'Make sure LM Studio or Ollama is running and accessible.',
          duration: 5000,
        });
        set({ isLoading: false });
      }
    } catch (error) {
      console.error('Failed to load models:', error);
      toast.error('Failed to connect to model provider', {
        description: 'Could not reach the models API. Check if LM Studio or Ollama is running.',
        duration: 5000,
      });
      set({ isLoading: false });
    }
  },

  // Set selected model
  setSelectedModel: (modelId: string) => {
    const { models } = get();
    const selectedModelData = models.find((m) => m.id === modelId);
    
    set({ 
      selectedModel: modelId,
      modelSelectorOpen: false,
    });

    // Update context window when model changes
    if (selectedModelData) {
      set((state) => ({
        tokenUsage: {
          ...state.tokenUsage,
          context_window: selectedModelData.max_context_length,
        },
      }));
    }
  },

  // Set session model ID (for loaded sessions)
  setSessionModelId: (modelId: string | null) => {
    set({ sessionModelId: modelId });
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

  // Set model selector open state
  setModelSelectorOpen: (open: boolean) => {
    set({ modelSelectorOpen: open });
  },

  // Get model ID for pricing calculations
  getModelForPricing: () => {
    const { models, selectedModel, sessionModelId } = get();
    const currentModelId = sessionModelId || selectedModel;

    if (!currentModelId) {
      return 'gpt-4o';
    }

    const modelData = models.find((m) => m.id === currentModelId);
    return modelData?.pricing_model || currentModelId;
  },
    }),
    {
      name: 'model-storage',
      partialize: (state) => ({
        selectedModel: state.selectedModel,
      }),
    }
  )
);
