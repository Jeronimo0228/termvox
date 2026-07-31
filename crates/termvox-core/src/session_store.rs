use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{AgentKind, Result, TermVoxError};

const STORE_VERSION: u32 = 1;

/// On-disk workspace session metadata for resuming agent chats across `termvox` runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSession {
    pub version: u32,
    pub agent: AgentKind,
    pub cwd: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termvox_session_id: Option<String>,
    pub updated_at: u64,
}

impl WorkspaceSession {
    #[must_use]
    pub fn new(agent: AgentKind, cwd: PathBuf) -> Self {
        Self {
            version: STORE_VERSION,
            agent,
            cwd,
            remote_id: None,
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
        let session: Self = serde_json::from_str(&content)
            .map_err(|error| TermVoxError::Config(format!("parse {}: {error}", path.display())))?;
        if session.version != STORE_VERSION {
            return Ok(None);
        }
        Ok(Some(session))
    }

    /// Persist the workspace session atomically to the configured path.
    ///
    /// # Errors
    ///
    /// Returns an error when the destination directory or file cannot be written.
    pub fn save(&mut self, path: &Path) -> Result<()> {
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

    pub fn touch_remote_id(&mut self, remote_id: impl Into<String>) {
        self.remote_id = Some(remote_id.into());
    }

    #[must_use]
    pub fn matches(&self, agent: AgentKind, cwd: &Path) -> bool {
        self.agent == agent && self.cwd == cwd
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
    fn round_trips_workspace_session() {
        let dir = std::env::temp_dir().join(format!("termvox-session-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(".termvox/session.json");
        let mut session = WorkspaceSession::new(AgentKind::Cursor, dir.clone());
        session.touch_remote_id("sess_abc");
        session.save(&path).expect("save");
        let loaded = WorkspaceSession::load(&path).expect("load").expect("some");
        assert_eq!(loaded.remote_id.as_deref(), Some("sess_abc"));
        assert_eq!(loaded.agent, AgentKind::Cursor);
        let _ = std::fs::remove_dir_all(dir);
    }
}
