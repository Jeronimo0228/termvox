//! Discover upstream session ids from agent state on disk and PTY output heuristics.

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::SupportedAgent;

/// Best-effort lookup of the latest upstream session id for a workspace directory.
#[must_use]
pub fn discover_remote_session(kind: SupportedAgent, cwd: &Path) -> Option<String> {
    let canonical = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    match kind {
        SupportedAgent::Cursor => discover_cursor(&canonical),
        SupportedAgent::OpenCode => discover_opencode(&canonical),
        SupportedAgent::Claude => discover_claude(&canonical),
        SupportedAgent::Codex | SupportedAgent::Gemini | SupportedAgent::Aider | SupportedAgent::Amp => {
            None
        }
    }
}

/// Scan arbitrary PTY output for structured or heuristic session identifiers.
#[must_use]
pub fn scan_output_for_session_id(kind: SupportedAgent, text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(id) = crate::extract_session_id(kind, line) {
            return Some(id);
        }
    }
    heuristic_session_id(text)
}

fn discover_cursor(cwd: &Path) -> Option<String> {
    let root = dirs::home_dir()?.join(".cursor/projects").join(cursor_project_slug(cwd));
    latest_subdirectory_id(root.join("agent-transcripts"))
}

fn discover_claude(cwd: &Path) -> Option<String> {
    let root = dirs::home_dir()?
        .join(".claude/projects")
        .join(claude_project_slug(cwd));
    latest_subdirectory_id(root)
}

fn discover_opencode(cwd: &Path) -> Option<String> {
    let db = dirs::data_dir()?.join("opencode/opencode.db");
    if !db.is_file() {
        return None;
    }
    query_sqlite(
        &db,
        &format!(
            "SELECT id FROM session WHERE directory = '{}' ORDER BY time_updated DESC LIMIT 1;",
            cwd.display().to_string().replace('\'', "''")
        ),
    )
    .or_else(|| {
        query_sqlite(
            &db,
            "SELECT id FROM session ORDER BY time_updated DESC LIMIT 1;",
        )
    })
}

fn query_sqlite(db: &Path, sql: &str) -> Option<String> {
    let output = Command::new("sqlite3")
        .arg(db)
        .arg(sql)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() { None } else { Some(id) }
}

fn latest_subdirectory_id(dir: PathBuf) -> Option<String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().ok().is_some_and(|t| t.is_dir()))
        .collect();
    entries.sort_by_key(|entry| {
        entry
            .metadata()
            .ok()
            .and_then(|meta| meta.modified().ok())
    });
    entries
        .pop()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|id| looks_like_session_id(id))
}

fn looks_like_session_id(value: &str) -> bool {
    value.len() >= 8 && value.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

fn cursor_project_slug(cwd: &Path) -> String {
    cwd.to_string_lossy()
        .trim_start_matches('/')
        .replace('/', "-")
}

fn claude_project_slug(cwd: &Path) -> String {
    format!(
        "-{}",
        cwd.to_string_lossy().trim_start_matches('/').replace('/', "-")
    )
}

fn heuristic_session_id(text: &str) -> Option<String> {
    for needle in [
        "\"session_id\":\"",
        "\"session_id\": \"",
        "\"chat_id\":\"",
        "\"chat_id\": \"",
        "\"session\":\"",
        "\"session\": \"",
    ] {
        if let Some(rest) = text.split_once(needle)
            && let Some(id) = rest.1.split('"').next()
            && looks_like_session_id(id)
        {
            return Some(id.to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_slug_matches_cursor_projects_layout() {
        assert_eq!(
            cursor_project_slug(Path::new(
                "/home/jeronimorestrepoangel/Documentos/PDP/Plazario"
            )),
            "home-jeronimorestrepoangel-Documentos-PDP-Plazario"
        );
    }

    #[test]
    fn heuristic_finds_session_id_in_json_fragment() {
        let sample = r#"noise {"type":"system","session_id":"96f670e8-613e-4277-9c0e-ef4a9a118683"}"#;
        assert_eq!(
            scan_output_for_session_id(SupportedAgent::Cursor, sample),
            Some("96f670e8-613e-4277-9c0e-ef4a9a118683".into())
        );
    }
}
