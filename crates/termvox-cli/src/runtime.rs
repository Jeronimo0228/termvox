#[cfg(feature = "embedded-whisper")]
use std::io::IsTerminal;
use std::{
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use anyhow::{Result, bail};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use termvox_agents::CliAgent;
use termvox_audio::{AudioRecorder, trim_with_vad_hangover};
use termvox_core::{
    AgentAdapter, AgentKind, AgentRequest, AgentSession, AppConfig, PromptPipeline, SpeechEngine,
    SpeechEngineKind, TranscriptionOptions, assess_prompt,
};
use termvox_hotkeys::{HotkeyRegistration, TriggerState};
use termvox_plugin_sdk::PluginAgentAdapter;
#[cfg(feature = "embedded-whisper")]
use termvox_speech::{DownloadProgress, ModelManager, ModelManifest};
use termvox_speech::{EmbeddedWhisperEngine, OpenAiSpeechEngine, SidecarSpeechEngine};
use tokio_util::sync::CancellationToken;

use crate::{
    cli::RecordAction,
    ui::{RawMode, confirm, parse_key, print_agent_event},
};

pub(crate) async fn test_audio(config: AppConfig, seconds: u64) -> Result<()> {
    let speech = ensure_speech_engine(&config).await?;
    println!("Recording for {seconds} second(s)...");
    let recorder = AudioRecorder::start(&config.audio)?;
    tokio::time::sleep(Duration::from_secs(seconds)).await;
    let audio = trim_with_vad_hangover(
        &recorder.stop().await?,
        config.audio.vad_threshold_db,
        config.audio.vad_silence_ms,
    );
    println!(
        "Captured {:.2}s of voiced audio; transcribing...",
        audio.duration_seconds()
    );
    let transcript = speech
        .transcribe(
            audio,
            &TranscriptionOptions {
                language: Some(config.language),
                initial_prompt: None,
            },
            CancellationToken::new(),
        )
        .await?;
    println!("{}", transcript.text);
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn start(
    config: AppConfig,
    toggle: bool,
    global_hotkey: Option<&str>,
) -> Result<()> {
    let agent = selected_agent(&config)?;
    let info = agent.probe().await;
    if !info.installed {
        bail!(
            "{} is not installed; install it or select another agent",
            info.id
        );
    }
    let speech = ensure_speech_engine(&config).await?;
    let session = agent.start().await?;
    let shutdown = CancellationToken::new();
    let signal_cancel = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            signal_cancel.cancel();
        }
    });
    if let Some(shortcut) = global_hotkey {
        let registration = HotkeyRegistration::register(shortcut)?;
        println!(
            "TermVox ready. {} {shortcut} globally to talk; Ctrl+C quits.",
            if toggle { "Press" } else { "Hold" }
        );
        let mut recorder = None;
        while !shutdown.is_cancelled() {
            if let Some(state) = registration.poll() {
                match state {
                    TriggerState::Pressed if recorder.is_none() => {
                        recorder = Some(AudioRecorder::start(&config.audio)?);
                        println!("Recording...");
                    }
                    TriggerState::Pressed if toggle => {
                        let audio = recorder.take().expect("checked above").stop().await?;
                        process_utterance(
                            &config,
                            audio,
                            speech.as_ref(),
                            agent.as_ref(),
                            &session,
                            shutdown.child_token(),
                        )
                        .await?;
                    }
                    TriggerState::Released if !toggle && recorder.is_some() => {
                        let audio = recorder.take().expect("checked above").stop().await?;
                        process_utterance(
                            &config,
                            audio,
                            speech.as_ref(),
                            agent.as_ref(),
                            &session,
                            shutdown.child_token(),
                        )
                        .await?;
                    }
                    TriggerState::Pressed | TriggerState::Released => {}
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        agent.cancel(&session).await?;
        agent.shutdown().await?;
        return Ok(());
    }
    let _raw_mode = RawMode::enter()?;
    let ptt_key = parse_key(&config.push_to_talk)?;
    println!(
        "TermVox ready. {} {} to talk; press q or Ctrl+C to quit.\r",
        if toggle { "Press" } else { "Hold" },
        config.push_to_talk,
    );
    let mut recorder = None;
    loop {
        if shutdown.is_cancelled() {
            break;
        }
        let event = tokio::task::spawn_blocking(event::read).await??;
        let Event::Key(key) = event else { continue };
        if (key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press)
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            break;
        }
        if key.code == ptt_key && key.kind == KeyEventKind::Press && recorder.is_none() {
            print!("Recording...\r");
            io::stdout().flush()?;
            recorder = Some(AudioRecorder::start(&config.audio)?);
        } else if key.code == ptt_key
            && ((toggle && key.kind == KeyEventKind::Press)
                || (!toggle && key.kind == KeyEventKind::Release))
            && recorder.is_some()
        {
            let audio = recorder.take().expect("checked above").stop().await?;
            disable_raw_mode()?;
            if let Err(error) = process_utterance(
                &config,
                audio,
                speech.as_ref(),
                agent.as_ref(),
                &session,
                shutdown.child_token(),
            )
            .await
            {
                eprintln!("TermVox: {error}");
            }
            enable_raw_mode()?;
            println!("Hold {} to talk; q quits.\r", config.push_to_talk);
        }
    }
    shutdown.cancel();
    agent.cancel(&session).await?;
    agent.shutdown().await?;
    Ok(())
}

async fn process_utterance(
    config: &AppConfig,
    audio: termvox_core::AudioBuffer,
    speech: &dyn SpeechEngine,
    agent: &dyn AgentAdapter,
    session: &AgentSession,
    cancel: CancellationToken,
) -> Result<()> {
    let audio = trim_with_vad_hangover(
        &audio,
        config.audio.vad_threshold_db,
        config.audio.vad_silence_ms,
    );
    if audio.samples.is_empty() {
        println!("No speech detected.");
        return Ok(());
    }
    let transcript = speech
        .transcribe(
            audio,
            &TranscriptionOptions {
                language: Some(config.language.clone()),
                initial_prompt: None,
            },
            cancel.child_token(),
        )
        .await?;
    let prompt = PromptPipeline::from_config(&config.pipeline).process(&transcript.text);
    println!("\nHeard:  {}", transcript.text);
    println!("Prompt: {prompt}");
    let risk = assess_prompt(&prompt);
    if risk.requires_confirmation {
        println!("Risk signals: {}", risk.matches.join(", "));
    }
    let must_confirm = config.confirmation || risk.requires_confirmation || !config.auto_send;
    if must_confirm && !confirm("Send to agent? [y/N] ")? {
        println!("Cancelled.");
        return Ok(());
    }
    let mut events = agent
        .send_prompt(
            session,
            AgentRequest {
                prompt,
                cwd: std::env::current_dir()?,
                limits: config.runtime.clone(),
                permission_profile: config.permission_profile,
            },
            cancel,
        )
        .await?;
    while let Some(event) = events.recv().await {
        print_agent_event(&event?);
    }
    Ok(())
}

pub(crate) async fn record(config: AppConfig, action: RecordAction) -> Result<()> {
    let marker = record_marker();
    match action {
        RecordAction::Stop => {
            if marker.exists() {
                std::fs::remove_file(&marker)?;
                println!("Stop requested");
            } else {
                println!("No external recording is active");
            }
        }
        RecordAction::Toggle if marker.exists() => {
            std::fs::remove_file(&marker)?;
            println!("Stop requested");
        }
        RecordAction::Start | RecordAction::Toggle => {
            let speech = ensure_speech_engine(&config).await?;
            if let Some(parent) = marker.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&marker, std::process::id().to_string())?;
            println!("External recording started; run `termvox record stop` to finish.");
            let recorder = AudioRecorder::start(&config.audio)?;
            while marker.exists() {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            let audio = recorder.stop().await?;
            let agent = selected_agent(&config)?;
            let session = agent.start().await?;
            process_utterance(
                &config,
                audio,
                speech.as_ref(),
                agent.as_ref(),
                &session,
                CancellationToken::new(),
            )
            .await?;
            agent.shutdown().await?;
        }
    }
    Ok(())
}

async fn ensure_speech_engine(config: &AppConfig) -> Result<Arc<dyn SpeechEngine>> {
    let engine = speech_engine(config);
    if let Err(error) = engine.healthcheck().await {
        if config.speech_engine != SpeechEngineKind::WhisperCpp {
            return Err(error.into());
        }
        #[cfg(not(feature = "embedded-whisper"))]
        return Err(error.into());
        #[cfg(feature = "embedded-whisper")]
        {
            if config.whisper.model.is_file() {
                return Err(error.into());
            }
            if !io::stdin().is_terminal() {
                bail!(
                    "{error}; install the free local model with `termvox models install default`"
                );
            }
            let manifest = ModelManifest::bundled()?;
            let artifact = manifest
                .find("whisper-base", std::env::consts::OS)
                .ok_or_else(|| anyhow::anyhow!("reviewed default Whisper model is unavailable"))?;
            if !confirm(&format!(
                "Download the free local Whisper model ({} MiB) now? [y/N] ",
                artifact.size_bytes / (1024 * 1024)
            ))? {
                bail!("model download declined; run `termvox models install default` later");
            }
            let mut last_percent = 0_u64;
            ModelManager::default()
                .download_verified_with(
                    &artifact.url,
                    &config.whisper.model,
                    &artifact.sha256,
                    CancellationToken::new(),
                    move |progress| print_model_progress(progress, &mut last_percent),
                )
                .await?;
            println!(
                "Verified local model installed at {}",
                config.whisper.model.display()
            );
            engine.healthcheck().await?;
        }
    }
    Ok(engine)
}

#[cfg(feature = "embedded-whisper")]
fn print_model_progress(progress: DownloadProgress, last_percent: &mut u64) {
    let Some(total) = progress.total_bytes.filter(|total| *total > 0) else {
        return;
    };
    let percent = progress.downloaded_bytes.saturating_mul(100) / total;
    if percent == 100 || percent >= last_percent.saturating_add(5) {
        println!(
            "{percent:>3}% ({}/{total} bytes)",
            progress.downloaded_bytes
        );
        *last_percent = percent;
    }
}

pub(crate) fn speech_engine(config: &AppConfig) -> Arc<dyn SpeechEngine> {
    match config.speech_engine {
        SpeechEngineKind::WhisperCpp => {
            Arc::new(EmbeddedWhisperEngine::new(config.whisper.clone()))
        }
        SpeechEngineKind::OpenAi => Arc::new(OpenAiSpeechEngine::new(config.openai.clone())),
        SpeechEngineKind::Parakeet => {
            Arc::new(SidecarSpeechEngine::parakeet(config.parakeet.clone()))
        }
        SpeechEngineKind::Vosk => Arc::new(SidecarSpeechEngine::vosk(config.vosk.clone())),
    }
}

fn selected_agent(config: &AppConfig) -> Result<Arc<dyn AgentAdapter>> {
    if let Some(id) = &config.agent_plugin {
        let plugin = config
            .plugins
            .iter()
            .find(|plugin| plugin.enabled && &plugin.id == id)
            .ok_or_else(|| anyhow::anyhow!("enabled agent plugin not found: {id}"))?;
        return Ok(Arc::new(PluginAgentAdapter::new(plugin.clone())));
    }
    Ok(match config.agent {
        AgentKind::Codex => Arc::new(CliAgent::codex()),
        AgentKind::Claude => Arc::new(CliAgent::claude()),
        AgentKind::Cursor => Arc::new(CliAgent::cursor()),
        AgentKind::Gemini => Arc::new(CliAgent::gemini()),
        AgentKind::Aider => Arc::new(CliAgent::aider()),
        AgentKind::Amp => Arc::new(CliAgent::amp()),
    })
}

pub(crate) fn all_agents() -> [CliAgent; 6] {
    [
        CliAgent::codex(),
        CliAgent::claude(),
        CliAgent::cursor(),
        CliAgent::gemini(),
        CliAgent::aider(),
        CliAgent::amp(),
    ]
}

fn record_marker() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("termvox-recording")
}
