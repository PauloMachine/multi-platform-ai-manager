mod cli_env;
mod commands;
mod models;
mod providers;
mod session_handoff;
mod storage;
mod tray;

use commands::AppState;
use parking_lot::Mutex;
use std::sync::Arc;
use storage::{ensure_app_dir, load_config, rebuild_sessions_from_events};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    ensure_app_dir().ok();
    let _ = rebuild_sessions_from_events();
    let config = load_config();
    let config_arc = Arc::new(Mutex::new(config.clone()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            config: Mutex::new(config),
        })
        .setup(move |app| {
            providers::install_all_hooks();

            if let Err(e) = tray::setup_tray(app.handle()) {
                eprintln!("Failed to setup tray: {e}");
            }

            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                let config_clone = config_arc.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        if config_clone.lock().minimize_to_tray {
                            api.prevent_close();
                            if let Some(w) = app_handle.get_webview_window("main") {
                                let _ = w.hide();
                            }
                        }
                    }
                });
            }

            tray::start_background_tasks(app.handle().clone(), config_arc);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_config,
            commands::complete_onboarding,
            commands::get_dashboard_state,
            commands::get_provider_usage,
            commands::get_providers_auth,
            commands::connect_provider_cmd,
            commands::install_provider_hooks,
            commands::install_all_provider_hooks,
            commands::refresh_usage,
            commands::get_sessions_cmd,
            commands::get_session_cmd,
            commands::get_model_usage,
            commands::get_tray_snapshot,
            commands::show_main_window,
            commands::hide_main_window,
            commands::quit_app,
            commands::run_session_on_provider_cmd,
            commands::stop_session_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
