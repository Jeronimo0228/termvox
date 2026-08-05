//! Integrated agent shell: upstream CLI TUI + `TermVox` mic bar.

mod bar;
mod keys;
mod messages;
mod pty_filter;

use std::{
    io::{self, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{self, ClearType},
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use termvox_agents::{
    InteractiveLaunch, SupportedAgent, extract_session_id, scan_output_for_session_id,
};
use termvox_audio::AudioRecorder;
use termvox_core::{AgentAdapter, AgentKind, AppConfig, SpeechEngine, agent_ui};

use crate::workspace;
use tokio_util::sync::CancellationToken;

use crate::runtime::{
    configured_cli_agent_kind, ensure_agent_authenticated, ensure_speech_engine,
    prepare_utterance,
};

use self::bar::{BarState, ShellBar};

const BAR_REFRESH: Duration = Duration::from_millis(80);
const IDLE_BAR_REFRESH: Duration = Duration::from_millis(350);

/// Run the integrated shell for the selected or overridden agent.
pub async fn run(
    config: AppConfig,
    agent: AgentKind,
    trailing_args: Vec<String>,
    fresh: bool,
) -> Result<()> {
    let cli_agent = configured_cli_agent_kind(agent, &config);
    let info = cli_agent.probe().await;
    if !info.installed {
        bail!(
            "{} is not installed; install it or pick another agent",
            info.id
        );
    }
    ensure_agent_authenticated(&info)?;
    let speech = ensure_speech_engine(&config).await?;
    let cwd = std::env::current_dir()?;
    let profile = config.agents.profile(agent);
    let allow_discover = config.workspace.discover_session && !fresh;
    let remote_id = if fresh {
        None
    } else {
        workspace::load_remote_id(&config, agent, &cwd)
    };
    let launch = cli_agent.interactive_launch(
        &cwd,
        &trailing_args,
        &profile.shell_invocation(),
        remote_id.as_deref(),
    );
    let config = Arc::new(config);
    let supported = to_supported(agent);
    tokio::task::spawn_blocking(move || {
        run_session(
            &config,
            agent,
            supported,
            &launch,
            &speech,
            &cwd,
            remote_id,
            allow_discover,
        )
    })
    .await
    .context("shell session task failed")?
}

#[allow(clippy::too_many_lines)]
fn run_session(
    config: &AppConfig,
    agent: AgentKind,
    supported: SupportedAgent,
    launch: &InteractiveLaunch,
    speech: &Arc<dyn SpeechEngine>,
    cwd: &std::path::Path,
    resumed_remote_id: Option<String>,
    allow_discover: bool,
) -> Result<()> {
    let rt = tokio::runtime::Handle::current();
    terminal::enable_raw_mode().context("enable raw mode")?;
    let mut guard = TerminalGuard::new();
    pty_filter::reclaim_host_keyboard(&mut io::stdout()).context("reclaim host keyboard")?;
    let voice_hotkeys = keys::shell_voice_hotkeys(config);
    let (cols, rows) = terminal_size()?;
    let agent_rows = rows.saturating_sub(1).max(1);
    setup_viewport(agent_rows, rows)?;
    let mut bar = ShellBar::new(
        agent_ui(agent),
        voice_hotkeys.clone(),
        config.shell.exit_hotkey.clone(),
        config.language.clone(),
        rows,
        cols,
    );
    bar.set_session_hint(resumed_remote_id.clone());
    if let Some(id) = &resumed_remote_id {
        bar.set_state(BarState::Notice(messages::resume_notice(
            &config.language,
            id,
        )));
        bar.draw()?;
        std::thread::sleep(Duration::from_millis(900));
        bar.set_state(BarState::Ready);
    }
    bar.draw()?;

    let mut current_remote_id = resumed_remote_id;
    let captured_remote = Arc::new(Mutex::new(current_remote_id.clone()));

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: agent_rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("open PTY")?;

    let mut command = CommandBuilder::new(&launch.executable);
    command.cwd(&launch.cwd);
    for arg in &launch.args {
        command.arg(arg);
    }
    let child = pair
        .slave
        .spawn_command(command)
        .with_context(|| format!("spawn {}", launch.executable.display()))?;
    drop(pair.slave);

    let mut writer = pair.master.take_writer().context("pty writer")?;
    let mut reader = pair.master.try_clone_reader().context("pty reader")?;
    let stop_reader = Arc::new(AtomicBool::new(false));
    let stop_copy = Arc::clone(&stop_reader);
    let (pty_activity_tx, pty_activity_rx) = mpsc::channel();
    let capture = Arc::clone(&captured_remote);
    let copy_agent = thread::spawn(move || {
        copy_pty_output(
            &mut reader,
            &stop_copy,
            &pty_activity_tx,
            supported,
            &capture,
        )
    });

    let child = Arc::new(Mutex::new(child));
    let (exit_tx, exit_rx) = mpsc::channel();
    let child_wait = Arc::clone(&child);
    thread::spawn(move || {
        let status = child_wait.lock().expect("child mutex").wait();
        let _ = exit_tx.send(status);
    });

    let global_hotkey = register_shell_hotkey(&voice_hotkeys);

    let mut recorder: Option<AudioRecorder> = None;
    let mut awaiting_confirm: Option<String> = None;
    let mut anim_frame = 0_u8;
    let mut last_bar_draw = Instant::now();
    let mut last_persist = Instant::now();
    let mut running = true;
    let mut speech_prewarmed = false;

    if allow_discover {
        let discover_agent = supported;
        let discover_cwd = cwd.to_path_buf();
        let discover_target = Arc::clone(&captured_remote);
        let discover_stop = Arc::clone(&stop_reader);
        thread::spawn(move || {
            while !discover_stop.load(Ordering::Acquire) {
                if discover_target.lock().expect("session mutex").is_some() {
                    break;
                }
                if let Some(id) =
                    termvox_agents::discover_remote_session(discover_agent, &discover_cwd)
                {
                    *discover_target.lock().expect("session mutex") = Some(id);
                    break;
                }
                thread::sleep(Duration::from_secs(5));
            }
        });
    }

    while running {
        if exit_rx.try_recv().is_ok() {
            running = false;
            continue;
        }

        if let Some(remote_id) = captured_remote.lock().expect("session mutex").clone() {
            if current_remote_id.as_deref() != Some(remote_id.as_str()) {
                current_remote_id = Some(remote_id.clone());
                bar.set_session_hint(Some(remote_id));
                last_persist = Instant::now();
            }
        }

        if config.workspace.persist_session && last_persist.elapsed() >= Duration::from_secs(15) {
            workspace::persist_remote_id(config, agent, cwd, current_remote_id.clone());
            last_persist = Instant::now();
        }

        if let Some(recorder_ref) = &recorder {
            if config.audio.auto_stop_on_silence && recorder_ref.auto_stop_triggered() {
                let audio = rt.block_on(recorder.take().expect("recorder").stop())?;
                handle_finished_recording(
                    config,
                    speech,
                    &rt,
                    &mut bar,
                    &mut writer,
                    &mut awaiting_confirm,
                    audio,
                )?;
            }
        }

        if should_redraw_bar(
            recorder.as_ref(),
            &pty_activity_rx,
            last_bar_draw,
            awaiting_confirm.is_some(),
        ) {
            if recorder.is_some() || bar.needs_animation() {
                anim_frame = anim_frame.wrapping_add(1);
                let level = recorder.as_ref().map_or(0.0, AudioRecorder::input_level);
                bar.set_recording_visuals(anim_frame, level);
            }
            bar.draw()?;
            last_bar_draw = Instant::now();
        }

        if let Some(registration) = &global_hotkey {
            if let Some(termvox_hotkeys::TriggerState::Pressed) = registration.poll() {
                toggle_voice(
                    config,
                    speech,
                    &rt,
                    &mut recorder,
                    &mut bar,
                    &mut writer,
                    &mut awaiting_confirm,
                    &mut anim_frame,
                    &mut last_bar_draw,
                    &mut speech_prewarmed,
                )?;
            }
        }

        if !event::poll(Duration::from_millis(25)).context("poll input")? {
            continue;
        }

        match event::read().context("read input")? {
            Event::Key(key) if awaiting_confirm.is_some() => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('y' | 'Y') => {
                        if let Some(prompt) = awaiting_confirm.take() {
                            inject_prompt(&mut writer, config, &mut bar, &prompt)?;
                        }
                    }
                    KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                        awaiting_confirm = None;
                        bar.set_state(BarState::Ready);
                        bar.draw()?;
                    }
                    _ => {}
                }
            }
            Event::Key(key) if keys::is_shell_exit(&key, &config.shell.exit_hotkey) => {
                bar.set_state(BarState::Exiting);
                bar.draw()?;
                running = false;
            }
            Event::Key(key) if keys::is_voice_hotkey(&key, &voice_hotkeys) => {
                if awaiting_confirm.is_some() {
                    continue;
                }
                toggle_voice(
                    config,
                    speech,
                    &rt,
                    &mut recorder,
                    &mut bar,
                    &mut writer,
                    &mut awaiting_confirm,
                    &mut anim_frame,
                    &mut last_bar_draw,
                    &mut speech_prewarmed,
                )?;
            }
            Event::Key(key) => {
                if keys::is_voice_hotkey(&key, &voice_hotkeys)
                    || keys::is_shell_exit(&key, &config.shell.exit_hotkey)
                {
                    continue;
                }
                keys::forward_key(key, &mut writer)?;
            }
            Event::Resize(cols, rows) => {
                let agent_rows = rows.saturating_sub(1).max(1);
                resize_viewport(agent_rows, rows)?;
                bar.set_size(rows, cols);
                bar.draw()?;
                let _ = pair.master.resize(PtySize {
                    rows: agent_rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
            _ => {}
        }
    }

    stop_reader.store(true, Ordering::Release);
    let _ = copy_agent.join();
    let _ = child.lock().expect("child mutex").kill();
    if config.workspace.persist_session {
        workspace::persist_remote_id(config, agent, cwd, current_remote_id.clone());
    }
    bar.set_state(BarState::Exiting);
    bar.draw()?;
    guard.restore()?;
    Ok(())
}

fn should_redraw_bar(
    recorder: Option<&AudioRecorder>,
    pty_activity_rx: &mpsc::Receiver<()>,
    last_draw: Instant,
    confirming: bool,
) -> bool {
    if pty_activity_rx.try_recv().is_ok() {
        return true;
    }
    if recorder.is_some() || confirming {
        return last_draw.elapsed() >= BAR_REFRESH;
    }
    last_draw.elapsed() >= IDLE_BAR_REFRESH
}

fn toggle_voice(
    config: &AppConfig,
    speech: &Arc<dyn SpeechEngine>,
    rt: &tokio::runtime::Handle,
    recorder: &mut Option<AudioRecorder>,
    bar: &mut ShellBar,
    writer: &mut impl Write,
    awaiting_confirm: &mut Option<String>,
    anim_frame: &mut u8,
    last_bar_draw: &mut Instant,
    speech_prewarmed: &mut bool,
) -> Result<()> {
    if let Some(recorder_handle) = recorder.take() {
        let audio = rt.block_on(recorder_handle.stop())?;
        handle_finished_recording(config, speech, rt, bar, writer, awaiting_confirm, audio)?;
    } else {
        if !*speech_prewarmed {
            let _ = rt.block_on(speech.prewarm());
            *speech_prewarmed = true;
        }
        *recorder = Some(AudioRecorder::start(&config.audio)?);
        bar.set_state(BarState::Recording);
        *anim_frame = 0;
        bar.set_recording_visuals(0, 0.0);
        bar.draw()?;
        *last_bar_draw = Instant::now();
    }
    Ok(())
}

fn register_shell_hotkey(voice_hotkeys: &[String]) -> Option<termvox_hotkeys::HotkeyRegistration> {
    for hotkey in voice_hotkeys {
        if let Ok(registration) = termvox_hotkeys::HotkeyRegistration::register(hotkey) {
            return Some(registration);
        }
    }
    None
}

fn handle_finished_recording(
    config: &AppConfig,
    speech: &Arc<dyn SpeechEngine>,
    rt: &tokio::runtime::Handle,
    bar: &mut ShellBar,
    writer: &mut impl Write,
    awaiting_confirm: &mut Option<String>,
    audio: termvox_core::AudioBuffer,
) -> Result<()> {
    bar.set_state(BarState::Transcribing);
    bar.draw()?;

    let (partial_tx, partial_rx) = mpsc::channel::<String>();
    let (done_tx, done_rx) = mpsc::channel();
    let shared_config = Arc::new(config.clone());
    let speech = Arc::clone(speech);
    let rt = rt.clone();
    let worker_config = Arc::clone(&shared_config);
    let worker = thread::spawn(move || {
        let on_partial = Arc::new(move |text: String| {
            let _ = partial_tx.send(text);
        });
        let result = rt.block_on(prepare_utterance(
            &worker_config,
            audio,
            speech.as_ref(),
            Some(on_partial),
            CancellationToken::new(),
        ));
        let _ = done_tx.send(result);
    });

    loop {
        while let Ok(partial) = partial_rx.try_recv() {
            bar.set_state(BarState::Partial(partial));
            bar.draw()?;
        }
        if let Ok(result) = done_rx.try_recv() {
            let _ = worker.join();
            return finish_prepared_utterance(
                shared_config.as_ref(),
                bar,
                writer,
                awaiting_confirm,
                result,
            );
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn finish_prepared_utterance(
    config: &AppConfig,
    bar: &mut ShellBar,
    writer: &mut impl Write,
    awaiting_confirm: &mut Option<String>,
    prepared: Result<Option<crate::runtime::PreparedUtterance>>,
) -> Result<()> {
    let Some(prepared) = prepared? else {
        bar.set_state(BarState::Error(messages::no_speech(&config.language)));
        bar.draw()?;
        std::thread::sleep(Duration::from_millis(900));
        bar.set_state(BarState::Ready);
        bar.draw()?;
        return Ok(());
    };

    let must_confirm = prepared.requires_confirmation && !config.shell.skip_confirmation;
    if must_confirm {
        *awaiting_confirm = Some(prepared.prompt);
        bar.set_state(BarState::Confirm(messages::confirm(
            &config.language,
            &prepared.transcript,
        )));
        bar.draw()?;
        return Ok(());
    }

    inject_prompt(writer, config, bar, &prepared.prompt)
}

fn inject_prompt(
    writer: &mut impl Write,
    config: &AppConfig,
    bar: &mut ShellBar,
    prompt: &str,
) -> Result<()> {
    writer
        .write_all(prompt.as_bytes())
        .context("write prompt to agent")?;
    if config.shell.auto_submit {
        writer.write_all(b"\r").context("submit prompt")?;
    }
    writer.flush().context("flush agent stdin")?;
    bar.set_state(BarState::Injected(messages::injected(&config.language)));
    bar.draw()?;
    std::thread::sleep(Duration::from_millis(600));
    bar.set_state(BarState::Ready);
    bar.draw()?;
    Ok(())
}

fn copy_pty_output(
    reader: &mut dyn io::Read,
    stop: &AtomicBool,
    activity: &mpsc::Sender<()>,
    agent: SupportedAgent,
    captured_remote: &Arc<Mutex<Option<String>>>,
) {
    let mut buffer = [0_u8; 8192];
    let mut line_buffer = Vec::new();
    let mut stdout = io::stdout();
    while !stop.load(Ordering::Acquire) {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                let filtered = pty_filter::filter_agent_output(&buffer[..count]);
                if let Ok(chunk) = std::str::from_utf8(&filtered) {
                    if let Some(id) = scan_output_for_session_id(agent, chunk) {
                        *captured_remote.lock().expect("session mutex") = Some(id);
                    }
                }
                for byte in &filtered {
                    if *byte == b'\n' {
                        if let Ok(line) = std::str::from_utf8(&line_buffer) {
                            if let Some(id) = extract_session_id(agent, line) {
                                *captured_remote.lock().expect("session mutex") = Some(id);
                            }
                        }
                        line_buffer.clear();
                    } else {
                        line_buffer.push(*byte);
                    }
                }
                if !filtered.is_empty() {
                    let _ = stdout.write_all(&filtered);
                    let _ = stdout.flush();
                    let _ = pty_filter::reclaim_host_keyboard(&mut stdout);
                }
                let _ = activity.send(());
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => break,
        }
    }
}

fn terminal_size() -> Result<(u16, u16)> {
    let (cols, rows) = terminal::size().context("terminal size")?;
    Ok((cols, rows.max(2)))
}

fn setup_viewport(agent_rows: u16, total_rows: u16) -> Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "\x1b[1;{agent_rows}r")?;
    write!(stdout, "\x1b[{total_rows};1H")?;
    stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
    stdout.flush()?;
    Ok(())
}

fn resize_viewport(agent_rows: u16, total_rows: u16) -> Result<()> {
    setup_viewport(agent_rows, total_rows)
}

struct TerminalGuard {
    active: bool,
}

impl TerminalGuard {
    fn new() -> Self {
        Self { active: true }
    }

    fn restore(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        terminal::disable_raw_mode().context("disable raw mode")?;
        let mut stdout = io::stdout();
        write!(stdout, "\x1b[r\x1b[?1049l\x1b[?25h\r\n")?;
        stdout.flush()?;
        pty_filter::reclaim_host_keyboard(&mut stdout)?;
        self.active = false;
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

use crossterm::ExecutableCommand;

const fn to_supported(agent: AgentKind) -> SupportedAgent {
    match agent {
        AgentKind::Codex => SupportedAgent::Codex,
        AgentKind::Claude => SupportedAgent::Claude,
        AgentKind::Cursor => SupportedAgent::Cursor,
        AgentKind::Gemini => SupportedAgent::Gemini,
        AgentKind::Aider => SupportedAgent::Aider,
        AgentKind::Amp => SupportedAgent::Amp,
        AgentKind::OpenCode => SupportedAgent::OpenCode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_redraws_faster_than_idle() {
        assert!(BAR_REFRESH < IDLE_BAR_REFRESH);
    }
}
