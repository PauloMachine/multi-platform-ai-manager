use crate::models::{AIEvent, AISession, AISessionDetails, SessionActivity, SessionStatus};
use crate::storage::{events_path, incoming_events_path, sessions_cache_path};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SessionsCache {
    sessions: HashMap<String, AISessionDetails>,
}

fn load_sessions_cache() -> SessionsCache {
    let path = sessions_cache_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cache) = serde_json::from_str(&content) {
                return cache;
            }
        }
    }
    SessionsCache::default()
}

fn save_sessions_cache(cache: &SessionsCache) -> std::io::Result<()> {
    let content = serde_json::to_string_pretty(cache)?;
    fs::write(sessions_cache_path(), content)
}

pub fn append_event(event: &AIEvent) -> std::io::Result<()> {
    let path = events_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let line = serde_json::to_string(event)?;
    writeln!(file, "{line}")?;
    update_session_from_event(event)?;
    Ok(())
}

pub fn read_events(limit: Option<usize>) -> Vec<AIEvent> {
    let path = events_path();
    if !path.exists() {
        return vec![];
    }
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let reader = BufReader::new(file);
    let mut events: Vec<AIEvent> = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter_map(|l| serde_json::from_str(&l).ok())
        .collect();
    if let Some(n) = limit {
        let start = events.len().saturating_sub(n);
        events = events[start..].to_vec();
    }
    events
}

pub fn process_incoming_events() -> usize {
    let incoming = incoming_events_path();
    if !incoming.exists() {
        return 0;
    }
    let content = match fs::read_to_string(&incoming) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    if content.trim().is_empty() {
        return 0;
    }

    let records = parse_incoming_records(&content);
    let mut count = 0;
    for record in records {
        if let Ok(mut event) = serde_json::from_str::<AIEvent>(&record) {
            normalize_event(&mut event);
            if append_event(&event).is_ok() {
                count += 1;
            }
        }
    }
    let _ = fs::write(&incoming, "");
    count
}

fn parse_incoming_records(content: &str) -> Vec<String> {
    let mut records = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains("\\n") {
            for part in trimmed.split("\\n") {
                let part = part.trim();
                if !part.is_empty() {
                    records.push(part.to_string());
                }
            }
        } else {
            records.push(trimmed.to_string());
        }
    }
    if records.is_empty() {
        for part in content.split("\\n") {
            let part = part.trim();
            if !part.is_empty() {
                records.push(part.to_string());
            }
        }
    }
    records
}

fn normalize_event(event: &mut AIEvent) {
    if event.event_type == "unknown" {
        if let Some(data) = &event.data {
            if let Some(name) = data.get("hook_event_name").and_then(|v| v.as_str()) {
                event.event_type = name.to_string();
            }
        }
    }

    let project_missing = event
        .project
        .as_ref()
        .map(|p| p.is_empty())
        .unwrap_or(true);
    if project_missing {
        if let Some(data) = &event.data {
            if let Some(cwd) = data.get("cwd").and_then(|v| v.as_str()) {
                if !cwd.is_empty() {
                    event.project = Some(cwd.to_string());
                }
            }
            if let Some(roots) = data.get("workspace_roots").and_then(|v| v.as_array()) {
                if let Some(first) = roots.first().and_then(|v| v.as_str()) {
                    event.project = Some(first.to_string());
                }
            }
        }
    }
}

fn effective_event_type(event: &AIEvent) -> String {
    if event.event_type != "unknown" {
        return event.event_type.clone();
    }
    event
        .data
        .as_ref()
        .and_then(|d| d.get("hook_event_name"))
        .and_then(|v| v.as_str())
        .unwrap_or(&event.event_type)
        .to_string()
}

fn session_has_activity(details: &AISessionDetails) -> bool {
    details.session.initial_prompt.is_some()
        || details
            .session
            .project
            .as_ref()
            .is_some_and(|p| !p.is_empty())
        || !details.activity.is_empty()
}

