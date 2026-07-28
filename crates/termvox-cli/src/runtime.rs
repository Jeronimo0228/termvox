#[cfg(feature = "embedded-whisper")]
use std::io::IsTerminal;
use std::{io, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Result, bail};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use termvox_agents::{CliAgent, SupportedAgent};
use termvox_audio::{AudioRecorder, trim_with_vad_hangover};
use termvox_core::{
    AgentAdapter, AgentDisplayMode, AgentInfo, AgentKind, AgentRequest, AgentSession, AppConfig,
    PromptPipeline, SpeechEngine, SpeechEngineKind, TranscriptionOptions, assess_prompt,
    detect_environment, whisper_initial_prompt,
};
use termvox_hotkeys::{HotkeyRegistration, TriggerState};
use termvox_plugin_sdk::PluginAgentAdapter;
#[cfg(feature = "embedded-whisper")]
use termvox_speech::{DownloadProgress, ModelManager, ModelManifest};
use termvox_speech::{EmbeddedWhisperEngine, OpenAiSpeechEngine, SidecarSpeechEngine};
use tokio_util::sync::CancellationToken;

use crate::{
    cli::RecordAction,
    delivery,
    session_ui::SessionUi,
    telemetry,
    ui::{RawMode, confirm, parse_key, print_agent_event},
    workspace,
};

pub fn resolve_toggle(config: &AppConfig, toggle_flag: bool) -> bool {
    if toggle_flag {
        return true;
    }
    let env = detect_environment();
    config.runtime.auto_toggle_on_wayland && (env.wayland || env.windows)
}

