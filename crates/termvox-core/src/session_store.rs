use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{AgentKind, Result, TermVoxError};

const STORE_VERSION: u32 = 2;

/// On-disk workspace session metadata for resuming agent chats across `termvox` runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSession {
    pub version: u32,
    pub cwd: PathBuf,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub agents: BTreeMap<AgentKind, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termvox_session_id: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Deserialize)]
struct WorkspaceSessionV1 {
    #[allow(dead_code)]
    version: u32,
    agent: AgentKind,
    cwd: PathBuf,
    #[serde(default)]
    remote_id: Option<String>,
    #[serde(default)]
    termvox_session_id: Option<String>,
    updated_at: u64,
}

impl WorkspaceSession {
    #[must_use]
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            version: STORE_VERSION,
            cwd,
            agents: BTreeMap::new(),
            termvox_session_id: None,
            updated_at: unix_now(),
        }
    }

    #[must_use]
    pub fn session_path(cwd: &Path, relative: Option<&Path>) -> PathBuf {
        relative.map_or_else(|| cwd.join(".termvox/session.json"), Path::to_path_buf)
    }

    /// Load a workspace session from disk when present and compatible.
    ///
    /// # Errors
    ///
    /// Returns an error when the file exists but cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Option<Self>> {
        if !path.is_file() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(path)
            .map_err(|error| TermVoxError::Config(format!("read {}: {error}", path.display())))?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|error| TermVoxError::Config(format!("parse {}: {error}", path.display())))?;
        let version = value
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1) as u32;
        match version {
            STORE_VERSION => {
                let session: Self = serde_json::from_value(value).map_err(|error| {
                    TermVoxError::Config(format!("parse {}: {error}", path.display()))
                })?;
                Ok(Some(session))
            }
            1 => {
                let legacy: WorkspaceSessionV1 = serde_json::from_value(value).map_err(
                    |error| TermVoxError::Config(format!("parse {}: {error}", path.display())),
                )?;
                Ok(Some(migrate_v1(legacy)))
            }
            _ => Ok(None),
        }
    }

    /// Persist the workspace session atomically to the configured path.
    ///
    /// # Errors
    ///
    /// Returns an error when the destination directory or file cannot be written.
    pub fn save(&mut self, path: &Path) -> Result<()> {
        self.version = STORE_VERSION;
        self.updated_at = unix_now();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                TermVoxError::Config(format!("create {}: {error}", parent.display()))
            })?;
        }
        let payload = serde_json::to_string_pretty(self)
            .map_err(|error| TermVoxError::Config(error.to_string()))?;
        std::fs::write(path, payload)
            .map_err(|error| TermVoxError::Config(format!("write {}: {error}", path.display())))
    }

    #[must_use]
    pub fn remote_id(&self, agent: AgentKind) -> Option<&str> {
        self.agents.get(&agent).map(String::as_str)
    }

    pub fn set_remote_id(&mut self, agent: AgentKind, remote_id: impl Into<String>) {
        self.agents.insert(agent, remote_id.into());
    }

    pub fn clear_remote_id(&mut self, agent: AgentKind) {
        self.agents.remove(&agent);
    }

    #[must_use]
    pub fn matches_workspace(&self, cwd: &Path) -> bool {
        let stored = self
            .cwd
            .canonicalize()
            .unwrap_or_else(|_| self.cwd.clone());
        let current = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
        stored == current
    }
}

fn migrate_v1(legacy: WorkspaceSessionV1) -> WorkspaceSession {
    let mut agents = BTreeMap::new();
    if let Some(id) = legacy.remote_id.filter(|value| !value.is_empty()) {
        agents.insert(legacy.agent, id);
    }
    WorkspaceSession {
        version: STORE_VERSION,
        cwd: legacy.cwd,
        agents,
        termvox_session_id: legacy.termvox_session_id,
        updated_at: legacy.updated_at,
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_canonicalizes_workspace_root() {
        let dir = std::env::temp_dir().join(format!("termvox-session-match-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let session = WorkspaceSession::new(dir.clone());
        assert!(session.matches_workspace(&dir));
        #[cfg(unix)]
        {
            let link = dir.with_extension("link");
            let _ = std::fs::remove_file(&link);
            std::os::unix::fs::symlink(&dir, &link).expect("symlink");
            assert!(session.matches_workspace(&link));
            let _ = std::fs::remove_file(link);
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn stores_remote_ids_per_agent_in_one_workspace() {
        let dir = std::env::temp_dir().join(format!("termvox-session-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(".termvox/session.json");
        let mut session = WorkspaceSession::new(dir.clone());
        session.set_remote_id(AgentKind::Cursor, "cursor_sess");
        session.set_remote_id(AgentKind::OpenCode, "opencode_sess");
        session.save(&path).expect("save");
        let loaded = WorkspaceSession::load(&path).expect("load").expect("some");
        assert_eq!(loaded.remote_id(AgentKind::Cursor), Some("cursor_sess"));
        assert_eq!(loaded.remote_id(AgentKind::OpenCode), Some("opencode_sess"));
        assert_eq!(loaded.version, STORE_VERSION);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn migrates_v1_session_files() {
        let dir = std::env::temp_dir().join(format!("termvox-session-v1-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join(".termvox")).expect("termvox dir");
        let path = dir.join(".termvox/session.json");
        std::fs::write(
            &path,
            r#"{
  "version": 1,
  "agent": "opencode",
  "cwd": "/tmp/project",
  "remote_id": "ses_legacy",
  "updated_at": 1
}"#,
        )
        .expect("write");
        let loaded = WorkspaceSession::load(&path).expect("load").expect("some");
        assert_eq!(loaded.remote_id(AgentKind::OpenCode), Some("ses_legacy"));
        assert!(loaded.remote_id(AgentKind::Cursor).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }
}