fn cursor_session_start_is_real(event: &AIEvent) -> bool {
    if event.provider != "cursor" {
        return true;
    }
    let data = match &event.data {
        Some(d) => d,
        None => return event.project.as_ref().is_some_and(|p| !p.is_empty()),
    };
    let has_conversation = data
        .get("conversation_id")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty());
    let has_workspace = event.project.as_ref().is_some_and(|p| !p.is_empty())
        || data
            .get("workspace_roots")
            .and_then(|v| v.as_array())
            .is_some_and(|roots| !roots.is_empty());
    has_conversation || has_workspace
}

fn mark_session_running(session: &mut AISession) {
    session.status = SessionStatus::Running;
    session.ended_at = None;
    session.duration_secs = None;
}

fn apply_event_to_cache(cache: &mut SessionsCache, event: &AIEvent) {
    let entry = cache
        .sessions
        .entry(event.session_id.clone())
        .or_insert_with(|| AISessionDetails {
            session: AISession {
                id: event.session_id.clone(),
                provider: event.provider.clone(),
                model: event.model.clone(),
                project: event.project.clone(),
                started_at: event.timestamp.clone(),
                ended_at: None,
                duration_secs: None,
                status: SessionStatus::Running,
                initial_prompt: None,
                tokens: None,
                cost: None,
                conversation_id: None,
            },
            tools_used: vec![],
            files_modified: vec![],
            activity: vec![],
            transcript_path: None,
            error_message: None,
        });

    let session = &mut entry.session;
    if let Some(model) = &event.model {
        if !model.is_empty() {
            session.model = Some(model.clone());
        }
    }
    if session.project.is_none() || session.project.as_deref() == Some("") {
        session.project = event.project.clone();
    }

    if let Some(data) = &event.data {
        if entry.transcript_path.is_none() {
            if let Some(tp) = data.get("transcript_path").and_then(|v| v.as_str()) {
                entry.transcript_path = Some(tp.to_string());
            }
        }
        if session.conversation_id.is_none() {
            if let Some(cid) = data.get("conversation_id").and_then(|v| v.as_str()) {
                session.conversation_id = Some(cid.to_string());
            }
        }
    }

    let event_type = effective_event_type(event);
    match event_type.as_str() {
        "sessionStart" | "SessionStart" => {
            if cursor_session_start_is_real(event) {
                mark_session_running(session);
                session.started_at = event.timestamp.clone();
            } else {
                session.status = SessionStatus::Aborted;
                session.ended_at = Some(event.timestamp.clone());
            }
            if let Some(data) = &event.data {
                if let Some(prompt) = data.get("prompt").and_then(|v| v.as_str()) {
                    session.initial_prompt = Some(truncate(prompt, 500));
                }
                if let Some(cid) = data.get("conversation_id").and_then(|v| v.as_str()) {
                    if !cid.is_empty() {
                        session.conversation_id = Some(cid.to_string());
                    }
                }
                if let Some(tp) = data.get("transcript_path").and_then(|v| v.as_str()) {
                    entry.transcript_path = Some(tp.to_string());
                }
            }
        }
        "beforeSubmitPrompt" | "UserPromptSubmit" => {
            // Cursor reuses conversation_id across turns — reopen the session.
            mark_session_running(session);
            if let Some(data) = &event.data {
                if let Some(prompt) = data.get("prompt").and_then(|v| v.as_str()) {
                    session.initial_prompt = Some(truncate(prompt, 500));
                }
            }
        }
        "preToolUse" | "PreToolUse" | "postToolUse" | "PostToolUse" => {
            if session.status != SessionStatus::Running {
                mark_session_running(session);
            }
            if let Some(data) = &event.data {
                if let Some(tool) = data.get("tool_name").and_then(|v| v.as_str()) {
                    if !entry.tools_used.contains(&tool.to_string()) {
                        entry.tools_used.push(tool.to_string());
                    }
                }
            }
            entry.activity.push(SessionActivity {
                timestamp: event.timestamp.clone(),
                description: format_activity(event),
                activity_type: Some(event_type.clone()),
            });
        }
        "afterFileEdit" => {
            if session.status != SessionStatus::Running {
                mark_session_running(session);
            }
            if let Some(data) = &event.data {
                if let Some(file) = data.get("file_path").and_then(|v| v.as_str()) {
                    if !entry.files_modified.contains(&file.to_string()) {
                        entry.files_modified.push(file.to_string());
                    }
                }
            }
            entry.activity.push(SessionActivity {
                timestamp: event.timestamp.clone(),
                description: format_activity(event),
                activity_type: Some(event_type.clone()),
            });
        }
        "stop" | "sessionEnd" | "Stop" | "SessionEnd" | "SubagentStop" => {
            session.ended_at = Some(event.timestamp.clone());
            session.status = match event.status.as_deref() {
                Some("failed") | Some("error") => SessionStatus::Failed,
                Some("aborted") | Some("cancelled") => SessionStatus::Aborted,
                Some("completed") | Some("success") => SessionStatus::Completed,
                _ => SessionStatus::Completed,
            };
            if let Some(data) = &event.data {
                if let Some(err) = data.get("error_message").and_then(|v| v.as_str()) {
                    entry.error_message = Some(err.to_string());
                    session.status = SessionStatus::Failed;
                }
                if let Some(tokens) = data.get("tokens").and_then(|v| v.as_u64()) {
                    session.tokens = Some(session.tokens.unwrap_or(0) + tokens);
                }
                let input = data.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let output = data.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                if input + output > 0 {
                    session.tokens = Some(session.tokens.unwrap_or(0) + input + output);
                }
                if let Some(cost) = data.get("cost").and_then(|v| v.as_f64()) {
                    session.cost = Some(session.cost.unwrap_or(0.0) + cost);
                }
            }
            if let (Ok(start), Ok(end)) = (
                DateTime::parse_from_rfc3339(&session.started_at),
                DateTime::parse_from_rfc3339(event.timestamp.as_str()),
            ) {
                session.duration_secs = Some((end - start).num_seconds().max(0) as u64);
            }
            if event
                .data
                .as_ref()
                .and_then(|d| d.get("forced"))
                .and_then(|v| v.as_bool())
                == Some(true)
            {
                entry.activity.push(SessionActivity {
                    timestamp: event.timestamp.clone(),
                    description: "Stopped manually from Multi-platform AI Manager".to_string(),
                    activity_type: Some(event_type.clone()),
                });
            }
        }
        _ => {
            if !event_type.is_empty() && event_type != "unknown" {
                entry.activity.push(SessionActivity {
                    timestamp: event.timestamp.clone(),
                    description: format_activity(event),
                    activity_type: Some(event_type.clone()),
                });
            }
        }
    }
}

