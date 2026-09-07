import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppConfig,
  AISession,
  AISessionDetails,
  DashboardState,
  HookInstallResult,
  ModelUsage,
  ProviderAuthStatus,
  ProviderUsage,
} from "../types";

export const api = {
  getConfig: () => invoke<AppConfig>("get_config"),
  updateConfig: (config: AppConfig) => invoke<void>("update_config", { config }),
  completeOnboarding: () => invoke<void>("complete_onboarding"),
  getDashboardState: () => invoke<DashboardState>("get_dashboard_state"),
  getProviderUsage: (provider: string) =>
    invoke<ProviderUsage>("get_provider_usage", { provider }),
  getProvidersAuth: () => invoke<ProviderAuthStatus[]>("get_providers_auth"),
  connectProvider: (provider: string) =>
    invoke<string>("connect_provider_cmd", { provider }),
  installProviderHooks: (provider: string) =>
    invoke<HookInstallResult>("install_provider_hooks", { provider }),
  installAllProviderHooks: () =>
    invoke<HookInstallResult[]>("install_all_provider_hooks"),
  refreshUsage: () => invoke<DashboardState>("refresh_usage"),
  getSessions: (provider?: string, model?: string) =>
    invoke<AISession[]>("get_sessions_cmd", { provider, model }),
  getSession: (id: string) => invoke<AISessionDetails | null>("get_session_cmd", { id }),
  getModelUsage: (provider?: string) =>
    invoke<ModelUsage[]>("get_model_usage", { provider }),
  showMainWindow: () => invoke<void>("show_main_window"),
  hideMainWindow: () => invoke<void>("hide_main_window"),
  quitApp: () => invoke<void>("quit_app"),
  runSessionOnProvider: (sessionId: string, toProvider: string) =>
    invoke<import("../types").SessionRunResult>("run_session_on_provider_cmd", {
      sessionId,
      toProvider,
    }),
  stopSession: (sessionId: string) =>
    invoke<void>("stop_session_cmd", { sessionId }),
};

export function onDashboardUpdated(cb: (state: DashboardState) => void) {
  return listen<DashboardState>("dashboard-updated", (event) => {
    cb(event.payload);
  });
}

export function onNavigate(cb: (path: string) => void) {
  return listen<string>("navigate", (event) => {
    cb(event.payload);
  });
}
