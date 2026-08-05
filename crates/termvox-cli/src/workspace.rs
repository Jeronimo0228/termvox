//! Workspace session persistence shared by shell and subprocess runtimes.

use std::path::{Path, PathBuf};

use termvox_agents::{SupportedAgent, discover_remote_session};
use termvox_core::{AgentKind, AgentSession, AppConfig, WorkspaceSession};

#[must_use]
pub fn session_path(config: &AppConfig, cwd: &Path) -> PathBuf {
    WorkspaceSession::session_path(cwd, config.workspace.session_file.as_deref())
}

#[must_use]
pub fn canonical_workspace(cwd: &Path) -> PathBuf {
    cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf())
}

#[must_use]
pub fn load_remote_id(config: &AppConfig, agent: AgentKind, cwd: &Path) -> Option<String> {
    if !config.workspace.persist_session {
        return None;
    }
    let cwd = canonical_workspace(cwd);
    let path = session_path(config, &cwd);
    if let Ok(Some(session)) = WorkspaceSession::load(&path) {
        if session.matches_workspace(&cwd) {
            if let Some(id) = session.remote_id(agent) {
                return Some(id.to_owned());
            }
        }
    }
    if config.workspace.discover_session {
        discover_remote_session(to_supported(agent), &cwd)
    } else {
        None
    }
}

pub fn persist_remote_id(
    config: &AppConfig,
    agent: AgentKind,
    cwd: &Path,
    remote_id: Option<String>,
) {
    if !config.workspace.persist_session {
        return;
    }
    let cwd = canonical_workspace(cwd);
    let path = session_path(config, &cwd);
    let mut session = WorkspaceSession::load(&path)
        .ok()
        .flatten()
        .filter(|stored| stored.matches_workspace(&cwd))
        .unwrap_or_else(|| WorkspaceSession::new(cwd.clone()));
    session.cwd = cwd;
    match remote_id.filter(|value| !value.is_empty()) {
        Some(id) => session.set_remote_id(agent, id),
        None => session.clear_remote_id(agent),
    }
    if let Err(error) = session.save(&path) {
        tracing::warn!(%error, "failed to persist workspace session");
    }
}

pub async fn hydrate_agent_session(
    config: &AppConfig,
    agent: AgentKind,
    session: &AgentSession,
    fresh: bool,
) {
    if fresh || !config.workspace.persist_session {
        return;
    }
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    if let Some(id) = load_remote_id(config, agent, &cwd) {
        session.set_remote_id(id).await;
    }
}

pub async fn save_agent_session(config: &AppConfig, agent: AgentKind, session: &AgentSession) {
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    let remote_id = session.remote_id().await;
    persist_remote_id(config, agent, &cwd, remote_id);
}

const fn to_supported(agent: AgentKind) -> SupportedAgent {
    match agent {
        AgentKind::Codex => SupportedAgent::Codex,
        AgentKind::Claude => SupportedAgent::Claude,
        AgentKind::Cursor => SupportedAgent::Cursor,
        AgentKind::Gemini => SupportedAgent::Gemini,
        AgentKind::Aider => SupportedAgent::Aider,
        AgentKind::Amp => SupportedAgent::Amp,
        AgentKind::OpenCode => SupportedAgent::OpenCode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persist_keeps_other_agents_in_same_workspace() {
        let dir = std::env::temp_dir().join(format!("termvox-ws-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let config = AppConfig::default();
        persist_remote_id(
            &config,
            AgentKind::Cursor,
            &dir,
            Some("cursor_12345678".into()),
        );
        persist_remote_id(
            &config,
            AgentKind::OpenCode,
            &dir,
            Some("opencode_12345678".into()),
        );
        assert_eq!(
            load_remote_id(&config, AgentKind::Cursor, &dir),
            Some("cursor_12345678".into())
        );
        assert_eq!(
            load_remote_id(&config, AgentKind::OpenCode, &dir),
            Some("opencode_12345678".into())
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
