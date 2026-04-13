import { create } from 'zustand';
import { fetchConfig, type ConfigResponse } from '@/lib/chat-api';

interface ConfigState {
  // State
  ragEnabled: boolean;
  webSounds: boolean;
  audioEnabled: boolean;
  apiUrl: string;
  contextWindow: number;
  isLoading: boolean;
  isLoaded: boolean;

  // Actions
  loadConfig: () => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set) => ({
  // Initial state
  ragEnabled: false, // default to false to prevent flickering until loaded
  webSounds: true, // default to true
  audioEnabled: false, // default to false (opt-in)
  apiUrl: '',
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
        webSounds: config.web_sounds,
        audioEnabled: config.audio_enabled,
        apiUrl: config.api_url,
        contextWindow: config.context_window,
        isLoading: false,
        isLoaded: true,
      });
      console.log('📋 Configuration loaded:', {
        ragEnabled: config.rag_enabled,
        webSounds: config.web_sounds,
        audioEnabled: config.audio_enabled,
        contextWindow: config.context_window,
      });
    } catch (error) {
      console.error('Failed to load config:', error);
      set({
        isLoading: false,
        isLoaded: true,
        ragEnabled: false, // default to false on error
        webSounds: true, // default to true on error
        audioEnabled: false, // default to false on error (opt-in)
      });
    }
  },
}));