pub(crate) async fn test_audio(config: AppConfig, seconds: u64) -> Result<()> {
    let speech = ensure_speech_engine(&config).await?;
    if config.whisper.prewarm_on_start {
        speech.prewarm().await?;
    }
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
            &transcription_options(&config),
            CancellationToken::new(),
        )
        .await?;
    println!("{} ({} ms)", transcript.text, transcript.duration_ms);
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn start(
    config: AppConfig,
    toggle: bool,
    global_hotkey: Option<&str>,
) -> Result<()> {
    let toggle = resolve_toggle(&config, toggle);
    let agent = selected_agent(&config)?;
    let info = agent.probe().await;
    if !info.installed && !is_companion_mode(&config) {
        bail!(
            "{} is not installed; install it or select another agent",
            info.id
        );
    }
    if !is_companion_mode(&config) {
        ensure_agent_authenticated(&info)?;
    }
    let profile = config.agents.profile(config.agent);
    if profile.resolved_display(config.agent) == AgentDisplayMode::Shell {
        let kind = config.agent;
        return crate::shell::run(config, kind, Vec::new(), false).await;
    }
    let speech = ensure_speech_engine(&config).await?;
    schedule_prewarm(Arc::clone(&speech), &config.whisper);
    let session = agent.start().await?;
    workspace::hydrate_agent_session(&config, config.agent, &session, false).await;
    let ui = session_ui(&config, toggle);
    let shutdown = CancellationToken::new();
    let signal_cancel = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            signal_cancel.cancel();
        }
    });
    if let Some(shortcut) = global_hotkey {
        let registration = HotkeyRegistration::register(shortcut)?;
        ui.show_global_ready(shortcut);
        let mut recorder: Option<AudioRecorder> = None;
        while !shutdown.is_cancelled() {
            if let Some(recorder_ref) = &recorder {
                if config.audio.auto_stop_on_silence && recorder_ref.auto_stop_triggered() {
                    let audio = recorder.take().expect("checked above").stop().await?;
                    process_utterance(
                        &config,
                        &ui,
                        audio,
                        speech.as_ref(),
                        agent.as_ref(),
                        &session,
                        shutdown.child_token(),
                        false,
                    )
                    .await?;
                    continue;
                }
            }
            if let Some(state) = registration.poll() {
                match state {
                    TriggerState::Pressed if recorder.is_none() => {
                        recorder = Some(AudioRecorder::start(&config.audio)?);
                        ui.show_recording();
                    }
                    TriggerState::Pressed if toggle => {
                        let audio = recorder.take().expect("checked above").stop().await?;
                        process_utterance(
                            &config,
                            &ui,
                            audio,
                            speech.as_ref(),
                            agent.as_ref(),
                            &session,
                            shutdown.child_token(),
                            false,
                        )
                        .await?;
                    }
                    TriggerState::Released if !toggle && recorder.is_some() => {
                        let audio = recorder.take().expect("checked above").stop().await?;
                        process_utterance(
                            &config,
                            &ui,
                            audio,
                            speech.as_ref(),
                            agent.as_ref(),
                            &session,
                            shutdown.child_token(),
                            false,
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
        workspace::save_agent_session(&config, config.agent, &session).await;
        return Ok(());
    }
    let _raw_mode = RawMode::enter()?;
    let ptt_key = parse_key(&config.push_to_talk)?;
    ui.show_startup(info.version.as_deref());
    let mut recorder: Option<AudioRecorder> = None;
    loop {
        if shutdown.is_cancelled() {
            break;
        }
        if let Some(recorder_ref) = &recorder {
            if config.audio.auto_stop_on_silence && recorder_ref.auto_stop_triggered() {
                let audio = recorder.take().expect("checked above").stop().await?;
                if let Err(error) = process_utterance(
                    &config,
                    &ui,
                    audio,
                    speech.as_ref(),
                    agent.as_ref(),
                    &session,
                    shutdown.child_token(),
                    true,
                )
                .await
                {
                    ui.show_error(&error.to_string());
                }
                ui.show_idle();
                continue;
            }
        }
        let ready =
            tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(50))).await??;
        if !ready {
            continue;
        }
        let event = tokio::task::spawn_blocking(event::read).await??;
        let Event::Key(key) = event else { continue };
        if (key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press)
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            break;
        }
        if key.code == ptt_key && key.kind == KeyEventKind::Press && recorder.is_none() {
            ui.show_recording();
            recorder = Some(AudioRecorder::start(&config.audio)?);
        } else if key.code == ptt_key
            && ((toggle && key.kind == KeyEventKind::Press)
                || (!toggle && key.kind == KeyEventKind::Release))
            && recorder.is_some()
        {
            let audio = recorder.take().expect("checked above").stop().await?;
            if let Err(error) = process_utterance(
                &config,
                &ui,
                audio,
                speech.as_ref(),
                agent.as_ref(),
                &session,
                shutdown.child_token(),
                true,
            )
            .await
            {
                ui.show_error(&error.to_string());
            }
            ui.show_idle();
        }
    }
    shutdown.cancel();
    agent.cancel(&session).await?;
    agent.shutdown().await?;
    workspace::save_agent_session(&config, config.agent, &session).await;
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn start_daemon_session(
    config: AppConfig,
    hotkey: &str,
    ipc_toggle: Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    use crate::daemon;

    let agent = selected_agent(&config)?;
    let speech = ensure_speech_engine(&config).await?;
    schedule_prewarm(Arc::clone(&speech), &config.whisper);
    let session = agent.start().await?;
    workspace::hydrate_agent_session(&config, config.agent, &session, false).await;
    let ui = session_ui(&config, true);
    let registration = HotkeyRegistration::register(hotkey)?;
    ui.show_global_ready(hotkey);
    let shutdown = CancellationToken::new();
    let signal_cancel = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            signal_cancel.cancel();
        }
    });
    let mut recorder: Option<AudioRecorder> = None;
    while !shutdown.is_cancelled() {
        if daemon::take_toggle(&ipc_toggle) {
            if recorder.is_none() {
                recorder = Some(AudioRecorder::start(&config.audio)?);
                ui.show_recording();
            } else {
                let audio = recorder.take().expect("checked above").stop().await?;
                process_utterance(
                    &config,
                    &ui,
                    audio,
                    speech.as_ref(),
                    agent.as_ref(),
                    &session,
                    shutdown.child_token(),
                    false,
                )
                .await?;
            }
        }
        if let Some(recorder_ref) = &recorder {
            if config.audio.auto_stop_on_silence && recorder_ref.auto_stop_triggered() {
                let audio = recorder.take().expect("checked above").stop().await?;
                process_utterance(
                    &config,
                    &ui,
                    audio,
                    speech.as_ref(),
                    agent.as_ref(),
                    &session,
                    shutdown.child_token(),
                    false,
                )
                .await?;
                continue;
            }
        }
        if let Some(state) = registration.poll() {
            if matches!(state, TriggerState::Pressed) {
                if recorder.is_none() {
                    recorder = Some(AudioRecorder::start(&config.audio)?);
                    ui.show_recording();
                } else {
                    let audio = recorder.take().expect("checked above").stop().await?;
                    process_utterance(
                        &config,
                        &ui,
                        audio,
                        speech.as_ref(),
                        agent.as_ref(),
                        &session,
                        shutdown.child_token(),
                        false,
                    )
                    .await?;
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    agent.cancel(&session).await?;
    agent.shutdown().await?;
    workspace::save_agent_session(&config, config.agent, &session).await;
    Ok(())
}

async fn process_utterance(
    config: &AppConfig,
    ui: &SessionUi,
    audio: termvox_core::AudioBuffer,
    speech: &dyn SpeechEngine,
    agent: &dyn AgentAdapter,
    session: &AgentSession,
    cancel: CancellationToken,
    interactive_terminal: bool,
) -> Result<()> {
    let audio = trim_with_vad_hangover(
        &audio,
        config.audio.vad_threshold_db,
        config.audio.vad_silence_ms,
    );
    let audio_seconds = audio.duration_seconds();
    if audio.samples.is_empty() {
        ui.show_no_speech();
        return Ok(());
    }
    ui.show_transcribing();
    let mut options = transcription_options(config);
    if config.whisper.streaming {
        let ui_partial = ui.clone();
        options.on_partial = Some(Arc::new(move |partial| {
            ui_partial.show_partial_transcript(&partial);
        }));
    }
    let transcript = speech
        .transcribe(audio, &options, cancel.child_token())
        .await?;
    let prompt = PromptPipeline::from_config(&config.pipeline).process(&transcript.text);
    let risk = assess_prompt(&prompt);
    ui.show_transcript(
        &transcript.text,
        &prompt,
        transcript.duration_ms,
        &risk.matches,
    );
    let must_confirm = config.confirmation || risk.requires_confirmation || !config.auto_send;
    if must_confirm && interactive_terminal {
        disable_raw_mode()?;
        let approved = confirm(&ui.show_confirm_prompt())?;
        enable_raw_mode()?;
        if !approved {
            ui.show_cancelled();
            return Ok(());
        }
    } else if must_confirm {
        ui.show_cancelled();
        return Ok(());
    }
    if ui.mode() == AgentDisplayMode::Companion {
        let profile = config.agents.profile(config.agent);
        let delivery_mode = profile.resolved_delivery(config.agent);
        let paste_window = profile.resolved_paste_window_title(config.agent);
        match delivery::deliver_prompt(&prompt, delivery_mode, paste_window) {
            Ok(outcome) => ui.show_delivery(outcome, paste_window),
            Err(error) => ui.show_delivery_failed(&error.to_string()),
        }
        let _ = telemetry::record_utterance(
            config,
            transcript.duration_ms,
            audio_seconds,
            Some(delivery_mode.as_str()),
        );
        return Ok(());
    }
    let theme = match ui.mode() {
        AgentDisplayMode::Branded => Some(ui.theme()),
        _ => None,
    };
    let mut events = agent
        .send_prompt(
            session,
            AgentRequest {
                prompt,
                cwd: std::env::current_dir()?,
                limits: config.runtime.clone(),
                permission_profile: config.permission_profile,
                invocation: config.agents.profile(config.agent).invocation(),
            },
            cancel,
        )
        .await?;
    while let Some(event) = events.recv().await {
        print_agent_event(&event?, theme);
    }
    workspace::save_agent_session(config, config.agent, session).await;
    Ok(())
}

pub(crate) fn transcription_options(config: &AppConfig) -> TranscriptionOptions {
    TranscriptionOptions {
        language: Some(config.language.clone()),
        initial_prompt: whisper_initial_prompt(&config.pipeline),
        on_partial: None,
    }
}

fn session_ui(config: &AppConfig, toggle: bool) -> SessionUi {
    let profile = config.agents.profile(config.agent);
    SessionUi::new(
        config.agent,
        profile.resolved_display(config.agent),
        &config.push_to_talk,
        toggle,
    )
}

fn is_companion_mode(config: &AppConfig) -> bool {
    config
        .agents
        .profile(config.agent)
        .resolved_display(config.agent)
        == AgentDisplayMode::Companion
}

/// Fail fast when the selected agent requires upstream auth that is not configured.
pub(crate) fn ensure_agent_authenticated(info: &AgentInfo) -> Result<()> {
    if let Some(auth) = &info.auth {
        if !auth.ok {
            let login = auth
                .login_command
                .as_deref()
                .unwrap_or("see upstream agent auth docs");
            bail!(
                "{} is not authenticated ({}). Run `{login}` first.",
                info.id,
                auth.detail
            );
        }
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
            schedule_prewarm(Arc::clone(&speech), &config.whisper);
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
            let ui = session_ui(&config, true);
            process_utterance(
                &config,
                &ui,
                audio,
                speech.as_ref(),
                agent.as_ref(),
                &session,
                CancellationToken::new(),
                false,
            )
            .await?;
            agent.shutdown().await?;
        }
    }
    Ok(())
}

pub(crate) async fn ensure_speech_engine(config: &AppConfig) -> Result<Arc<dyn SpeechEngine>> {
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
                .find("whisper-tiny", std::env::consts::OS)
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
    Ok(configured_cli_agent(config.agent, config))
}

pub(crate) fn configured_cli_agent_kind(kind: AgentKind, config: &AppConfig) -> CliAgent {
    let supported = match kind {
        AgentKind::Codex => SupportedAgent::Codex,
        AgentKind::Claude => SupportedAgent::Claude,
        AgentKind::Cursor => SupportedAgent::Cursor,
        AgentKind::Gemini => SupportedAgent::Gemini,
        AgentKind::Aider => SupportedAgent::Aider,
        AgentKind::Amp => SupportedAgent::Amp,
        AgentKind::OpenCode => SupportedAgent::OpenCode,
    };
    CliAgent::with_executable(supported, config.agents.resolve_executable(kind))
}

pub(crate) fn configured_cli_agent(kind: AgentKind, config: &AppConfig) -> Arc<dyn AgentAdapter> {
    Arc::new(configured_cli_agent_kind(kind, config))
}

pub(crate) fn all_agents() -> [CliAgent; 7] {
    [
        CliAgent::codex(),
        CliAgent::claude(),
        CliAgent::cursor(),
        CliAgent::gemini(),
        CliAgent::aider(),
        CliAgent::amp(),
        CliAgent::opencode(),
    ]
}

pub(crate) fn schedule_prewarm(speech: Arc<dyn SpeechEngine>, whisper: &termvox_core::WhisperConfig) {
    if whisper.prewarm_on_start {
        tokio::spawn(async move {
            let _ = speech.prewarm().await;
        });
        return;
    }
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let _ = speech.prewarm().await;
    });
}

