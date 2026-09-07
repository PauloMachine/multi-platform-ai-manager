use crate::models::AppConfig;
use crate::storage::{config_path, ensure_app_dir};
use std::fs;

pub fn load_config() -> AppConfig {
    ensure_app_dir().ok();
    let path = config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    let config = AppConfig::default();
    save_config(&config).ok();
    config
}

pub fn save_config(config: &AppConfig) -> std::io::Result<()> {
    ensure_app_dir()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path(), content)
}
