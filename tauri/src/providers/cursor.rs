use crate::cli_env::command;
use super::{base_usage, Provider};
use crate::models::{AuthStatus, HookInstallResult, ProviderAuthStatus, ProviderUsage};
use crate::storage::{ensure_app_dir, hooks_dir, APP_SLUG};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const HOOK_MARKER: &str = APP_SLUG;

pub struct CursorProvider;

impl CursorProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn binary_path() -> Option<PathBuf> {
        Self::cursor_agent_path().or_else(|| which_binary("agent"))
    }

    fn cursor_app_path() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let path = PathBuf::from("/Applications/Cursor.app/Contents/Resources/app/bin/cursor");
            if path.exists() {
                return Some(path);
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                let path = PathBuf::from(local)
                    .join("Programs")
                    .join("cursor")
                    .join("Cursor.exe");
                if path.exists() {
                    return Some(path);
                }
            }
        }
        which_binary("cursor")
    }

    fn cursor_agent_path() -> Option<PathBuf> {
        let local = dirs::home_dir()?.join(".local/bin/cursor-agent");
        if local.exists() {
            return Some(local);
        }
        which_binary("cursor-agent")
    }

    fn version(&self) -> Option<String> {
        Self::cursor_app_path().and_then(|path| {
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
        })
    }
}

