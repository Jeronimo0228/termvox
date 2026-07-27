use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::Result;

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl AudioBuffer {
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn duration_seconds(&self) -> f32 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.samples.len() as f32 / self.sample_rate as f32
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Transcription {
    pub text: String,
    pub language: Option<String>,
    pub duration_ms: u64,
    pub segments: Vec<TranscriptSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranscriptSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct TranscriptionOptions {
    pub language: Option<String>,
    pub initial_prompt: Option<String>,
}

#[async_trait]
pub trait SpeechEngine: Send + Sync {
    fn id(&self) -> &'static str;
    async fn healthcheck(&self) -> Result<()>;
    async fn prewarm(&self) -> Result<()> {
        Ok(())
    }
    async fn transcribe(
        &self,
        audio: AudioBuffer,
        options: &TranscriptionOptions,
        cancel: CancellationToken,
    ) -> Result<Transcription>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    Started { session_id: Option<String> },
    Message { text: String },
    Tool { name: String, status: String },
    Completed { result: String },
    Failed { message: String },
}

pub type AgentEventStream = mpsc::Receiver<Result<AgentEvent>>;
