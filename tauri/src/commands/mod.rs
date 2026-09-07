use crate::models::{
    AppConfig, DashboardState, HookInstallResult, ProviderAuthStatus, ProviderUsage,
    TraySnapshot,
};
use crate::providers::{
    all_auth_status, cached_provider_usage, connect_provider, install_all_hooks, install_hooks,
    invalidate_provider_cache,
};
use crate::storage::{
    compute_model_usage, force_stop_session, get_active_sessions, get_session, get_sessions,
    process_incoming_events, read_events, save_config, APP_DISPLAY_NAME,
};
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

pub fn build_dashboard_state(_config: &AppConfig, force_providers: bool) -> DashboardState {
    process_incoming_events();
    let providers = cached_provider_usage(force_providers);
    let models = compute_model_usage(None);
    let active_sessions = get_active_sessions();
    let recent_events = read_events(Some(50));
    let recent_sessions = get_sessions(None, None)
        .into_iter()
        .take(20)
        .collect();

    DashboardState {
        providers,
        models,
        active_sessions,
        recent_events,
        recent_sessions,
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> AppConfig {
    state.config.lock().clone()
}

#[tauri::command]
pub fn update_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    save_config(&config).map_err(|e| e.to_string())?;
    *state.config.lock() = config;
    Ok(())
}

#[tauri::command]
pub fn complete_onboarding(state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.lock().clone();
    config.onboarding_completed = true;
    save_config(&config).map_err(|e| e.to_string())?;
    *state.config.lock() = config;
    Ok(())
}

#[tauri::command]
pub fn get_dashboard_state(state: State<'_, AppState>) -> DashboardState {
    let config = state.config.lock().clone();
    build_dashboard_state(&config, false)
}

#[tauri::command]
pub fn get_provider_usage(provider: String) -> ProviderUsage {
    process_incoming_events();
    cached_provider_usage(false)
        .into_iter()
        .find(|p| p.provider == provider)
        .unwrap_or_else(|| cached_provider_usage(true)[0].clone())
}

#[tauri::command]
pub fn get_providers_auth() -> Vec<ProviderAuthStatus> {
    all_auth_status()
}

#[tauri::command]
pub fn connect_provider_cmd(provider: String) -> Result<String, String> {
    connect_provider(&provider)
}

#[tauri::command]
pub fn install_provider_hooks(provider: String) -> HookInstallResult {
    install_hooks(&provider)
}

#[tauri::command]
pub fn install_all_provider_hooks() -> Vec<HookInstallResult> {
    install_all_hooks()
}

#[tauri::command]
pub fn refresh_usage(app: AppHandle, state: State<'_, AppState>) -> DashboardState {
    invalidate_provider_cache();
    let config = state.config.lock().clone();
    let dashboard = build_dashboard_state(&config, true);
    let _ = app.emit("dashboard-updated", &dashboard);
    dashboard
}

#[tauri::command]
pub fn get_sessions_cmd(provider: Option<String>, model: Option<String>) -> Vec<crate::models::AISession> {
    process_incoming_events();
    get_sessions(provider.as_deref(), model.as_deref())
}

#[tauri::command]
pub fn get_session_cmd(id: String) -> Option<crate::models::AISessionDetails> {
    process_incoming_events();
    get_session(&id)
}

#[tauri::command]
pub fn get_model_usage(provider: Option<String>) -> Vec<crate::models::ModelUsage> {
    process_incoming_events();
    compute_model_usage(provider.as_deref())
}

#[tauri::command]
pub fn get_tray_snapshot() -> TraySnapshot {
    process_incoming_events();
    let providers = cached_provider_usage(false);
    let active_sessions = get_active_sessions();
    let title = build_tray_title(&providers);
    TraySnapshot {
        providers,
        active_sessions,
        title,
    }
}

pub fn build_tray_title(providers: &[ProviderUsage]) -> String {
    let parts: Vec<String> = providers
        .iter()
        .filter_map(|p| {
            let pct = p
                .five_hour
                .as_ref()
                .and_then(|w| w.percentage)
                .map(|v| format!("{:.0}%", v))?;
            Some(format!("{} {}", p.name.split_whitespace().next().unwrap_or(&p.name), pct))
        })
        .collect();

    if parts.is_empty() {
        APP_DISPLAY_NAME.to_string()
    } else if parts.join(" | ").len() > 24 {
        let pcts: Vec<String> = providers
            .iter()
            .filter_map(|p| {
                p.five_hour
                    .as_ref()
                    .and_then(|w| w.percentage)
                    .map(|v| format!("{:.0}%", v))
            })
            .collect();
        if pcts.is_empty() {
            APP_DISPLAY_NAME.to_string()
        } else {
            format!("MPAI {}", pcts.join(" | "))
        }
    } else {
        parts.join(" | ")
    }
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn run_session_on_provider_cmd(
    session_id: String,
    to_provider: String,
) -> Result<crate::models::SessionRunResult, String> {
    process_incoming_events();
    let run = crate::session_handoff::run_session_on_provider(&session_id, &to_provider)?;
    Ok(crate::models::SessionRunResult {
        continuation_prompt: run.continuation_prompt,
        from_provider: run.from_provider,
        to_provider: run.to_provider,
        session_id: run.session_id,
    })
}

#[tauri::command]
pub fn stop_session_cmd(app: AppHandle, state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    force_stop_session(&session_id)?;
    let config = state.config.lock().clone();
    let dashboard = build_dashboard_state(&config, false);
    let _ = app.emit("dashboard-updated", &dashboard);
    Ok(())
}
