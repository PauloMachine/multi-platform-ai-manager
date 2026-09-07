use crate::commands::build_tray_title;
use crate::models::AppConfig;
use crate::providers::{cached_provider_usage, invalidate_provider_cache};
use crate::storage::{process_incoming_events, save_config};
use crate::storage::APP_DISPLAY_NAME;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open_i = MenuItem::with_id(app, "open", "Open Dashboard", true, None::<&str>)?;
    let refresh_i = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&open_i, &refresh_i, &settings_i, &sep, &quit_i])?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip(APP_DISPLAY_NAME)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
            "refresh" => {
                invalidate_provider_cache();
                let config = app
                    .try_state::<crate::commands::AppState>()
                    .map(|s| s.config.lock().clone())
                    .unwrap_or_default();
                let state = crate::commands::build_dashboard_state(&config, true);
                let _ = app.emit("dashboard-updated", &state);
                update_tray_title(app);
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    let _ = app.emit("navigate", "/settings");
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    update_tray_title(app);
    Ok(())
}

pub fn update_tray_title(app: &AppHandle) {
    let providers = cached_provider_usage(false);
    let title = build_tray_title(&providers);
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(&title));
    }
}

pub fn start_background_tasks(app: AppHandle, config: Arc<Mutex<AppConfig>>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let processed = process_incoming_events();
            if processed > 0 {
                let cfg = config.lock().clone();
                let state = crate::commands::build_dashboard_state(&cfg, false);
                let _ = app.emit("dashboard-updated", &state);
                update_tray_title(&app);
                check_notifications(&app, &cfg).await;
            }

            let interval_secs = config.lock().refresh_interval_secs;
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;

            invalidate_provider_cache();
            let cfg = config.lock().clone();
            let state = crate::commands::build_dashboard_state(&cfg, true);
            let _ = app.emit("dashboard-updated", &state);
            update_tray_title(&app);
            check_notifications(&app, &cfg).await;
        }
    });
}

async fn check_notifications(app: &AppHandle, config: &AppConfig) {
    if !config.desktop_notifications {
        return;
    }

    let providers = cached_provider_usage(false);
    let mut config = config.clone();
    let mut changed = false;

    for provider in &providers {
        if let Some(window) = &provider.five_hour {
            if let Some(pct) = window.percentage {
                for threshold in [80.0, 90.0] {
                    if pct >= threshold {
                        let key = format!("{}-{}", provider.provider, threshold as u32);
                        if !config.notification_thresholds_sent.get(&key).copied().unwrap_or(false) {
                            send_notification(
                                app,
                                &format!("{} usage at {:.0}%", provider.name, pct),
                                &format!("You've reached {:.0}% of your 5-hour limit.", pct),
                            );
                            config.notification_thresholds_sent.insert(key, true);
                            changed = true;
                        }
                    }
                }
                if pct < 75.0 {
                    for threshold in [80, 90] {
                        let key = format!("{}-{}", provider.provider, threshold);
                        if config.notification_thresholds_sent.remove(&key).is_some() {
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    if changed {
        let _ = save_config(&config);
    }
}

fn send_notification(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}
