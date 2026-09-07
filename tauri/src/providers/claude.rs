use super::{base_usage, usage_window, Provider};
use crate::cli_env::command;
use crate::models::{
    AuthStatus, ClaudeUsageCache, HookInstallResult, ProviderAuthStatus, ProviderUsage,
};
use crate::storage::{claude_usage_cache_path, ensure_app_dir, hooks_dir, APP_SLUG};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const HOOK_MARKER: &str = APP_SLUG;

pub struct ClaudeProvider;

impl ClaudeProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn binary_path() -> Option<PathBuf> {
        Self::claude_path()
    }

    fn claude_path() -> Option<PathBuf> {
        let candidates = [
            dirs::home_dir().map(|h| h.join(".local/bin/claude")),
            Some(PathBuf::from("/usr/local/bin/claude")),
            Some(PathBuf::from("/opt/homebrew/bin/claude")),
        ];
        for path in candidates.into_iter().flatten() {
            if path.exists() {
                return Some(path);
            }
        }
        which_claude()
    }

    fn version(&self) -> Option<String> {
        let path = Self::claude_path()?;
        command(&path)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }
}

impl Provider for ClaudeProvider {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn name(&self) -> &'static str {
        "Claude Code"
    }

    fn detect(&self) -> bool {
        Self::claude_path().is_some()
    }

    fn auth_status(&self) -> ProviderAuthStatus {
        if !self.detect() {
            return ProviderAuthStatus {
                provider: self.id().to_string(),
                status: AuthStatus::NotInstalled,
                message: Some(
                    "Claude Code CLI not found. Install from https://code.claude.com".into(),
                ),
                version: None,
            };
        }

        let path = Self::claude_path().unwrap();
        let version = self.version();
        let output = command(&path).arg("auth").arg("status").output();

        match output {
            Ok(o) if o.status.success() => ProviderAuthStatus {
                provider: self.id().to_string(),
                status: AuthStatus::Connected,
                message: Some(String::from_utf8_lossy(&o.stdout).trim().to_string()),
                version,
            },
            Ok(o) => ProviderAuthStatus {
                provider: self.id().to_string(),
                status: AuthStatus::NotConnected,
                message: Some(String::from_utf8_lossy(&o.stderr).trim().to_string()),
                version,
            },
            Err(e) => ProviderAuthStatus {
                provider: self.id().to_string(),
                status: AuthStatus::Unknown,
                message: Some(e.to_string()),
                version,
            },
        }
    }

    fn connect(&self) -> Result<String, String> {
        if !self.detect() {
            return Err("Claude Code is not installed".into());
        }
        let path = Self::claude_path().unwrap();
        let status = command(&path)
            .arg("auth")
            .arg("login")
            .spawn()
            .map_err(|e| e.to_string())?
            .wait()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok("Claude Code login completed".into())
        } else {
            Err("Claude Code login failed or was cancelled".into())
        }
    }

    fn get_usage(&self) -> ProviderUsage {
        let installed = self.detect();
        let auth = self.auth_status().status;

        if !installed {
            return base_usage(self.id(), self.name(), false, AuthStatus::NotInstalled, None, None, None);
        }

        let cache = load_claude_usage_cache();
        let five_hour = cache.as_ref().and_then(|c| {
            c.five_hour_percentage.map(|p| {
                usage_window("5 hour usage", Some(p), c.five_hour_resets_at)
            })
        });
        let seven_day = cache.as_ref().and_then(|c| {
            c.seven_day_percentage.map(|p| {
                usage_window("7 day usage", Some(p), c.seven_day_resets_at)
            })
        });

        base_usage(
            self.id(),
            self.name(),
            true,
            auth,
            five_hour,
            seven_day,
            cache.map(|c| c.updated_at),
        )
    }

    fn install_hooks(&self) -> HookInstallResult {
        if !self.detect() {
            return HookInstallResult {
                success: false,
                message: "Claude Code not installed".into(),
                backup_path: None,
            };
        }

        ensure_app_dir().ok();
        let hook_script = hooks_dir().join("claude-event.sh");
        if let Err(e) = fs::write(&hook_script, claude_hook_script()) {
            return HookInstallResult {
                success: false,
                message: e.to_string(),
                backup_path: None,
            };
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&hook_script, fs::Permissions::from_mode(0o755));
        }

        let statusline_script = hooks_dir().join("claude-statusline.sh");
        if let Err(e) = fs::write(&statusline_script, claude_statusline_script()) {
            return HookInstallResult {
                success: false,
                message: e.to_string(),
                backup_path: None,
            };
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&statusline_script, fs::Permissions::from_mode(0o755));
        }

        match merge_claude_settings(&hook_script, &statusline_script) {
            Ok(backup) => HookInstallResult {
                success: true,
                message: "Claude hooks and statusline installed".into(),
                backup_path: backup,
            },
            Err(e) => HookInstallResult {
                success: false,
                message: e,
                backup_path: None,
            },
        }
    }
}

