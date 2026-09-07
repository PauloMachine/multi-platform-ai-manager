use crate::models::{ProviderAuthStatus, ProviderUsage};
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use parking_lot::Mutex;

use super::{ClaudeProvider, CodexProvider, CursorProvider, Provider};

static CACHE: LazyLock<Mutex<ProviderCache>> = LazyLock::new(|| Mutex::new(ProviderCache::empty()));

const TTL: Duration = Duration::from_secs(45);

struct ProviderCache {
    usage: Vec<ProviderUsage>,
    auth: Vec<ProviderAuthStatus>,
    updated_at: Option<Instant>,
}

impl ProviderCache {
    fn empty() -> Self {
        Self {
            usage: Vec::new(),
            auth: Vec::new(),
            updated_at: None,
        }
    }

    fn is_fresh(&self) -> bool {
        self.updated_at
            .is_some_and(|t| t.elapsed() < TTL && !self.usage.is_empty())
    }

    fn refresh(&mut self) {
        self.usage = vec![
            ClaudeProvider::new().get_usage(),
            CursorProvider::new().get_usage(),
            CodexProvider::new().get_usage(),
        ];
        self.auth = self
            .usage
            .iter()
            .map(|u| ProviderAuthStatus {
                provider: u.provider.clone(),
                status: u.auth_status.clone(),
                message: None,
                version: None,
            })
            .collect();
        self.updated_at = Some(Instant::now());
    }
}

pub fn invalidate_provider_cache() {
    CACHE.lock().updated_at = None;
}

pub fn cached_provider_usage(force_refresh: bool) -> Vec<ProviderUsage> {
    let mut cache = CACHE.lock();
    if force_refresh || !cache.is_fresh() {
        cache.refresh();
    }
    cache.usage.clone()
}

pub fn cached_auth_status(force_refresh: bool) -> Vec<ProviderAuthStatus> {
    let mut cache = CACHE.lock();
    if force_refresh || !cache.is_fresh() {
        cache.refresh();
    }
    cache.auth.clone()
}
