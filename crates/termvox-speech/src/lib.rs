//! Interchangeable local and remote speech-to-text engines.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc
)]

use std::{
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use termvox_core::{
    AudioBuffer, OpenAiConfig, Result, SidecarSpeechConfig, SpeechEngine, TermVoxError,
    TranscriptSegment, Transcription, TranscriptionOptions, WhisperConfig,
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    process::Command,
};
use tokio_util::sync::CancellationToken;

pub struct WhisperCppEngine {
    config: WhisperConfig,
}

impl WhisperCppEngine {
    #[must_use]
    pub fn new(config: WhisperConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl SpeechEngine for WhisperCppEngine {
    fn id(&self) -> &'static str {
        "whispercpp"
    }

    async fn healthcheck(&self) -> Result<()> {
        if which_executable(&self.config.executable).is_none() {
            return Err(TermVoxError::Speech(format!(
                "{} was not found in PATH",
                self.config.executable
            )));
        }
        if !self.config.model.is_file() {
            return Err(TermVoxError::Speech(format!(
                "Whisper model not found: {}",
                self.config.model.display()
            )));
        }
        Ok(())
    }

    async fn transcribe(
        &self,
        audio: AudioBuffer,
        options: &TranscriptionOptions,
        cancel: CancellationToken,
    ) -> Result<Transcription> {
        self.healthcheck().await?;
        if audio.samples.is_empty() {
            return Err(TermVoxError::Speech("audio contains no voice".into()));
        }
        let started = Instant::now();
        let stem = temp_stem("termvox-whisper");
        let wav_path = stem.with_extension("wav");
        let output_path = stem.with_extension("txt");
        fs::write(&wav_path, encode_wav(&audio)?).await?;

        let mut command = Command::new(&self.config.executable);
        command
            .arg("-m")
            .arg(&self.config.model)
            .arg("-f")
            .arg(&wav_path)
            .arg("-otxt")
            .arg("-of")
            .arg(&stem)
            .arg("-np")
            .kill_on_drop(true);
        if let Some(language) = &options.language {
            command.arg("-l").arg(language);
        }
        if let Some(prompt) = &options.initial_prompt {
            command.arg("--prompt").arg(prompt);
        }

        let output = tokio::select! {
            () = cancel.cancelled() => {
                cleanup(&[&wav_path, &output_path]).await;
                return Err(TermVoxError::Cancelled);
            }
            output = command.output() => output?,
        };
        if !output.status.success() {
            cleanup(&[&wav_path, &output_path]).await;
            return Err(TermVoxError::Speech(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }
        let text = fs::read_to_string(&output_path)
            .await
            .or_else(|_| String::from_utf8(output.stdout).map_err(std::io::Error::other))?
            .trim()
            .to_owned();
        cleanup(&[&wav_path, &output_path]).await;
        Ok(Transcription {
            text,
            language: options.language.clone(),
            duration_ms: started.elapsed().as_millis() as u64,
            segments: Vec::new(),
        })
    }
}

pub struct OpenAiSpeechEngine {
    config: OpenAiConfig,
    client: reqwest::Client,
}

impl OpenAiSpeechEngine {
    #[must_use]
    pub fn new(config: OpenAiConfig) -> Self {
        if !config.endpoint.starts_with("https://api.openai.com/") {
            tracing::warn!(
                endpoint = %config.endpoint,
                "using a custom OpenAI-compatible endpoint; audio and credentials are sent to it"
            );
        }
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SpeechEngine for OpenAiSpeechEngine {
    fn id(&self) -> &'static str {
        "openai"
    }

    async fn healthcheck(&self) -> Result<()> {
        std::env::var(&self.config.api_key_env)
            .map_err(|_| TermVoxError::Speech(format!("{} is not set", self.config.api_key_env)))?;
        Ok(())
    }

    async fn transcribe(
        &self,
        audio: AudioBuffer,
        options: &TranscriptionOptions,
        cancel: CancellationToken,
    ) -> Result<Transcription> {
        self.healthcheck().await?;
        let started = Instant::now();
        let api_key = std::env::var(&self.config.api_key_env)
            .map_err(|error| TermVoxError::Speech(error.to_string()))?;
        let audio_part = Part::bytes(encode_wav(&audio)?)
            .file_name("termvox.wav")
            .mime_str("audio/wav")
            .map_err(|error| TermVoxError::Speech(error.to_string()))?;
        let mut form = Form::new()
            .text("model", self.config.model.clone())
            .text("response_format", "verbose_json")
            .part("file", audio_part);
        if let Some(language) = &options.language {
            form = form.text("language", language.clone());
        }
        if let Some(prompt) = &options.initial_prompt {
            form = form.text("prompt", prompt.clone());
        }
        let request = self
            .client
            .post(&self.config.endpoint)
            .bearer_auth(api_key)
            .multipart(form)
            .send();
        let response = tokio::select! {
            () = cancel.cancelled() => return Err(TermVoxError::Cancelled),
            response = tokio::time::timeout(
                std::time::Duration::from_secs(self.config.timeout_seconds),
                request,
            ) => response
                .map_err(|_| TermVoxError::Speech("OpenAI transcription timed out".into()))?
                .map_err(|error| TermVoxError::Speech(error.to_string()))?,
        };
        let status = response.status();
        let mut response = response;
        let mut bytes = Vec::new();
        loop {
            let chunk = tokio::select! {
                () = cancel.cancelled() => return Err(TermVoxError::Cancelled),
                chunk = tokio::time::timeout(
                    std::time::Duration::from_secs(self.config.timeout_seconds),
                    response.chunk(),
                ) => chunk
                    .map_err(|_| TermVoxError::Speech("OpenAI response body timed out".into()))?
                    .map_err(|error| TermVoxError::Speech(error.to_string()))?,
            };
            let Some(chunk) = chunk else { break };
            if bytes.len().saturating_add(chunk.len()) > self.config.max_response_bytes {
                return Err(TermVoxError::Speech(
                    "OpenAI response exceeded configured limit".into(),
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        let body: Value = serde_json::from_slice(&bytes)
            .map_err(|error| TermVoxError::Speech(error.to_string()))?;
        if !status.is_success() {
            let message = body
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("OpenAI transcription failed");
            return Err(TermVoxError::Speech(message.to_owned()));
        }
        let segments = body
            .get("segments")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|segment| TranscriptSegment {
                start_ms: seconds_to_ms(segment.get("start")),
                end_ms: seconds_to_ms(segment.get("end")),
                text: segment
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            })
            .collect();
        Ok(Transcription {
            text: body
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            language: body
                .get("language")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            duration_ms: started.elapsed().as_millis() as u64,
            segments,
        })
    }
}

pub struct SidecarSpeechEngine {
    id: &'static str,
    config: SidecarSpeechConfig,
}

impl SidecarSpeechEngine {
    #[must_use]
    pub fn parakeet(config: SidecarSpeechConfig) -> Self {
        Self {
            id: "parakeet",
            config,
        }
    }

    #[must_use]
    pub fn vosk(config: SidecarSpeechConfig) -> Self {
        Self { id: "vosk", config }
    }
}

#[async_trait]
impl SpeechEngine for SidecarSpeechEngine {
    fn id(&self) -> &'static str {
        self.id
    }

    async fn healthcheck(&self) -> Result<()> {
        if which_executable(&self.config.executable).is_none() {
            return Err(TermVoxError::Speech(format!(
                "{} sidecar was not found in PATH",
                self.config.executable
            )));
        }
        if !self.config.model.exists() {
            return Err(TermVoxError::Speech(format!(
                "{} model not found: {}",
                self.id,
                self.config.model.display()
            )));
        }
        Ok(())
    }

    async fn transcribe(
        &self,
        audio: AudioBuffer,
        options: &TranscriptionOptions,
        cancel: CancellationToken,
    ) -> Result<Transcription> {
        self.healthcheck().await?;
        let started = Instant::now();
        let stem = temp_stem(self.id);
        let wav_path = stem.with_extension("wav");
        fs::write(&wav_path, encode_wav(&audio)?).await?;
        let mut command = Command::new(&self.config.executable);
        command
            .args(["--input", &wav_path.display().to_string()])
            .args(["--model", &self.config.model.display().to_string()])
            .arg("--json")
            .kill_on_drop(true);
        if let Some(language) = &options.language {
            command.args(["--language", language]);
        }
        let output = tokio::select! {
            () = cancel.cancelled() => {
                cleanup(&[&wav_path]).await;
                return Err(TermVoxError::Cancelled);
            }
            output = command.output() => output?,
        };
        cleanup(&[&wav_path]).await;
        if !output.status.success() {
            return Err(TermVoxError::Speech(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }
        let value: Value = serde_json::from_slice(&output.stdout)
            .map_err(|error| TermVoxError::Speech(error.to_string()))?;
        Ok(Transcription {
            text: value
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            language: value
                .get("language")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| options.language.clone()),
            duration_ms: started.elapsed().as_millis() as u64,
            segments: Vec::new(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelArtifact {
    pub id: String,
    pub provider: String,
    pub version: String,
    pub license: String,
    pub platform: String,
    pub size_bytes: u64,
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelManifest {
    pub schema_version: u32,
    pub artifacts: Vec<ModelArtifact>,
}

impl ModelManifest {
    /// Loads the release-reviewed artifact registry bundled with this crate.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedded manifest is malformed.
    pub fn bundled() -> Result<Self> {
        serde_json::from_str(include_str!("../models/manifest.json"))
            .map_err(|error| TermVoxError::Speech(error.to_string()))
    }

    #[must_use]
    pub fn find(&self, id: &str, platform: &str) -> Option<&ModelArtifact> {
        self.artifacts.iter().find(|artifact| {
            artifact.id == id && (artifact.platform == platform || artifact.platform == "all")
        })
    }
}

pub struct ModelManager {
    client: reqwest::Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub resumed: bool,
}

impl Default for ModelManager {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl ModelManager {
    pub async fn download_verified(
        &self,
        url: &str,
        destination: &Path,
        expected_sha256: &str,
    ) -> Result<()> {
        self.download_verified_with(
            url,
            destination,
            expected_sha256,
            CancellationToken::new(),
            |_| {},
        )
        .await
    }

    pub async fn download_verified_with<F>(
        &self,
        url: &str,
        destination: &Path,
        expected_sha256: &str,
        cancel: CancellationToken,
        mut progress: F,
    ) -> Result<()>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        if expected_sha256.len() != 64 {
            return Err(TermVoxError::Speech(
                "a 64-character SHA-256 checksum is required".into(),
            ));
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).await?;
        }
        let temporary = destination.with_extension("part");
        let existing = fs::metadata(&temporary)
            .await
            .map_or(0, |metadata| metadata.len());
        let mut request = self.client.get(url);
        if existing > 0 {
            request = request.header(reqwest::header::RANGE, format!("bytes={existing}-"));
        }
        let mut response = request
            .send()
            .await
            .map_err(|error| TermVoxError::Speech(error.to_string()))?;
        if existing > 0 && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            fs::remove_file(&temporary).await?;
            response = self
                .client
                .get(url)
                .send()
                .await
                .map_err(|error| TermVoxError::Speech(error.to_string()))?;
        }
        response = response
            .error_for_status()
            .map_err(|error| TermVoxError::Speech(error.to_string()))?;
        let append = fs::metadata(&temporary).await.is_ok()
            && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
        let resumed_bytes = if append { existing } else { 0 };
        let total_bytes = response
            .content_length()
            .map(|remaining| remaining.saturating_add(resumed_bytes));
        let mut output = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(&temporary)
            .await?;
        let mut downloaded = resumed_bytes;
        loop {
            let chunk = tokio::select! {
                () = cancel.cancelled() => return Err(TermVoxError::Cancelled),
                chunk = response.chunk() => chunk
                    .map_err(|error| TermVoxError::Speech(error.to_string()))?,
            };
            let Some(chunk) = chunk else { break };
            output.write_all(&chunk).await?;
            downloaded = downloaded.saturating_add(chunk.len() as u64);
            progress(DownloadProgress {
                downloaded_bytes: downloaded,
                total_bytes,
                resumed: append,
            });
        }
        output.flush().await?;
        output.sync_all().await?;
        drop(output);

        let mut input = fs::File::open(&temporary).await?;
        input.seek(std::io::SeekFrom::Start(0)).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; 64 * 1024];
        loop {
            let read = input.read(&mut buffer).await?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        let actual = hex::encode(hasher.finalize());
        if !actual.eq_ignore_ascii_case(expected_sha256) {
            let _ = fs::remove_file(&temporary).await;
            return Err(TermVoxError::Speech(format!(
                "model checksum mismatch: expected {expected_sha256}, got {actual}"
            )));
        }
        atomic_replace(&temporary, destination).await?;
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
async fn atomic_replace(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(source, destination).await
}

#[cfg(target_os = "windows")]
async fn atomic_replace(source: &Path, destination: &Path) -> std::io::Result<()> {
    // Windows cannot atomically replace an existing path with std::fs. Preserve the
    // verified existing model rather than creating an unprotected replacement gap.
    if destination.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "remove the existing verified model before replacing it",
        ));
    }
    fs::rename(source, destination).await
}

pub fn encode_wav(audio: &AudioBuffer) -> Result<Vec<u8>> {
    let data_len = audio
        .samples
        .len()
        .checked_mul(2)
        .ok_or_else(|| TermVoxError::Speech("audio is too large".into()))?;
    let data_len =
        u32::try_from(data_len).map_err(|_| TermVoxError::Speech("audio is too large".into()))?;
    let mut wav = Vec::with_capacity(data_len as usize + 44);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&audio.sample_rate.to_le_bytes());
    wav.extend_from_slice(&(audio.sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for sample in &audio.samples {
        let pcm = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        wav.extend_from_slice(&pcm.to_le_bytes());
    }
    Ok(wav)
}

fn seconds_to_ms(value: Option<&Value>) -> u64 {
    value
        .and_then(Value::as_f64)
        .unwrap_or_default()
        .mul_add(1000.0, 0.0) as u64
}

fn which_executable(executable: &str) -> Option<PathBuf> {
    let path = Path::new(executable);
    if path.components().count() > 1 && path.is_file() {
        return Some(path.to_path_buf());
    }
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|directory| directory.join(executable))
            .find(|candidate| candidate.is_file())
    })
}

fn temp_stem(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nonce}", std::process::id()))
}

async fn cleanup(paths: &[&Path]) {
    for path in paths {
        let _ = fs::remove_file(path).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_has_valid_header_and_pcm_length() {
        let wav = encode_wav(&AudioBuffer {
            samples: vec![0.0; 16_000],
            sample_rate: 16_000,
        })
        .unwrap();
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(wav.len(), 32_044);
    }

    #[tokio::test]
    async fn model_download_requires_checksum() {
        let error = ModelManager::default()
            .download_verified("https://example.com/model", Path::new("unused"), "")
            .await
            .unwrap_err();
        assert!(error.to_string().contains("checksum"));
    }

    #[test]
    fn bundled_manifest_contains_verified_portable_model() {
        let manifest = ModelManifest::bundled().unwrap();
        let model = manifest
            .find("vosk-model-small-es-0.42", std::env::consts::OS)
            .unwrap();
        assert_eq!(model.sha256.len(), 64);
        assert_eq!(model.license, "Apache-2.0");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn whisper_adapter_invokes_cli_without_a_shell() {
        use std::os::unix::fs::PermissionsExt;

        let stem = temp_stem("termvox-fake-whisper");
        let executable = stem.with_extension("sh");
        let model = stem.with_extension("bin");
        fs::write(
            &executable,
            "#!/bin/sh\nout=''\nwhile [ \"$#\" -gt 0 ]; do\n\
             if [ \"$1\" = '-of' ]; then shift; out=\"$1\"; fi\nshift\ndone\n\
             printf 'hello world\\n' > \"${out}.txt\"\n",
        )
        .await
        .unwrap();
        let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&executable, permissions).unwrap();
        fs::write(&model, b"fake").await.unwrap();
        let engine = WhisperCppEngine::new(WhisperConfig {
            executable: executable.display().to_string(),
            model: model.clone(),
        });
        let result = engine
            .transcribe(
                AudioBuffer {
                    samples: vec![0.1; 160],
                    sample_rate: 16_000,
                },
                &TranscriptionOptions::default(),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        cleanup(&[&executable, &model]).await;
        assert_eq!(result.text, "hello world");
    }
}
