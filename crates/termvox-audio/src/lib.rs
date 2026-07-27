//! Cross-platform microphone capture with bounded, non-blocking callbacks.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::too_many_arguments,
    clippy::unused_async
)]

#[cfg(feature = "native-audio")]
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

#[cfg(feature = "native-audio")]
use cpal::{
    Device, SampleFormat, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
#[cfg(feature = "native-audio")]
use crossbeam_queue::ArrayQueue;
use termvox_core::{AudioBuffer, AudioConfig, Result, TermVoxError};
#[cfg(feature = "native-audio")]
use tokio::{sync::mpsc, task::JoinHandle};
#[cfg(feature = "native-audio")]
use tokio_util::sync::CancellationToken;

#[cfg(feature = "native-audio")]
const FRAME_CHANNEL_CAPACITY: usize = 64;
#[cfg(feature = "native-audio")]
const MAX_CALLBACK_SAMPLES: usize = 16_384;

#[cfg(feature = "native-audio")]
pub struct AudioRecorder {
    stream: Stream,
    collector: JoinHandle<()>,
    cancel: CancellationToken,
    samples: Arc<Mutex<Vec<f32>>>,
    source_rate: u32,
    target_rate: u32,
    dropped_frames: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AudioMetrics {
    pub dropped_frames: u64,
}

#[cfg(feature = "native-audio")]
impl AudioRecorder {
    pub fn start(config: &AudioConfig) -> Result<Self> {
        let host = cpal::default_host();
        let device = select_device(&host, config.device.as_deref())?;
        let supported = device
            .default_input_config()
            .map_err(|error| TermVoxError::Audio(error.to_string()))?;
        let sample_format = supported.sample_format();
        let stream_config: StreamConfig = supported.into();
        let channels = usize::from(stream_config.channels);
        let source_rate = stream_config.sample_rate;
        let (tx, mut rx) = mpsc::channel::<Vec<f32>>(FRAME_CHANNEL_CAPACITY);
        let dropped_frames = Arc::new(AtomicU64::new(0));
        let frame_pool = Arc::new(ArrayQueue::new(FRAME_CHANNEL_CAPACITY + 1));
        for _ in 0..=FRAME_CHANNEL_CAPACITY {
            let _ = frame_pool.push(Vec::with_capacity(MAX_CALLBACK_SAMPLES));
        }
        let max_samples = config.max_seconds as usize * source_rate as usize;
        let samples = Arc::new(Mutex::new(Vec::with_capacity(max_samples.min(1_920_000))));
        let collected = Arc::clone(&samples);
        let cancel = CancellationToken::new();
        let collector_cancel = cancel.clone();
        let collector_pool = Arc::clone(&frame_pool);
        let collector = tokio::spawn(async move {
            loop {
                tokio::select! {
                    () = collector_cancel.cancelled() => {
                        while let Ok(frame) = rx.try_recv() {
                            append_bounded(&collected, &frame, max_samples);
                            recycle_frame(&collector_pool, frame);
                        }
                        break;
                    }
                    frame = rx.recv() => {
                        let Some(frame) = frame else { break };
                        append_bounded(&collected, &frame, max_samples);
                        recycle_frame(&collector_pool, frame);
                    }
                }
            }
        });

        let error_callback = |error| tracing::error!(%error, "audio input stream failed");
        let dropped = Arc::clone(&dropped_frames);
        let stream = match sample_format {
            SampleFormat::F32 => build_stream(
                &device,
                &stream_config,
                channels,
                tx,
                Arc::clone(&frame_pool),
                Arc::clone(&dropped),
                error_callback,
                |sample: f32| sample,
            ),
            SampleFormat::I16 => build_stream(
                &device,
                &stream_config,
                channels,
                tx,
                Arc::clone(&frame_pool),
                Arc::clone(&dropped),
                error_callback,
                |sample: i16| f32::from(sample) / f32::from(i16::MAX),
            ),
            SampleFormat::U16 => build_stream(
                &device,
                &stream_config,
                channels,
                tx,
                frame_pool,
                dropped,
                error_callback,
                |sample: u16| (f32::from(sample) / f32::from(u16::MAX)).mul_add(2.0, -1.0),
            ),
            format => Err(TermVoxError::Audio(format!(
                "unsupported input sample format: {format:?}"
            ))),
        }?;
        stream
            .play()
            .map_err(|error| TermVoxError::Audio(error.to_string()))?;

        Ok(Self {
            stream,
            collector,
            cancel,
            samples,
            source_rate,
            target_rate: config.sample_rate,
            dropped_frames,
        })
    }

    #[must_use]
    pub fn metrics(&self) -> AudioMetrics {
        AudioMetrics {
            dropped_frames: self.dropped_frames.load(Ordering::Relaxed),
        }
    }

    pub async fn stop(self) -> Result<AudioBuffer> {
        self.stream
            .pause()
            .map_err(|error| TermVoxError::Audio(error.to_string()))?;
        self.cancel.cancel();
        self.collector
            .await
            .map_err(|error| TermVoxError::Audio(error.to_string()))?;
        let samples = Arc::try_unwrap(self.samples)
            .map_err(|_| TermVoxError::Audio("audio collector is still active".into()))?
            .into_inner()
            .map_err(|_| TermVoxError::Audio("audio buffer lock was poisoned".into()))?;
        Ok(AudioBuffer {
            samples: resample_linear(&samples, self.source_rate, self.target_rate),
            sample_rate: self.target_rate,
        })
    }
}

#[cfg(not(feature = "native-audio"))]
pub struct AudioRecorder;

#[cfg(not(feature = "native-audio"))]
impl AudioRecorder {
    pub fn start(_config: &AudioConfig) -> Result<Self> {
        Err(TermVoxError::Audio(
            "native audio support was disabled at compile time".into(),
        ))
    }

    pub async fn stop(self) -> Result<AudioBuffer> {
        Err(TermVoxError::Audio(
            "native audio support was disabled at compile time".into(),
        ))
    }
}

#[cfg(feature = "native-audio")]
fn select_device(host: &cpal::Host, requested: Option<&str>) -> Result<Device> {
    if let Some(requested) = requested {
        let devices = host
            .input_devices()
            .map_err(|error| TermVoxError::Audio(error.to_string()))?;
        for device in devices {
            if device
                .description()
                .is_ok_and(|description| description.name() == requested)
            {
                return Ok(device);
            }
        }
        return Err(TermVoxError::Audio(format!(
            "input device not found: {requested}"
        )));
    }
    host.default_input_device()
        .ok_or_else(|| TermVoxError::Audio("no default input device".into()))
}

#[cfg(feature = "native-audio")]
fn build_stream<T, E, C>(
    device: &Device,
    config: &StreamConfig,
    channels: usize,
    tx: mpsc::Sender<Vec<f32>>,
    frame_pool: Arc<ArrayQueue<Vec<f32>>>,
    dropped_frames: Arc<AtomicU64>,
    error_callback: E,
    convert: C,
) -> Result<Stream>
where
    T: cpal::SizedSample,
    E: FnMut(cpal::Error) + Send + 'static,
    C: Fn(T) -> f32 + Send + 'static + Copy,
{
    device
        .build_input_stream(
            *config,
            move |data: &[T], _| {
                let sample_count = data.len().div_ceil(channels);
                let Some(mut mono) = frame_pool.pop() else {
                    dropped_frames.fetch_add(1, Ordering::Relaxed);
                    return;
                };
                if sample_count > mono.capacity() {
                    recycle_frame(&frame_pool, mono);
                    dropped_frames.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                mono.clear();
                mono.extend(data.chunks(channels).map(|frame| {
                    frame.iter().copied().map(convert).sum::<f32>() / frame.len() as f32
                }));
                if let Err(error) = tx.try_send(mono) {
                    recycle_frame(&frame_pool, error.into_inner());
                    dropped_frames.fetch_add(1, Ordering::Relaxed);
                }
            },
            error_callback,
            Some(Duration::from_millis(100)),
        )
        .map_err(|error| TermVoxError::Audio(error.to_string()))
}

#[cfg(feature = "native-audio")]
fn recycle_frame(pool: &ArrayQueue<Vec<f32>>, mut frame: Vec<f32>) {
    frame.clear();
    let _ = pool.push(frame);
}

#[cfg(feature = "native-audio")]
fn append_bounded(target: &Mutex<Vec<f32>>, frame: &[f32], max_samples: usize) {
    let Ok(mut target) = target.lock() else {
        return;
    };
    let remaining = max_samples.saturating_sub(target.len());
    target.extend_from_slice(&frame[..frame.len().min(remaining)]);
}

#[cfg(feature = "native-audio")]
pub fn input_devices() -> Result<Vec<String>> {
    cpal::default_host()
        .input_devices()
        .map_err(|error| TermVoxError::Audio(error.to_string()))?
        .map(|device| {
            device
                .description()
                .map(|description| description.name().to_owned())
                .map_err(|error| TermVoxError::Audio(error.to_string()))
        })
        .collect()
}

#[cfg(not(feature = "native-audio"))]
pub fn input_devices() -> Result<Vec<String>> {
    Err(TermVoxError::Audio(
        "native audio support was disabled at compile time".into(),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadDecision {
    Silence,
    SpeechStarted,
    Speech,
    SpeechEnded,
}

#[derive(Debug)]
pub struct VadStateMachine {
    start_threshold_db: f32,
    stop_threshold_db: f32,
    hangover_frames: usize,
    silence_frames: usize,
    speaking: bool,
}

impl VadStateMachine {
    #[must_use]
    pub fn new(threshold_db: f32, silence_ms: u64, frame_ms: u64) -> Self {
        Self {
            start_threshold_db: threshold_db,
            stop_threshold_db: threshold_db - 3.0,
            hangover_frames: usize::try_from(silence_ms / frame_ms.max(1)).unwrap_or(usize::MAX),
            silence_frames: 0,
            speaking: false,
        }
    }

    pub fn process(&mut self, frame: &[f32]) -> VadDecision {
        let db = frame_db(frame);
        if !self.speaking {
            if db >= self.start_threshold_db {
                self.speaking = true;
                self.silence_frames = 0;
                VadDecision::SpeechStarted
            } else {
                VadDecision::Silence
            }
        } else if db >= self.stop_threshold_db {
            self.silence_frames = 0;
            VadDecision::Speech
        } else {
            self.silence_frames = self.silence_frames.saturating_add(1);
            if self.silence_frames > self.hangover_frames {
                self.speaking = false;
                self.silence_frames = 0;
                VadDecision::SpeechEnded
            } else {
                VadDecision::Speech
            }
        }
    }
}

fn frame_db(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return f32::NEG_INFINITY;
    }
    let rms = (frame.iter().map(|sample| sample * sample).sum::<f32>() / frame.len() as f32).sqrt();
    20.0 * rms.max(f32::EPSILON).log10()
}

#[must_use]
pub fn resample_linear(input: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if input.is_empty() || source_rate == target_rate || source_rate == 0 || target_rate == 0 {
        return input.to_vec();
    }
    let filtered;
    let input = if target_rate < source_rate {
        filtered = low_pass(input, source_rate, target_rate);
        filtered.as_slice()
    } else {
        input
    };
    let output_len = input.len() * target_rate as usize / source_rate as usize;
    (0..output_len)
        .map(|index| {
            let position = index as f64 * f64::from(source_rate) / f64::from(target_rate);
            let lower = position.floor() as usize;
            let upper = (lower + 1).min(input.len() - 1);
            let fraction = (position - lower as f64) as f32;
            input[lower].mul_add(1.0 - fraction, input[upper] * fraction)
        })
        .collect()
}

fn low_pass(input: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    let taps = ((source_rate / target_rate).max(1) * 4) as usize;
    let mut output = Vec::with_capacity(input.len());
    let mut sum = 0.0_f32;
    for (index, sample) in input.iter().copied().enumerate() {
        sum += sample;
        if index >= taps {
            sum -= input[index - taps];
        }
        output.push(sum / (index.saturating_add(1).min(taps)) as f32);
    }
    output
}

#[must_use]
pub fn trim_with_vad(audio: &AudioBuffer, threshold_db: f32) -> AudioBuffer {
    trim_with_vad_hangover(audio, threshold_db, 0)
}

#[must_use]
pub fn trim_with_vad_hangover(
    audio: &AudioBuffer,
    threshold_db: f32,
    silence_ms: u64,
) -> AudioBuffer {
    let frame_size = (audio.sample_rate as usize / 50).max(1);
    let voiced = audio
        .samples
        .chunks(frame_size)
        .enumerate()
        .filter_map(|(index, frame)| {
            let db = frame_db(frame);
            (db >= threshold_db).then_some(index)
        })
        .collect::<Vec<_>>();
    let Some(first) = voiced.first() else {
        return AudioBuffer {
            samples: Vec::new(),
            sample_rate: audio.sample_rate,
        };
    };
    let last = *voiced.last().unwrap_or(first);
    let hangover_frames = usize::try_from(silence_ms / 20).unwrap_or(usize::MAX);
    let pre_roll_frames = hangover_frames.min(10);
    let start = first.saturating_sub(pre_roll_frames) * frame_size;
    let end = ((last.saturating_add(hangover_frames) + 1) * frame_size).min(audio.samples.len());
    AudioBuffer {
        samples: audio.samples[start..end].to_vec(),
        sample_rate: audio.sample_rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resamples_to_expected_length() {
        let output = resample_linear(&vec![0.0; 48_000], 48_000, 16_000);
        assert_eq!(output.len(), 16_000);
    }

    #[test]
    fn vad_removes_silent_edges() {
        let mut samples = vec![0.0; 320];
        samples.extend(vec![0.5; 320]);
        samples.extend(vec![0.0; 320]);
        let output = trim_with_vad(
            &AudioBuffer {
                samples,
                sample_rate: 16_000,
            },
            -40.0,
        );
        assert_eq!(output.samples.len(), 320);
    }

    #[test]
    fn vad_keeps_configured_hangover() {
        let mut samples = vec![0.0; 320];
        samples.extend(vec![0.5; 320]);
        samples.extend(vec![0.0; 1_600]);
        let output = trim_with_vad_hangover(
            &AudioBuffer {
                samples,
                sample_rate: 16_000,
            },
            -40.0,
            40,
        );
        assert_eq!(output.samples.len(), 1_280);
    }

    #[test]
    fn downsampling_reduces_nyquist_alias_energy() {
        let alternating = (0..4_800)
            .map(|index| if index % 2 == 0 { 1.0 } else { -1.0 })
            .collect::<Vec<_>>();
        let output = resample_linear(&alternating, 48_000, 16_000);
        let peak = output
            .iter()
            .skip(100)
            .copied()
            .map(f32::abs)
            .fold(0.0, f32::max);
        assert!(peak < 0.3);
    }

    #[test]
    fn vad_state_machine_uses_hysteresis_and_hangover() {
        let mut vad = VadStateMachine::new(-40.0, 40, 20);
        assert_eq!(vad.process(&[0.5; 320]), VadDecision::SpeechStarted);
        assert_eq!(vad.process(&[0.0; 320]), VadDecision::Speech);
        assert_eq!(vad.process(&[0.0; 320]), VadDecision::Speech);
        assert_eq!(vad.process(&[0.0; 320]), VadDecision::SpeechEnded);
    }
}