fn which_claude() -> Option<PathBuf> {
    let mut cmd = Command::new("which");
    crate::cli_env::augment_path(&mut cmd);
    let output = cmd.arg("claude").output().ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    None
}

fn load_claude_usage_cache() -> Option<ClaudeUsageCache> {
    let path = claude_usage_cache_path();
    if !path.exists() {
        return None;
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
}

fn claude_hook_script() -> String {
    format!(
        r#"#!/bin/bash
# {HOOK_MARKER} - Claude Code event hook
INPUT=$(cat)
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
EVENT_TYPE="${{CLAUDE_HOOK_EVENT_NAME:-unknown}}"
SESSION_ID=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('session_id', d.get('conversation_id', 'unknown')))" 2>/dev/null || echo "unknown")
MODEL=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('model', ''))" 2>/dev/null || echo "")
PROJECT=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('cwd', d.get('workspace', '')))" 2>/dev/null || echo "")

python3 - <<'PYEOF' "$INPUT" "$EVENT_TYPE" "$SESSION_ID" "$MODEL" "$PROJECT" "$TIMESTAMP"
import json, sys, uuid, os
from datetime import datetime

raw, event_type, session_id, model, project, timestamp = sys.argv[1:7]
data = {{}}
try:
    parsed = json.loads(raw)
    data = {{k: v for k, v in parsed.items() if k not in ('session_id', 'model', 'cwd')}}
except Exception:
    pass

event = {{
    "id": str(uuid.uuid4()),
    "provider": "claude",
    "session_id": session_id or "unknown",
    "timestamp": timestamp,
    "event_type": event_type,
    "model": model or None,
    "project": project or None,
    "status": None,
    "data": data if data else None,
}}

home = os.path.expanduser("~")
path = os.path.join(home, ".multi-platform-ai-manager", "incoming-events.ndjson")
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, "a", encoding="utf-8") as f:
    f.write(json.dumps(event) + chr(10))
PYEOF
exit 0
"#
    )
}

fn claude_statusline_script() -> String {
    format!(
        r#"#!/bin/bash
# {HOOK_MARKER} - Claude Code statusline (reads rate_limits from stdin)
INPUT=$(cat)
python3 - <<'PYEOF' "$INPUT"
import json, sys, os
from datetime import datetime, timezone

raw = sys.argv[1]
try:
    data = json.loads(raw)
except Exception:
    print("")
    sys.exit(0)

rate = data.get("rate_limits", {{}})
five = rate.get("five_hour", {{}})
seven = rate.get("seven_day", {{}})
model = data.get("model")

cache = {{
    "five_hour_percentage": five.get("used_percentage"),
    "seven_day_percentage": seven.get("used_percentage"),
    "five_hour_resets_at": five.get("resets_at"),
    "seven_day_resets_at": seven.get("resets_at"),
    "updated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "model": model,
}}

home = os.path.expanduser("~")
path = os.path.join(home, ".multi-platform-ai-manager", "claude-usage.json")
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, "w") as f:
    json.dump(cache, f)

five_pct = five.get("used_percentage")
if five_pct is not None:
    print(f"{{int(five_pct)}}%")
else:
    print("")
PYEOF
"#
    )
}

fn merge_claude_settings(
    hook_script: &PathBuf,
    statusline_script: &PathBuf,
) -> Result<Option<String>, String> {
    let settings_path = dirs::home_dir()
        .ok_or("No home directory")?
        .join(".claude")
        .join("settings.json");

    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let backup = if settings_path.exists() {
        let backup_path = settings_path.with_extension(format!("json.bak-{APP_SLUG}"));
        fs::copy(&settings_path, &backup_path).ok();
        Some(backup_path.to_string_lossy().to_string())
    } else {
        None
    };

    if !settings.is_object() {
        settings = serde_json::json!({});
    }
    let obj = settings.as_object_mut().unwrap();

    let hook_path = hook_script.to_string_lossy();
    let events = [
        "SessionStart",
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "Stop",
        "SessionEnd",
        "SubagentStop",
    ];

    if !obj.contains_key("hooks") {
        obj.insert("hooks".into(), serde_json::json!({}));
    }
    let hooks = obj.get_mut("hooks").unwrap().as_object_mut().unwrap();

    for event in events {
        let entry = serde_json::json!([{
            "command": hook_path.to_string(),
            "description": format!("{HOOK_MARKER} event capture")
        }]);
        match hooks.get_mut(event) {
            Some(existing) if existing.is_array() => {
                let arr = existing.as_array_mut().unwrap();
                let already = arr.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains(HOOK_MARKER))
                        .unwrap_or(false)
                });
                if !already {
                    if let Some(new) = entry.as_array().and_then(|a| a.first().cloned()) {
                        arr.push(new);
                    }
                }
            }
            _ => {
                hooks.insert(event.to_string(), entry);
            }
        }
    }

    obj.insert(
        "statusLine".into(),
        serde_json::json!({
            "type": "command",
            "command": statusline_script.to_string_lossy(),
            "description": format!("{HOOK_MARKER} usage capture"),
            "refreshInterval": 30
        }),
    );

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        &settings_path,
        serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(backup)
}
