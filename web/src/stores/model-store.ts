import { create } from 'zustand';
import { fetchModels, type ModelInfo, type TokenUsage } from '@/lib/chat-api';
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

export const useModelStore = create<ModelStore>((set, get) => ({
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
    set({ isLoading: true });
    try {
      const { models: fetchedModels } = await fetchModels('');

      if (fetchedModels.length > 0) {
        // Extract unique providers and sort them
        const providers = Array.from(new Set(fetchedModels.map((m) => m.provider))).sort();

        // Set default model - prefer Qwen Coder 2.5
        const defaultModel =
          fetchedModels.find((m) => m.id.includes('qwen2.5-coder')) ||
          fetchedModels.find((m) => m.id.includes('qwen') && m.id.includes('coder')) ||
          fetchedModels.find((m) => m.provider === 'Qwen') ||
          fetchedModels[0];

        set({
          models: fetchedModels,
          modelGroups: providers,
          selectedModel: defaultModel?.id || '',
          isLoading: false,
        });

        if (defaultModel) {
          console.log(`🤖 Default model: ${defaultModel.name} (${defaultModel.id})`);
        }
      } else {
        set({ isLoading: false });
      }
    } catch (error) {
      console.error('Failed to load models:', error);
      toast.error('Failed to load models');
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
}));
