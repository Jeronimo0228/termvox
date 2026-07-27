use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{AgentEventStream, PermissionProfile, Result, RuntimeLimits};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentInfo {
    pub id: String,
    pub executable: String,
    pub installed: bool,
    pub version: Option<String>,
    pub capabilities: AgentCapabilities,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentCapabilities {
    pub structured_output: bool,
    pub streaming: bool,
    pub resume: bool,
}

#[derive(Debug, Clone)]
pub struct AgentSession {
    pub id: Uuid,
    remote_id: Arc<RwLock<Option<String>>>,
}

impl Default for AgentSession {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            remote_id: Arc::new(RwLock::new(None)),
        }
    }
}

impl AgentSession {
    pub async fn remote_id(&self) -> Option<String> {
        self.remote_id.read().await.clone()
    }

    pub async fn set_remote_id(&self, remote_id: String) {
        *self.remote_id.write().await = Some(remote_id);
    }
}

#[derive(Debug, Clone)]
pub struct AgentRequest {
    pub prompt: String,
    pub cwd: PathBuf,
    pub limits: RuntimeLimits,
    pub permission_profile: PermissionProfile,
}

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    async fn probe(&self) -> AgentInfo;
    async fn start(&self) -> Result<AgentSession>;
    async fn send_prompt(
        &self,
        session: &AgentSession,
        request: AgentRequest,
        cancel: CancellationToken,
    ) -> Result<AgentEventStream>;
    async fn cancel(&self, _session: &AgentSession) -> Result<()> {
        Ok(())
    }
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
