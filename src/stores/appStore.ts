import { create } from "zustand";
import { api, onDashboardUpdated } from "../lib/tauri";
import type {
  AppConfig,
  DashboardState,
  ProviderAuthStatus,
} from "../types";

interface AppStore {
  config: AppConfig | null;
  dashboard: DashboardState | null;
  auth: ProviderAuthStatus[];
  loading: boolean;
  refreshing: boolean;
  error: string | null;
  init: () => Promise<void>;
  loadDashboard: () => Promise<void>;
  refresh: () => Promise<void>;
  loadConfig: () => Promise<void>;
  updateConfig: (config: AppConfig) => Promise<void>;
  loadAuth: () => Promise<void>;
  connectProvider: (provider: string) => Promise<string>;
}

export const useAppStore = create<AppStore>((set, get) => ({
  config: null,
  dashboard: null,
  auth: [],
  loading: true,
  refreshing: false,
  error: null,

  init: async () => {
    try {
      await get().loadConfig();
      await get().loadAuth();
      await get().loadDashboard();
      onDashboardUpdated((state) => set({ dashboard: state }));
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  loadDashboard: async () => {
    const dashboard = await api.getDashboardState();
    set({ dashboard });
  },

  refresh: async () => {
    set({ refreshing: true });
    try {
      const dashboard = await api.refreshUsage();
      set({ dashboard });
    } finally {
      set({ refreshing: false });
    }
  },

  loadConfig: async () => {
    const config = await api.getConfig();
    set({ config });
  },

  updateConfig: async (config) => {
    await api.updateConfig(config);
    set({ config });
  },

  loadAuth: async () => {
    const auth = await api.getProvidersAuth();
    set({ auth });
  },

  connectProvider: async (provider) => {
    const result = await api.connectProvider(provider);
    await get().loadAuth();
    await get().refresh();
    return result;
  },
}));
