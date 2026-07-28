use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::{AgentsConfig, LegacyCursorConfig, Result, TermVoxError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SpeechEngineKind {
    #[serde(rename = "whisper", alias = "whispercpp")]
    WhisperCpp,
    OpenAi,
    Parakeet,
    Vosk,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentKind {
    Codex,
    Claude,
    Cursor,
    Gemini,
    Aider,
    Amp,
    OpenCode,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionProfile {
    #[default]
    Safe,
    WorkspaceWrite,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimeLimits {
    pub agent_timeout_seconds: u64,
    pub speech_timeout_seconds: u64,
    pub shutdown_timeout_seconds: u64,
    pub max_output_bytes: usize,
    pub max_json_frame_bytes: usize,
    /// Prefer toggle push-to-talk on Wayland when hold-to-talk is unreliable.
    pub auto_toggle_on_wayland: bool,
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            agent_timeout_seconds: 900,
            speech_timeout_seconds: 180,
            shutdown_timeout_seconds: 5,
            max_output_bytes: 8 * 1024 * 1024,
            max_json_frame_bytes: 1024 * 1024,
            auto_toggle_on_wayland: true,
        }
    }
}

impl RuntimeLimits {
    #[must_use]
    pub const fn agent_timeout(&self) -> Duration {
        Duration::from_secs(self.agent_timeout_seconds)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PerformanceProfile {
    /// Lowest RAM and latency — recommended default for voice commands.
    #[default]
    Fast,
    /// Balance between speed and accuracy.
    Balanced,
    /// Highest accuracy, higher RAM and latency.
    Accurate,
    /// User manages individual tuning keys.
    Custom,
}

impl PerformanceProfile {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Balanced => "balanced",
            Self::Accurate => "accurate",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub performance_profile: PerformanceProfile,
    pub speech_engine: SpeechEngineKind,
    pub agent: AgentKind,
    pub agent_plugin: Option<String>,
    pub push_to_talk: String,
    pub language: String,
    pub auto_send: bool,
    pub confirmation: bool,
    pub permission_profile: PermissionProfile,
    pub audio: AudioConfig,
    pub runtime: RuntimeLimits,
    pub whisper: WhisperConfig,
    pub openai: OpenAiConfig,
    pub agents: AgentsConfig,
    pub parakeet: SidecarSpeechConfig,
    pub vosk: SidecarSpeechConfig,
    pub plugins: Vec<PluginConfig>,
    pub pipeline: PipelineConfig,
    pub daemon: DaemonConfig,
    pub shell: ShellConfig,
    pub telemetry: TelemetryConfig,
    /// Apply host hints (Wayland toggle, RAM profile suggestions).
    pub auto_tune_from_environment: bool,
    /// Deprecated: use `[agents.cursor]`. Kept for backward-compatible loading only.
    #[serde(default, skip_serializing)]
    cursor: Option<LegacyCursorConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            performance_profile: PerformanceProfile::Fast,
            speech_engine: SpeechEngineKind::WhisperCpp,
            agent: AgentKind::Codex,
            agent_plugin: None,
            push_to_talk: "SPACE".into(),
            language: "es".into(),
            auto_send: false,
            confirmation: true,
            permission_profile: PermissionProfile::Safe,
            audio: AudioConfig::default(),
            runtime: RuntimeLimits::default(),
            whisper: WhisperConfig::default(),
            openai: OpenAiConfig::default(),
            agents: AgentsConfig::default(),
            parakeet: SidecarSpeechConfig::parakeet_default(),
            vosk: SidecarSpeechConfig::vosk_default(),
            plugins: Vec::new(),
            pipeline: PipelineConfig::default(),
            daemon: DaemonConfig::default(),
            shell: ShellConfig::default(),
            telemetry: TelemetryConfig::default(),
            auto_tune_from_environment: true,
            cursor: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    pub device: Option<String>,
    pub sample_rate: u32,
    pub max_seconds: u32,
    pub vad_threshold_db: f32,
    pub vad_silence_ms: u64,
    /// Stop recording automatically after trailing silence (toggle mode).
    pub auto_stop_on_silence: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            device: None,
            sample_rate: 16_000,
            max_seconds: 30,
            vad_threshold_db: -45.0,
            vad_silence_ms: 400,
            auto_stop_on_silence: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct WhisperConfig {
    pub model: PathBuf,
    /// Decoder worker count. Zero selects the available CPU parallelism.
    pub threads: usize,
    /// Cap automatic thread selection to limit CPU/RAM overhead.
    pub max_threads: usize,
    /// Load the model during startup instead of the first utterance.
    pub prewarm_on_start: bool,
    /// Tune Whisper for short voice commands (lower latency, less context RAM).
    pub optimize_for_latency: bool,
    /// Use GPU acceleration when the Whisper build supports it.
    pub use_gpu: bool,
    /// Emit partial transcript segments while Whisper decodes.
    pub streaming: bool,
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            model: crate::default_whisper_model(PerformanceProfile::Fast),
            threads: 0,
            max_threads: 4,
            prewarm_on_start: false,
            optimize_for_latency: true,
            use_gpu: false,
            streaming: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    pub hotkey: String,
    pub skip_confirmation: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            hotkey: "ALT+SPACE".into(),
            skip_confirmation: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellConfig {
    /// Toggle voice capture inside `termvox shell` (function keys recommended).
    pub hotkey: String,
    /// Send Enter after injecting transcribed text into the agent TUI.
    pub auto_submit: bool,
    /// Skip confirmation before injecting (overrides global confirmation when true).
    pub skip_confirmation: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            hotkey: "F8".into(),
            auto_submit: true,
            skip_confirmation: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub local_metrics_path: PathBuf,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            local_metrics_path: data_path("termvox/metrics.jsonl"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenAiConfig {
    pub api_key_env: String,
    pub model: String,
    pub endpoint: String,
    pub timeout_seconds: u64,
    pub max_response_bytes: usize,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key_env: "OPENAI_API_KEY".into(),
            model: "gpt-4o-mini-transcribe".into(),
            endpoint: "https://api.openai.com/v1/audio/transcriptions".into(),
            timeout_seconds: 180,
            max_response_bytes: 2 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SidecarSpeechConfig {
    pub executable: String,
    pub model: PathBuf,
}

impl SidecarSpeechConfig {
    fn parakeet_default() -> Self {
        Self {
            executable: "termvox-parakeet".into(),
            model: data_path("termvox/models/parakeet"),
        }
    }

    fn vosk_default() -> Self {
        Self {
            executable: "termvox-vosk".into(),
            model: data_path("termvox/models/vosk"),
        }
    }
}

impl Default for SidecarSpeechConfig {
    fn default() -> Self {
        Self::parakeet_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    pub id: String,
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub enabled: bool,
    pub env_allowlist: Vec<String>,
    pub timeout_seconds: u64,
    pub max_frame_bytes: usize,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            executable: PathBuf::new(),
            args: Vec::new(),
            enabled: true,
            env_allowlist: Vec::new(),
            timeout_seconds: 30,
            max_frame_bytes: 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineConfig {
    pub dictionary: BTreeMap<String, String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

impl AppConfig {
    /// Loads and merges global, project, and environment configuration.
    ///
    /// # Errors
    ///
    /// Returns an error when a file cannot be read or a layer is invalid.
    pub fn load(global: Option<&Path>, project: Option<&Path>) -> Result<Self> {
        let mut merged = toml::Value::try_from(Self::default())
            .map_err(|error| TermVoxError::Config(error.to_string()))?;
        for path in [global, project].into_iter().flatten() {
            if path.exists() {
                let text = std::fs::read_to_string(path)?;
                let layer = toml::from_str::<toml::Value>(&text)
                    .map_err(|error| TermVoxError::Config(error.to_string()))?;
                merge_toml(&mut merged, layer);
            }
        }
        apply_env(&mut merged);
        let mut config: AppConfig = merged
            .try_into()
            .map_err(|error| TermVoxError::Config(error.to_string()))?;
        config.normalize_legacy();
        crate::apply_performance_profile(&mut config);
        crate::apply_environment_hints(&mut config);
        Ok(config)
    }

    fn normalize_legacy(&mut self) {
        if let Some(legacy) = self.cursor.take()
            && legacy.trust_workspace
        {
            self.agents.cursor.trust_workspace = true;
        }
    }

    /// Validates all bounded runtime and provider settings.
    ///
    /// # Errors
    ///
    /// Returns a configuration error for unsafe or inconsistent values.
    pub fn validate(&self) -> Result<()> {
        if !(8_000..=192_000).contains(&self.audio.sample_rate) {
            return Err(config_error(
                "audio.sample_rate must be between 8000 and 192000",
            ));
        }
        if self.audio.max_seconds == 0 {
            return Err(config_error("audio.max_seconds must be positive"));
        }
        if self.runtime.max_output_bytes == 0 || self.runtime.max_json_frame_bytes == 0 {
            return Err(config_error("runtime byte limits must be positive"));
        }
        for plugin in &self.plugins {
            if plugin.id.trim().is_empty() || plugin.executable.as_os_str().is_empty() {
                return Err(config_error("plugins require non-empty id and executable"));
            }
        }
        Ok(())
    }
}

fn data_path(relative: &str) -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(relative)
}

fn config_error(message: &str) -> TermVoxError {
    TermVoxError::Config(message.into())
}

fn merge_toml(base: &mut toml::Value, layer: toml::Value) {
    match (base, layer) {
        (toml::Value::Table(base), toml::Value::Table(layer)) => {
            for (key, value) in layer {
                if let Some(existing) = base.get_mut(&key) {
                    merge_toml(existing, value);
                } else {
                    base.insert(key, value);
                }
            }
        }
        (base, layer) => *base = layer,
    }
}

fn apply_env(config: &mut toml::Value) {
    let Some(table) = config.as_table_mut() else {
        return;
    };
    if let Ok(value) = std::env::var("TERMVOX_AGENT") {
        table.insert("agent".into(), toml::Value::String(value.to_lowercase()));
    }
    if let Ok(value) = std::env::var("TERMVOX_SPEECH_ENGINE") {
        table.insert(
            "speech_engine".into(),
            toml::Value::String(value.to_lowercase()),
        );
    }
    if let Ok(value) = std::env::var("TERMVOX_LANGUAGE") {
        table.insert("language".into(), toml::Value::String(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn loads_and_merges_project_toml() {
        let path = std::env::temp_dir().join(format!("termvox-{}.toml", Uuid::new_v4()));
        std::fs::write(&path, "language = \"en\"\n[audio]\nmax_seconds = 42\n").unwrap();
        let config = AppConfig::load(None, Some(&path)).unwrap();
        let _ = std::fs::remove_file(path);
        assert_eq!(config.language, "en");
        assert_eq!(config.audio.max_seconds, 42);
        assert_eq!(config.audio.sample_rate, 16_000);
    }

    #[test]
    fn validates_audio_boundaries() {
        for sample_rate in [8_000, 192_000] {
            let mut config = AppConfig::default();
            config.audio.sample_rate = sample_rate;
            assert!(config.validate().is_ok());
        }
        for sample_rate in [0, 7_999, 192_001] {
            let mut config = AppConfig::default();
            config.audio.sample_rate = sample_rate;
            assert!(config.validate().is_err());
        }
    }

    #[test]
    fn rejects_zero_runtime_byte_limits() {
        let mut config = AppConfig::default();
        config.runtime.max_output_bytes = 0;
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.runtime.max_json_frame_bytes = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn accepts_legacy_whispercpp_name_but_serializes_whisper() {
        let config: AppConfig = toml::from_str("speech_engine = \"whispercpp\"").unwrap();
        assert_eq!(config.speech_engine, SpeechEngineKind::WhisperCpp);
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("speech_engine = \"whisper\""));
        assert!(!serialized.contains("whispercpp"));
    }

    #[test]
    fn loads_legacy_cursor_trust_into_agents_profile() {
        let path =
            std::env::temp_dir().join(format!("termvox-legacy-cursor-{}.toml", std::process::id()));
        std::fs::write(
            &path,
            r#"
            agent = "cursor"
            [cursor]
            trust_workspace = true
            "#,
        )
        .unwrap();
        let config = AppConfig::load(None, Some(&path)).unwrap();
        let _ = std::fs::remove_file(path);
        assert!(config.agents.cursor.trust_workspace);
    }
}
