//! Shared contracts, configuration, pipeline, and safety policy for `TermVox`.

mod agents;
mod auth;
mod config;
mod environment;
mod events;
mod performance;
mod pipeline;
mod policy;
mod sessions;

pub use agents::*;
pub use auth::*;
pub use config::*;
pub use environment::*;
pub use events::*;
pub use performance::*;
pub use pipeline::*;
pub use policy::*;
pub use sessions::*;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, TermVoxError>;

#[derive(Debug, Error)]
pub enum TermVoxError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("speech engine error: {0}")]
    Speech(String),
    #[error("agent error: {0}")]
    Agent(String),
    #[error("operation cancelled")]
    Cancelled,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
