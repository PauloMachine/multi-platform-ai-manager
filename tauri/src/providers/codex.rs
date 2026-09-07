use crate::cli_env::command;
use super::{base_usage, Provider};
use crate::models::{AuthStatus, HookInstallResult, ProviderAuthStatus, ProviderUsage};
use crate::storage::{ensure_app_dir, hooks_dir, APP_SLUG};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const HOOK_MARKER: &str = APP_SLUG;

pub struct CodexProvider;

impl CodexProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn binary_path() -> Option<PathBuf> {
        Self::codex_path()
    }

    fn codex_path() -> Option<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            let nvm_base = home.join(".nvm/versions/node");
            if nvm_base.is_dir() {
                if let Ok(entries) = fs::read_dir(&nvm_base) {
                    let mut candidates: Vec<PathBuf> = entries
                        .flatten()
                        .map(|entry| entry.path().join("bin/codex"))
                        .filter(|p| p.exists())
                        .collect();
                    candidates.sort();
                    if let Some(path) = candidates.pop() {
                        return Some(path);
                    }
                }
            }
            let local = home.join(".local/bin/codex");
            if local.exists() {
                return Some(local);
            }
        }
        for path in [
            PathBuf::from("/usr/local/bin/codex"),
            PathBuf::from("/opt/homebrew/bin/codex"),
        ] {
            if path.exists() {
                return Some(path);
            }
        }
        which_binary("codex")
    }

    fn version(&self) -> Option<String> {
        Self::codex_path().and_then(|path| {
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

impl Provider for CodexProvider {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn name(&self) -> &'static str {
        "Codex"
    }

    fn detect(&self) -> bool {
        Self::codex_path().is_some()
    }

    fn auth_status(&self) -> ProviderAuthStatus {
        if !self.detect() {
            return ProviderAuthStatus {
                provider: self.id().to_string(),
                status: AuthStatus::NotInstalled,
                message: Some(
                    "Codex CLI not found. Install with: npm install -g @openai/codex".into(),
                ),
                version: None,
            };
        }

        let path = Self::codex_path().unwrap();
        match command(&path).args(["login", "status"]).output() {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                ProviderAuthStatus {
                    provider: self.id().to_string(),
                    status: AuthStatus::Connected,
                    message: if stdout.is_empty() {
                        Some("Logged in".into())
                    } else {
                        Some(stdout)
                    },
                    version: self.version(),
                }
            }
            Ok(o) => {
                let msg = String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .to_string();
                let msg = if msg.is_empty() {
                    String::from_utf8_lossy(&o.stderr).trim().to_string()
                } else {
                    msg
                };
                ProviderAuthStatus {
                    provider: self.id().to_string(),
                    status: AuthStatus::NotConnected,
                    message: Some(if msg.is_empty() {
                        "Not logged in".into()
                    } else {
                        msg
                    }),
                    version: self.version(),
                }
            }
            Err(e) => ProviderAuthStatus {
                provider: self.id().to_string(),
                status: AuthStatus::Unknown,
                message: Some(e.to_string()),
                version: self.version(),
            },
        }
    }

    fn connect(&self) -> Result<String, String> {
        let path = Self::codex_path().ok_or("Codex CLI is not installed")?;
        let status = command(&path)
            .args(["login"])
            .spawn()
            .map_err(|e| e.to_string())?
            .wait()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok("Codex login completed".into())
        } else {
            Err("Codex login failed or was cancelled".into())
        }
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

        // Codex quota is only shown in the TUI (/usage) — no official CLI JSON export.
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
        let hook_script = hooks_dir().join("codex-event.sh");
        if let Err(e) = fs::write(&hook_script, codex_hook_script()) {
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

        match merge_codex_hooks(&hook_script) {
            Ok(backup) => HookInstallResult {
                success: true,
                message: "Codex hooks installed".into(),
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

fn codex_hook_script() -> String {
    format!(
        r#"#!/bin/bash
# {HOOK_MARKER} - Codex event hook
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
session_id = data.get("session_id") or "unknown"
model = data.get("model")
project = data.get("cwd")
status = data.get("reason") or data.get("final_status") or data.get("status")

skip = {{
    "session_id", "model", "cwd", "hook_event_name", "reason",
    "final_status", "status", "transcript_path", "prompt",
    "tool_name", "tool_input", "tool_use_id", "turn_id",
}}

event = {{
    "id": str(uuid.uuid4()),
    "provider": "codex",
    "session_id": str(session_id),
    "timestamp": timestamp,
    "event_type": event_type,
    "model": model,
    "project": project,
    "status": status,
    "data": {{k: v for k, v in data.items() if k not in skip}},
}}

for key in ("transcript_path", "prompt", "tool_name", "tool_input", "turn_id"):
    if key in data and data[key] is not None:
        event["data"][key] = data[key]

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

fn merge_codex_hooks(hook_script: &PathBuf) -> Result<Option<String>, String> {
    let hooks_path = dirs::home_dir()
        .ok_or("No home directory")?
        .join(".codex")
        .join("hooks.json");

    let mut settings: serde_json::Value = if hooks_path.exists() {
        let content = fs::read_to_string(&hooks_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({"hooks": {}}))
    } else {
        serde_json::json!({"hooks": {}})
    };

    let backup = if hooks_path.exists() {
        let backup_path = hooks_path.with_extension(format!("json.bak-{APP_SLUG}"));
        fs::copy(&hooks_path, &backup_path).ok();
        Some(backup_path.to_string_lossy().to_string())
    } else {
        None
    };

    if !settings.is_object() {
        settings = serde_json::json!({"hooks": {}});
    }
    let obj = settings.as_object_mut().unwrap();
    if !obj.contains_key("hooks") {
        obj.insert("hooks".into(), serde_json::json!({}));
    }
    let hooks = obj.get_mut("hooks").unwrap().as_object_mut().unwrap();

    let hook_path = hook_script.to_string_lossy().to_string();
    let events = [
        "SessionStart",
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "Stop",
        "SessionEnd",
    ];

    for event in events {
        ensure_codex_hook_entry(hooks, event, &hook_path);
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

fn ensure_codex_hook_entry(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    hook_path: &str,
) {
    let new_hook = serde_json::json!({
        "type": "command",
        "command": hook_path,
        "statusMessage": format!("{HOOK_MARKER} event capture")
    });

    let entry = hooks
        .entry(event.to_string())
        .or_insert_with(|| serde_json::json!([]));
    let groups = entry.as_array_mut().unwrap();

    let group_idx = groups.iter().position(|g| {
        g.is_object() && g.get("matcher").is_none()
    });

    if let Some(idx) = group_idx {
        let group = groups[idx].as_object_mut().unwrap();
        let hook_list = group
            .entry("hooks")
            .or_insert_with(|| serde_json::json!([]));
        let hooks_arr = hook_list.as_array_mut().unwrap();
        let already = hooks_arr.iter().any(|h| {
            h.get("command")
                .and_then(|c| c.as_str())
                .map(|c| c.contains(HOOK_MARKER))
                .unwrap_or(false)
        });
        if !already {
            hooks_arr.push(new_hook);
        }
    } else {
        groups.push(serde_json::json!({
            "hooks": [new_hook]
        }));
    }
}
