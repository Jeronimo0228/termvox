use std::{fs::OpenOptions, io::Write, path::PathBuf, time::SystemTime};

use anyhow::Result;
use serde::Serialize;
use termvox_core::{AppConfig, PerformanceProfile};

#[derive(Serialize)]
struct UtteranceMetric {
    ts_ms: u128,
    agent: String,
    performance_profile: String,
    transcript_ms: u64,
    audio_seconds: f32,
    delivery: Option<String>,
}

pub(crate) fn record_utterance(
    config: &AppConfig,
    transcript_ms: u64,
    audio_seconds: f32,
    delivery: Option<&str>,
) -> Result<()> {
    if !config.telemetry.enabled {
        return Ok(());
    }
    let metric = UtteranceMetric {
        ts_ms: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis()),
        agent: config.agent.id().into(),
        performance_profile: profile_label(config.performance_profile),
        transcript_ms,
        audio_seconds,
        delivery: delivery.map(str::to_owned),
    };
    if let Some(parent) = config.telemetry.local_metrics_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.telemetry.local_metrics_path)?;
    serde_json::to_writer(&mut file, &metric)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn profile_label(profile: PerformanceProfile) -> String {
    profile.as_str().into()
}

pub(crate) fn default_metrics_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("termvox/metrics.jsonl")
}