fn update_session_from_event(event: &AIEvent) -> std::io::Result<()> {
    let mut cache = load_sessions_cache();
    apply_event_to_cache(&mut cache, event);
    save_sessions_cache(&cache)
}

pub fn rebuild_sessions_from_events() -> std::io::Result<usize> {
    let events = read_events(None);
    let mut cache = SessionsCache::default();
    for event in &events {
        apply_event_to_cache(&mut cache, event);
    }
    let count = cache.sessions.len();
    save_sessions_cache(&cache)?;
    Ok(count)
}

fn session_sort_timestamp(details: &AISessionDetails) -> String {
    if details.session.status == SessionStatus::Running {
        return details
            .activity
            .last()
            .map(|a| a.timestamp.clone())
            .unwrap_or_else(|| details.session.started_at.clone());
    }
    details
        .session
        .ended_at
        .clone()
        .unwrap_or_else(|| details.session.started_at.clone())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

fn format_activity(event: &AIEvent) -> String {
    if let Some(data) = &event.data {
        if let Some(tool) = data.get("tool_name").and_then(|v| v.as_str()) {
            return format!("Tool: {tool}");
        }
        if let Some(input) = data.get("tool_input").and_then(|v| v.as_object()) {
            if let Some(cmd) = input.get("command").and_then(|v| v.as_str()) {
                return format!("Run {}", truncate(cmd, 80));
            }
            if let Some(file) = input.get("file_path").and_then(|v| v.as_str()) {
                return format!("Read {}", file.split('/').next_back().unwrap_or(file));
            }
        }
        if let Some(file) = data.get("file_path").and_then(|v| v.as_str()) {
            return format!("Edit {file}");
        }
        if let Some(cmd) = data.get("command").and_then(|v| v.as_str()) {
            return format!("Run {cmd}");
        }
    }
    event.event_type.clone()
}

pub fn get_sessions(provider: Option<&str>, model: Option<&str>) -> Vec<AISession> {
    let cache = load_sessions_cache();
    let mut sessions: Vec<(AISession, String, bool)> = cache
        .sessions
        .values()
        .filter(|d| {
            provider
                .map(|p| d.session.provider == p)
                .unwrap_or(true)
                && model
                    .map(|m| d.session.model.as_deref() == Some(m))
                    .unwrap_or(true)
        })
        .map(|d| {
            let running = d.session.status == SessionStatus::Running;
            (
                d.session.clone(),
                session_sort_timestamp(d),
                running,
            )
        })
        .collect();
    sessions.sort_by(|a, b| {
        b.2
            .cmp(&a.2)
            .then_with(|| b.1.cmp(&a.1))
    });
    sessions.into_iter().map(|(s, _, _)| s).collect()
}

pub fn get_session(id: &str) -> Option<AISessionDetails> {
    load_sessions_cache().sessions.get(id).cloned()
}

pub fn get_active_sessions() -> Vec<AISession> {
    let cache = load_sessions_cache();
    cache
        .sessions
        .values()
        .filter(|d| d.session.status == SessionStatus::Running && session_has_activity(d))
        .map(|d| d.session.clone())
        .collect()
}

pub fn force_stop_session(session_id: &str) -> Result<(), String> {
    let details = get_session(session_id).ok_or_else(|| format!("Session not found: {session_id}"))?;
    if details.session.status != SessionStatus::Running {
        return Err("Session is not running".to_string());
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    let event = AIEvent {
        id: uuid::Uuid::new_v4().to_string(),
        provider: details.session.provider.clone(),
        session_id: session_id.to_string(),
        timestamp,
        event_type: "Stop".to_string(),
        model: details.session.model.clone(),
        project: details.session.project.clone(),
        status: Some("aborted".to_string()),
        data: Some(std::collections::HashMap::from([(
            "forced".to_string(),
            serde_json::Value::Bool(true),
        )])),
    };

    append_event(&event).map_err(|e| e.to_string())
}

pub fn compute_model_usage(provider: Option<&str>) -> Vec<crate::models::ModelUsage> {
    let sessions = get_sessions(provider, None);
    let mut map: HashMap<String, (u32, Option<u64>, Option<f64>)> = HashMap::new();
    for s in sessions {
        let model = s.model.unwrap_or_else(|| "Unknown".to_string());
        let entry = map.entry(model).or_insert((0, None, None));
        entry.0 += 1;
        if let Some(t) = s.tokens {
            entry.1 = Some(entry.1.unwrap_or(0) + t);
        }
        if let Some(c) = s.cost {
            entry.2 = Some(entry.2.unwrap_or(0.0) + c);
        }
    }
    let total: u32 = map.values().map(|(c, _, _)| c).sum();
    map.into_iter()
        .map(|(model, (count, tokens, cost))| {
            let percentage = if total > 0 {
                Some((count as f64 / total as f64) * 100.0)
            } else {
                None
            };
            crate::models::ModelUsage {
                model,
                percentage,
                tokens,
                cost,
                session_count: count,
            }
        })
        .collect()
}

pub fn read_transcript_excerpt(path: &str, max_bytes: usize) -> Option<String> {
    if !Path::new(path).exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    if content.len() <= max_bytes {
        Some(content)
    } else {
        Some(format!("{}…", &content[..max_bytes]))
    }
}
