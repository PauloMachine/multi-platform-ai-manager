use crate::models::AISessionDetails;
use crate::providers::resolve_provider_binary;
use crate::storage::{get_session, read_transcript_excerpt, APP_SLUG};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct SessionRunResult {
    pub continuation_prompt: String,
    pub from_provider: String,
    pub to_provider: String,
    pub session_id: String,
}

pub fn build_session_run(
    session_id: &str,
    to_provider: &str,
) -> Result<SessionRunResult, String> {
    let details = get_session(session_id).ok_or("Session not found")?;
    let from = details.session.provider.clone();

    if from == to_provider {
        return Err("Target provider must differ from the session provider".into());
    }

    if resolve_provider_binary(to_provider).is_none() {
        return Err(format!(
            "{} CLI is not installed",
            provider_display(to_provider)
        ));
    }

    let from_name = provider_display(&from);
    let to_name = provider_display(to_provider);
    let continuation_prompt = build_continuation_prompt(&details, &from_name, &to_name);

    Ok(SessionRunResult {
        continuation_prompt,
        from_provider: from,
        to_provider: to_provider.to_string(),
        session_id: session_id.to_string(),
    })
}

pub fn run_session_on_provider(session_id: &str, to_provider: &str) -> Result<SessionRunResult, String> {
    let run = build_session_run(session_id, to_provider)?;
    let details = get_session(session_id).ok_or("Session not found")?;
    let binary = resolve_provider_binary(to_provider).ok_or_else(|| {
        format!(
            "{} CLI is not installed",
            provider_display(to_provider)
        )
    })?;

    let prompt_path = save_prompt_file(&run.session_id, &run.continuation_prompt)?;
    let project_dir = details
        .session
        .project
        .as_deref()
        .filter(|p| Path::new(p).is_dir())
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));

    launch_terminal(&binary, &prompt_path, &project_dir)?;

    Ok(run)
}

fn build_continuation_prompt(details: &AISessionDetails, from: &str, to: &str) -> String {
    let s = &details.session;
    let project = s
        .project
        .as_deref()
        .map(|p| {
            Path::new(p)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(p)
        })
        .unwrap_or("this project");
    let task = s
        .initial_prompt
        .as_deref()
        .unwrap_or("Continue the previous AI coding session");

    let mut activity_lines: Vec<String> = details
        .activity
        .iter()
        .rev()
        .take(12)
        .map(|a| format!("- {}", a.description))
        .collect();
    activity_lines.reverse();

    let files: String = if details.files_modified.is_empty() {
        "None recorded yet".into()
    } else {
        details.files_modified.join(", ")
    };

    let tools: String = if details.tools_used.is_empty() {
        "None recorded".into()
    } else {
        details.tools_used.join(", ")
    };

    let transcript_note = details
        .transcript_path
        .as_deref()
        .and_then(|p| read_transcript_excerpt(p, 1500))
        .map(|excerpt| format!("\n\nRecent transcript excerpt:\n{excerpt}"))
        .unwrap_or_default();

    format!(
        r#"Continue this coding task (handoff from {from} → {to}).

Project: {project}
Previous model: {model}
Original task: {task}

Work already done:
{activity}

Tools used: {tools}
Files modified: {files}

Pick up where the previous session left off. Do not restart from scratch — build on the progress above.{transcript}"#,
        from = from,
        to = to,
        project = project,
        model = s.model.as_deref().unwrap_or("unknown"),
        task = task,
        activity = if activity_lines.is_empty() {
            "- (no activity log yet)".into()
        } else {
            activity_lines.join("\n")
        },
        tools = tools,
        files = files,
        transcript = transcript_note,
    )
}

fn handoff_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(format!(".{APP_SLUG}"))
        .join("handoff")
}

fn save_prompt_file(session_id: &str, prompt: &str) -> Result<PathBuf, String> {
    let dir = handoff_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("run-{session_id}.txt"));
    fs::write(&path, prompt).map_err(|e| e.to_string())?;
    Ok(path)
}

fn launch_terminal(binary: &Path, prompt_path: &Path, project_dir: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let script_path = write_launch_script_windows(binary, prompt_path, project_dir)?;
        Command::new("cmd")
            .args([
                "/C",
                "start",
                "cmd",
                "/K",
                script_path.to_str().unwrap_or(""),
            ])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {e}"))?;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let script_path = write_launch_script_unix(binary, prompt_path, project_dir)?;

        #[cfg(target_os = "macos")]
        {
            let script = script_path.to_string_lossy();
            let escaped = script.replace('\\', "\\\\").replace('"', "\\\"");
            let applescript = format!(
                r#"tell application "Terminal"
    activate
    do script "bash \"{}\""
end tell"#,
                escaped
            );
            Command::new("osascript")
                .arg("-e")
                .arg(applescript)
                .spawn()
                .map_err(|e| format!("Failed to open Terminal: {e}"))?;
            return Ok(());
        }

        #[cfg(not(target_os = "macos"))]
        {
            Command::new("x-terminal-emulator")
                .arg("-e")
                .arg(&script_path)
                .spawn()
                .or_else(|_| {
                    Command::new("gnome-terminal")
                        .args(["--", "bash", script_path.to_str().unwrap_or("")])
                        .spawn()
                })
                .or_else(|_| {
                    Command::new("konsole")
                        .args(["-e", "bash", script_path.to_str().unwrap_or("")])
                        .spawn()
                })
                .map_err(|e| format!("Failed to open terminal: {e}"))?;
            Ok(())
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn write_launch_script_unix(
    binary: &Path,
    prompt_path: &Path,
    project_dir: &Path,
) -> Result<PathBuf, String> {
    let dir = handoff_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let script_path = dir.join("launch-session.sh");

    let script = format!(
        r#"#!/usr/bin/env bash
cd "{project_dir}" || exit 1
exec "{binary}" "$(cat "{prompt_path}")"
"#,
        project_dir = shell_escape(project_dir.to_string_lossy().as_ref()),
        binary = shell_escape(binary.to_string_lossy().as_ref()),
        prompt_path = shell_escape(prompt_path.to_string_lossy().as_ref()),
    );

    fs::write(&script_path, script).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).map_err(|e| e.to_string())?;
    }
    Ok(script_path)
}

#[cfg(target_os = "windows")]
fn write_launch_script_windows(
    binary: &Path,
    prompt_path: &Path,
    project_dir: &Path,
) -> Result<PathBuf, String> {
    let dir = handoff_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let script_path = dir.join("launch-session.cmd");

    let script = format!(
        "@echo off\r\n\
         cd /d \"{project_dir}\"\r\n\
         if errorlevel 1 exit /b 1\r\n\
         powershell -NoProfile -Command \"& '{binary}' (Get-Content -Raw -LiteralPath '{prompt_path}')\"\r\n",
        project_dir = windows_path_escape(project_dir),
        binary = windows_path_escape(binary),
        prompt_path = windows_path_escape(prompt_path),
    );

    fs::write(&script_path, script).map_err(|e| e.to_string())?;
    Ok(script_path)
}

#[cfg(target_os = "windows")]
fn windows_path_escape(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

fn shell_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn provider_display(id: &str) -> String {
    match id {
        "claude" => "Claude Code".into(),
        "cursor" => "Cursor".into(),
        "codex" => "Codex".into(),
        _ => id.into(),
    }
}
