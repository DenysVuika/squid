import { create } from 'zustand';
import { fetchConfig, type ConfigResponse } from '@/lib/chat-api';

interface ConfigState {
  // State
  ragEnabled: boolean;
  apiUrl: string;
  apiModel: string;
  contextWindow: number;
  isLoading: boolean;
  isLoaded: boolean;

  // Actions
  loadConfig: () => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set) => ({
  // Initial state
  ragEnabled: false, // default to false to prevent flickering until loaded
  apiUrl: '',
  apiModel: '',
  contextWindow: 0,
  isLoading: false,
  isLoaded: false,

  // Load configuration from API
  loadConfig: async () => {
    set({ isLoading: true });
    try {
      const config: ConfigResponse = await fetchConfig('');
      set({
        ragEnabled: config.rag_enabled,
        apiUrl: config.api_url,
        apiModel: config.api_model,
        contextWindow: config.context_window,
        isLoading: false,
        isLoaded: true,
      });
      console.log('📋 Configuration loaded:', {
        ragEnabled: config.rag_enabled,
        apiModel: config.api_model,
        contextWindow: config.context_window,
      });
    } catch (error) {
      console.error('Failed to load config:', error);
      set({
        isLoading: false,
        isLoaded: true,
        ragEnabled: false, // default to false on error
      });
    }
  },
}));
