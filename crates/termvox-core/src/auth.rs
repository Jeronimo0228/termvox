use serde::{Deserialize, Serialize};

/// Upstream agent authentication state reported by `termvox doctor` and shell preflight.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentAuthStatus {
    pub ok: bool,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_command: Option<String>,
}

impl AgentAuthStatus {
    #[must_use]
    pub fn unknown(detail: impl Into<String>) -> Self {
        Self {
            ok: true,
            detail: detail.into(),
            login_command: None,
        }
    }

    #[must_use]
    pub fn authenticated(detail: impl Into<String>) -> Self {
        Self {
            ok: true,
            detail: detail.into(),
            login_command: None,
        }
    }

    #[must_use]
    pub fn unauthenticated(detail: impl Into<String>, login_command: impl Into<String>) -> Self {
        Self {
            ok: false,
            detail: detail.into(),
            login_command: Some(login_command.into()),
        }
    }
}
