use std::path::PathBuf;

use crate::{AppConfig, PerformanceProfile};

#[must_use]
pub fn default_whisper_model(profile: PerformanceProfile) -> PathBuf {
    let filename = match profile {
        PerformanceProfile::Fast | PerformanceProfile::Balanced => "ggml-tiny.bin",
        PerformanceProfile::Accurate | PerformanceProfile::Custom => "ggml-base.bin",
    };
    data_path(&format!("termvox/models/{filename}"))
}

pub fn apply_performance_profile(config: &mut AppConfig) {
    if config.performance_profile == PerformanceProfile::Custom {
        return;
    }

    let defaults = AppConfig::default();
    let profile = config.performance_profile;

    if config.whisper.model == defaults.whisper.model {
        config.whisper.model = default_whisper_model(profile);
    }

    match profile {
        PerformanceProfile::Fast => {
            if config.audio.max_seconds == defaults.audio.max_seconds {
                config.audio.max_seconds = 30;
            }
            if config.audio.vad_silence_ms == defaults.audio.vad_silence_ms {
                config.audio.vad_silence_ms = 400;
            }
            config.audio.auto_stop_on_silence = true;
            config.whisper.prewarm_on_start = false;
            config.whisper.optimize_for_latency = true;
            if config.whisper.max_threads == defaults.whisper.max_threads {
                config.whisper.max_threads = 4;
            }
        }
        PerformanceProfile::Balanced => {
            if config.audio.max_seconds == defaults.audio.max_seconds {
                config.audio.max_seconds = 60;
            }
            if config.audio.vad_silence_ms == defaults.audio.vad_silence_ms {
                config.audio.vad_silence_ms = 600;
            }
            config.audio.auto_stop_on_silence = true;
            config.whisper.prewarm_on_start = true;
            config.whisper.optimize_for_latency = true;
            if config.whisper.max_threads == defaults.whisper.max_threads {
                config.whisper.max_threads = 6;
            }
        }
        PerformanceProfile::Accurate => {
            if config.audio.max_seconds == defaults.audio.max_seconds {
                config.audio.max_seconds = 120;
            }
            config.audio.auto_stop_on_silence = false;
            config.whisper.prewarm_on_start = true;
            config.whisper.optimize_for_latency = false;
            if config.whisper.max_threads == defaults.whisper.max_threads {
                config.whisper.max_threads = 0;
            }
        }
        PerformanceProfile::Custom => {}
    }
}

fn data_path(relative: &str) -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_profile_prefers_tiny_model() {
        let mut config = AppConfig::default();
        config.performance_profile = PerformanceProfile::Fast;
        apply_performance_profile(&mut config);
        assert!(config.whisper.model.ends_with("ggml-tiny.bin"));
        assert_eq!(config.audio.max_seconds, 30);
        assert!(config.audio.auto_stop_on_silence);
        assert!(!config.whisper.prewarm_on_start);
    }

    #[test]
    fn custom_profile_leaves_user_overrides() {
        let mut config = AppConfig::default();
        config.performance_profile = PerformanceProfile::Custom;
        config.audio.max_seconds = 17;
        apply_performance_profile(&mut config);
        assert_eq!(config.audio.max_seconds, 17);
    }
}