fn record_marker() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("termvox-recording")
}

pub(crate) struct PreparedUtterance {
    pub transcript: String,
    pub prompt: String,
    pub duration_ms: u64,
    pub risk_matches: Vec<String>,
    pub requires_confirmation: bool,
}

pub(crate) async fn prepare_utterance(
    config: &AppConfig,
    audio: termvox_core::AudioBuffer,
    speech: &dyn SpeechEngine,
    on_partial: Option<Arc<dyn Fn(String) + Send + Sync>>,
    cancel: CancellationToken,
) -> Result<Option<PreparedUtterance>> {
    let audio = trim_with_vad_hangover(
        &audio,
        config.audio.vad_threshold_db,
        config.audio.vad_silence_ms,
    );
    if audio.samples.is_empty() {
        return Ok(None);
    }
    let mut options = transcription_options(config);
    options.on_partial = on_partial;
    let transcript = speech
        .transcribe(audio, &options, cancel.child_token())
        .await?;
    let prompt = PromptPipeline::from_config(&config.pipeline).process(&transcript.text);
    let risk = assess_prompt(&prompt);
    Ok(Some(PreparedUtterance {
        transcript: transcript.text,
        prompt,
        duration_ms: transcript.duration_ms,
        risk_matches: risk.matches,
        requires_confirmation: config.confirmation || risk.requires_confirmation || !config.auto_send,
    }))
}
