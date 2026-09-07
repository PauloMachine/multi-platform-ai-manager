export type AuthStatus = "connected" | "notconnected" | "notinstalled" | "unknown";

export interface UsageWindow {
  percentage?: number | null;
  resets_at?: number | null;
  label: string;
}

export interface ProviderUsage {
  provider: string;
  name: string;
  installed: boolean;
  auth_status: AuthStatus;
  five_hour?: UsageWindow | null;
  seven_day?: UsageWindow | null;
  usage_available: boolean;
  last_updated?: string | null;
}

export interface ModelUsage {
  model: string;
  percentage?: number | null;
  tokens?: number | null;
  cost?: number | null;
  session_count: number;
}

export type SessionStatus =
  | "running"
  | "completed"
  | "failed"
  | "aborted"
  | "unknown";

export interface AISession {
  id: string;
  provider: string;
  model?: string | null;
  project?: string | null;
  started_at: string;
  ended_at?: string | null;
  duration_secs?: number | null;
  status: SessionStatus;
  initial_prompt?: string | null;
  tokens?: number | null;
  cost?: number | null;
  conversation_id?: string | null;
}

export interface SessionActivity {
  timestamp: string;
  description: string;
  activity_type?: string | null;
}

export interface AISessionDetails {
  session: AISession;
  tools_used: string[];
  files_modified: string[];
  activity: SessionActivity[];
  transcript_path?: string | null;
  error_message?: string | null;
}

export interface AIEvent {
  id: string;
  provider: string;
  session_id: string;
  timestamp: string;
  event_type: string;
  model?: string | null;
  project?: string | null;
  status?: string | null;
  data?: Record<string, unknown> | null;
}

export interface SessionRunResult {
  continuation_prompt: string;
  from_provider: string;
  to_provider: string;
  session_id: string;
}

export interface AppConfig {
  refresh_interval_secs: number;
  start_with_system: boolean;
  minimize_to_tray: boolean;
  desktop_notifications: boolean;
  enabled_providers: string[];
  onboarding_completed: boolean;
  notification_thresholds_sent: Record<string, boolean>;
}

export interface DashboardState {
  providers: ProviderUsage[];
  models: ModelUsage[];
  active_sessions: AISession[];
  recent_events: AIEvent[];
  recent_sessions: AISession[];
}

export interface ProviderAuthStatus {
  provider: string;
  status: AuthStatus;
  message?: string | null;
  version?: string | null;
}

export interface HookInstallResult {
  success: boolean;
  message: string;
  backup_path?: string | null;
}

export interface AIProvider {
  id: string;
  name: string;
}

export const AVAILABLE_PROVIDERS = [
  { id: "claude", name: "Claude Code" },
  { id: "cursor", name: "Cursor" },
  { id: "codex", name: "Codex" },
] as const;

export const REFRESH_INTERVALS = [
  { label: "30 seconds", value: 30 },
  { label: "1 minute", value: 60 },
  { label: "5 minutes", value: 300 },
  { label: "15 minutes", value: 900 },
  { label: "30 minutes", value: 1800 },
] as const;
