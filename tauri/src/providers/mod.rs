mod cache;
mod claude;
mod codex;
mod cursor;

pub use cache::{cached_auth_status, cached_provider_usage, invalidate_provider_cache};
pub use claude::ClaudeProvider;
pub use codex::CodexProvider;
pub use cursor::CursorProvider;

use crate::models::{
    AuthStatus, HookInstallResult, ProviderAuthStatus, ProviderUsage, UsageWindow,
};

pub trait Provider {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn detect(&self) -> bool;
    fn auth_status(&self) -> ProviderAuthStatus;
    fn connect(&self) -> Result<String, String>;
    fn get_usage(&self) -> ProviderUsage;
    fn install_hooks(&self) -> HookInstallResult;
}

fn usage_window(label: &str, percentage: Option<f64>, resets_at: Option<i64>) -> UsageWindow {
    UsageWindow {
        percentage,
        resets_at,
        label: label.to_string(),
    }
}

fn base_usage(
    provider: &str,
    name: &str,
    installed: bool,
    auth: AuthStatus,
    five_hour: Option<UsageWindow>,
    seven_day: Option<UsageWindow>,
    last_updated: Option<String>,
) -> ProviderUsage {
    let usage_available = five_hour
        .as_ref()
        .and_then(|w| w.percentage)
        .is_some()
        || seven_day
            .as_ref()
            .and_then(|w| w.percentage)
            .is_some();
    ProviderUsage {
        provider: provider.to_string(),
        name: name.to_string(),
        installed,
        auth_status: auth,
        five_hour,
        seven_day,
        usage_available,
        last_updated,
    }
}

pub fn all_auth_status() -> Vec<ProviderAuthStatus> {
    cached_auth_status(false)
}

pub fn connect_provider(provider: &str) -> Result<String, String> {
    let result = match provider {
        "claude" => ClaudeProvider::new().connect(),
        "cursor" => CursorProvider::new().connect(),
        "codex" => CodexProvider::new().connect(),
        _ => Err(format!("Unknown provider: {provider}")),
    };
    if result.is_ok() {
        invalidate_provider_cache();
    }
    result
}

pub fn install_hooks(provider: &str) -> HookInstallResult {
    match provider {
        "claude" => ClaudeProvider::new().install_hooks(),
        "cursor" => CursorProvider::new().install_hooks(),
        "codex" => CodexProvider::new().install_hooks(),
        _ => HookInstallResult {
            success: false,
            message: format!("Unknown provider: {provider}"),
            backup_path: None,
        },
    }
}

pub fn install_all_hooks() -> Vec<HookInstallResult> {
    vec![
        ClaudeProvider::new().install_hooks(),
        CursorProvider::new().install_hooks(),
        CodexProvider::new().install_hooks(),
    ]
}

pub fn resolve_provider_binary(provider: &str) -> Option<std::path::PathBuf> {
    match provider {
        "claude" => ClaudeProvider::binary_path(),
        "cursor" => CursorProvider::binary_path(),
        "codex" => CodexProvider::binary_path(),
        _ => None,
    }
}
