use std::{sync::Arc, time::Instant};

use anyhow::{Result, bail};
use serde::Serialize;
use termvox_core::AppConfig;
use tokio_util::sync::CancellationToken;

use crate::runtime::{ensure_speech_engine, transcription_options};

#[derive(serde::Serialize)]
struct BenchReport {
    profile: String,
    model: String,
    optimize_for_latency: bool,
    use_gpu: bool,
    samples: u32,
    runs: u32,
    p50_ms: u64,
    p95_ms: u64,
    min_ms: u64,
    max_ms: u64,
}

pub(crate) async fn run(config: AppConfig, runs: u32) -> Result<()> {
    if runs == 0 {
        bail!("runs must be positive");
    }
    let speech = ensure_speech_engine(&config).await?;
    speech.prewarm().await?;
    let sample_rate = config.audio.sample_rate;
    let audio = termvox_core::AudioBuffer {
        samples: synthetic_voice_samples(sample_rate, 2.0),
        sample_rate,
    };
    let options = transcription_options(&config);
    let mut timings = Vec::with_capacity(runs as usize);
    for _ in 0..runs {
        let started = Instant::now();
        let _ = speech
            .transcribe(
                termvox_core::AudioBuffer {
                    samples: audio.samples.clone(),
                    sample_rate: audio.sample_rate,
                },
                &options,
                CancellationToken::new(),
            )
            .await?;
        timings.push(started.elapsed().as_millis() as u64);
    }
    timings.sort_unstable();
    let report = BenchReport {
        profile: config.performance_profile.as_str().into(),
        model: config.whisper.model.display().to_string(),
        optimize_for_latency: config.whisper.optimize_for_latency,
        use_gpu: config.whisper.use_gpu,
        samples: audio.samples.len() as u32,
        runs,
        p50_ms: percentile(&timings, 50),
        p95_ms: percentile(&timings, 95),
        min_ms: *timings.first().unwrap_or(&0),
        max_ms: *timings.last().unwrap_or(&0),
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn percentile(values: &[u64], pct: u8) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = (values.len() * pct as usize)
        .div_ceil(100)
        .saturating_sub(1);
    values[index.min(values.len() - 1)]
}

fn synthetic_voice_samples(sample_rate: u32, seconds: f32) -> Vec<f32> {
    let count = (sample_rate as f32 * seconds) as usize;
    (0..count)
        .map(|index| {
            let t = index as f32 / sample_rate as f32;
            (t * 440.0 * std::f32::consts::TAU).sin() * 0.2
        })
        .collect()
}
