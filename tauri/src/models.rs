use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthStatus {
    Connected,
    NotConnected,
    NotInstalled,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAuthStatus {
    pub provider: String,
    pub status: AuthStatus,
    pub message: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindow {
    pub percentage: Option<f64>,
    pub resets_at: Option<i64>,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    pub provider: String,
    pub name: String,
    pub installed: bool,
    pub auth_status: AuthStatus,
    pub five_hour: Option<UsageWindow>,
    pub seven_day: Option<UsageWindow>,
    pub usage_available: bool,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    pub percentage: Option<f64>,
    pub tokens: Option<u64>,
    pub cost: Option<f64>,
    pub session_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Running,
    Completed,
    Failed,
    Aborted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISession {
    pub id: String,
    pub provider: String,
    pub model: Option<String>,
    pub project: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_secs: Option<u64>,
    pub status: SessionStatus,
    pub initial_prompt: Option<String>,
    pub tokens: Option<u64>,
    pub cost: Option<f64>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivity {
    pub timestamp: String,
    pub description: String,
    pub activity_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISessionDetails {
    pub session: AISession,
    pub tools_used: Vec<String>,
    pub files_modified: Vec<String>,
    pub activity: Vec<SessionActivity>,
    pub transcript_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEvent {
    pub id: String,
    pub provider: String,
    pub session_id: String,
    pub timestamp: String,
    pub event_type: String,
    pub model: Option<String>,
    pub project: Option<String>,
    pub status: Option<String>,
    pub data: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRunResult {
    pub continuation_prompt: String,
    pub from_provider: String,
    pub to_provider: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub refresh_interval_secs: u64,
    pub start_with_system: bool,
    pub minimize_to_tray: bool,
    pub desktop_notifications: bool,
    pub enabled_providers: Vec<String>,
    pub onboarding_completed: bool,
    pub notification_thresholds_sent: HashMap<String, bool>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_interval_secs: 60,
            start_with_system: false,
            minimize_to_tray: true,
            desktop_notifications: true,
            enabled_providers: vec!["claude".into(), "cursor".into(), "codex".into()],
            onboarding_completed: false,
            notification_thresholds_sent: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    pub providers: Vec<ProviderUsage>,
    pub models: Vec<ModelUsage>,
    pub active_sessions: Vec<AISession>,
    pub recent_events: Vec<AIEvent>,
    pub recent_sessions: Vec<AISession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraySnapshot {
    pub providers: Vec<ProviderUsage>,
    pub active_sessions: Vec<AISession>,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeUsageCache {
    pub five_hour_percentage: Option<f64>,
    pub seven_day_percentage: Option<f64>,
    pub five_hour_resets_at: Option<i64>,
    pub seven_day_resets_at: Option<i64>,
    pub updated_at: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInstallResult {
    pub success: bool,
    pub message: String,
    pub backup_path: Option<String>,
}