impl Provider for CursorProvider {
    fn id(&self) -> &'static str {
        "cursor"
    }

    fn name(&self) -> &'static str {
        "Cursor"
    }

    fn detect(&self) -> bool {
        Self::cursor_app_path().is_some() || Self::cursor_agent_path().is_some()
    }

    fn auth_status(&self) -> ProviderAuthStatus {
        if !self.detect() {
            return ProviderAuthStatus {
                provider: self.id().to_string(),
                status: AuthStatus::NotInstalled,
                message: Some("Cursor not found. Install from https://cursor.com".into()),
                version: None,
            };
        }

        if let Some(agent) = Self::cursor_agent_path() {
            let output = command(&agent).arg("status").output();
            match output {
                Ok(o) if o.status.success() => {
                    let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    let connected = !stdout.to_lowercase().contains("not logged")
                        && !stdout.to_lowercase().contains("not authenticated");
                    return ProviderAuthStatus {
                        provider: self.id().to_string(),
                        status: if connected {
                            AuthStatus::Connected
                        } else {
                            AuthStatus::NotConnected
                        },
                        message: Some(stdout),
                        version: self.version(),
                    };
                }
                Ok(o) => {
                    return ProviderAuthStatus {
                        provider: self.id().to_string(),
                        status: AuthStatus::NotConnected,
                        message: Some(String::from_utf8_lossy(&o.stderr).trim().to_string()),
                        version: self.version(),
                    };
                }
                Err(_) => {}
            }
        }

        if Self::cursor_app_path().is_some() {
            ProviderAuthStatus {
                provider: self.id().to_string(),
                status: AuthStatus::Connected,
                message: Some(
                    "Cursor installed. cursor-agent CLI not available — session hooks still work."
                        .into(),
                ),
                version: self.version(),
            }
        } else {
            ProviderAuthStatus {
                provider: self.id().to_string(),
                status: AuthStatus::Unknown,
                message: None,
                version: self.version(),
            }
        }
    }

    fn connect(&self) -> Result<String, String> {
        if let Some(agent) = Self::cursor_agent_path() {
            let status = command(&agent)
                .arg("login")
                .spawn()
                .map_err(|e| e.to_string())?
                .wait()
                .map_err(|e| e.to_string())?;
            if status.success() {
                return Ok("Cursor login completed".into());
            }
            return Err("Cursor login failed or was cancelled".into());
        }

        if let Some(cursor) = Self::cursor_app_path() {
            Command::new(&cursor)
                .spawn()
                .map_err(|e| e.to_string())?;
            return Ok("Opened Cursor — use cursor-agent login if CLI auth is needed".into());
        }

        Err("Cursor is not installed".into())
    }

    fn get_usage(&self) -> ProviderUsage {
        let installed = self.detect();
        let auth = self.auth_status().status;

        if !installed {
            return base_usage(
                self.id(),
                self.name(),
                false,
                AuthStatus::NotInstalled,
                None,
                None,
                None,
            );
        }

        // Cursor individual accounts do not expose official quota via CLI.
        base_usage(
            self.id(),
            self.name(),
            true,
            auth,
            None,
            None,
            None,
        )
    }

    fn install_hooks(&self) -> HookInstallResult {
        ensure_app_dir().ok();
        let hook_script = hooks_dir().join("cursor-event.sh");
        if let Err(e) = fs::write(&hook_script, cursor_hook_script()) {
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

        match merge_cursor_hooks(&hook_script) {
            Ok(backup) => HookInstallResult {
                success: true,
                message: "Cursor hooks installed".into(),
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

fn which_binary(name: &str) -> Option<PathBuf> {
    let mut cmd = Command::new("which");
    crate::cli_env::augment_path(&mut cmd);
    let output = cmd.arg(name).output().ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    None
}

fn cursor_hook_script() -> String {
    format!(
        r#"#!/bin/bash
# {HOOK_MARKER} - Cursor event hook
INPUT=$(cat)
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
python3 - <<'PYEOF' "$INPUT" "$TIMESTAMP"
import json, sys, uuid, os

raw, timestamp = sys.argv[1:3]
data = {{}}
try:
    parsed = json.loads(raw)
    data = parsed
except Exception:
    pass

event_type = data.get("hook_event_name") or "unknown"
session_id = (
    data.get("conversation_id")
    or data.get("session_id")
    or data.get("generation_id")
    or "unknown"
)
model = data.get("model") or data.get("model_id")
roots = data.get("workspace_roots") or []
project = data.get("workspace") or data.get("cwd") or (roots[0] if roots else None)
status = data.get("final_status") or data.get("status")

event = {{
    "id": str(uuid.uuid4()),
    "provider": "cursor",
    "session_id": str(session_id),
    "timestamp": timestamp,
    "event_type": event_type,
    "model": model,
    "project": project,
    "status": status,
    "data": {{k: v for k, v in data.items() if k not in ("conversation_id", "session_id", "model", "workspace", "cwd", "final_status", "status", "hook_event_name")}},
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

fn merge_cursor_hooks(hook_script: &PathBuf) -> Result<Option<String>, String> {
    let hooks_path = dirs::home_dir()
        .ok_or("No home directory")?
        .join(".cursor")
        .join("hooks.json");

    let mut settings: serde_json::Value = if hooks_path.exists() {
        let content = fs::read_to_string(&hooks_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({"version": 1, "hooks": {}}))
    } else {
        serde_json::json!({"version": 1, "hooks": {}})
    };

    let backup = if hooks_path.exists() {
        let backup_path = hooks_path.with_extension(format!("json.bak-{APP_SLUG}"));
        fs::copy(&hooks_path, &backup_path).ok();
        Some(backup_path.to_string_lossy().to_string())
    } else {
        None
    };

    if !settings.is_object() {
        settings = serde_json::json!({"version": 1, "hooks": {}});
    }
    let obj = settings.as_object_mut().unwrap();
    if !obj.contains_key("version") {
        obj.insert("version".into(), serde_json::json!(1));
    }
    if !obj.contains_key("hooks") {
        obj.insert("hooks".into(), serde_json::json!({}));
    }
    let hooks = obj.get_mut("hooks").unwrap().as_object_mut().unwrap();

    let hook_path = format!("{}", hook_script.to_string_lossy());
    let events = [
        "sessionStart",
        "beforeSubmitPrompt",
        "preToolUse",
        "postToolUse",
        "afterFileEdit",
        "stop",
        "sessionEnd",
    ];

    for event in events {
        let entry = serde_json::json!([{
            "command": hook_path.clone(),
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

    if let Some(parent) = hooks_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        &hooks_path,
        serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(backup)
}
