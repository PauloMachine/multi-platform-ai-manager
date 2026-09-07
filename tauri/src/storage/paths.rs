use std::fs;
use std::path::PathBuf;

pub const APP_SLUG: &str = "multi-platform-ai-manager";
pub const APP_DISPLAY_NAME: &str = "Multi-platform AI Manager";
pub const LEGACY_APP_SLUG: &str = "ai-usage-monitor";

pub fn app_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(format!(".{APP_SLUG}"))
}

fn migrate_legacy_app_dir() {
    let Some(home) = dirs::home_dir() else {
        return;
    };
    let new_dir = home.join(format!(".{APP_SLUG}"));
    let old_dir = home.join(format!(".{LEGACY_APP_SLUG}"));
    if old_dir.exists() && !new_dir.exists() {
        let _ = fs::rename(old_dir, new_dir);
    }
}

pub fn config_path() -> PathBuf {
    app_dir().join("config.json")
}

pub fn events_path() -> PathBuf {
    app_dir().join("events.ndjson")
}

pub fn incoming_events_path() -> PathBuf {
    app_dir().join("incoming-events.ndjson")
}

pub fn claude_usage_cache_path() -> PathBuf {
    app_dir().join("claude-usage.json")
}

pub fn sessions_cache_path() -> PathBuf {
    app_dir().join("sessions.json")
}

pub fn hooks_dir() -> PathBuf {
    app_dir().join("hooks")
}

pub fn ensure_app_dir() -> std::io::Result<()> {
    migrate_legacy_app_dir();
    fs::create_dir_all(app_dir())?;
    fs::create_dir_all(hooks_dir())?;
    Ok(())
}
