//! Integrated agent shell: upstream CLI TUI + `TermVox` mic bar.

mod bar;
mod keys;

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
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use termvox_agents::{InteractiveLaunch, SupportedAgent};
use termvox_audio::AudioRecorder;
use termvox_core::{agent_ui, AgentAdapter, AgentKind, AppConfig, SpeechEngine};
use tokio_util::sync::CancellationToken;

use crate::runtime::{
    configured_cli_agent_kind, ensure_agent_authenticated, ensure_speech_engine, prepare_utterance,
    schedule_prewarm,
};

use self::bar::{BarState, ShellBar};

const BAR_REFRESH: Duration = Duration::from_millis(80);
const IDLE_BAR_REFRESH: Duration = Duration::from_millis(350);

/// Run the integrated shell for the selected or overridden agent.
pub async fn run(
    config: AppConfig,
    agent: AgentKind,
    trailing_args: Vec<String>,
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
    schedule_prewarm(Arc::clone(&speech), &config.whisper);
    let profile = config.agents.profile(agent);
    let launch = cli_agent.interactive_launch(
        &std::env::current_dir()?,
        &trailing_args,
        &profile.invocation(),
    );
    let config = Arc::new(config);
    tokio::task::spawn_blocking(move || run_session(&config, agent, &launch, &speech))
        .await
        .context("shell session task failed")?
}

fn run_session(
    config: &AppConfig,
    agent: AgentKind,
    launch: &InteractiveLaunch,
    speech: &Arc<dyn SpeechEngine>,
) -> Result<()> {
    let rt = tokio::runtime::Handle::current();
    terminal::enable_raw_mode().context("enable raw mode")?;
    let mut guard = TerminalGuard::new();
    let (cols, rows) = terminal_size()?;
    let agent_rows = rows.saturating_sub(1).max(1);
    setup_viewport(agent_rows, rows)?;
    let mut bar = ShellBar::new(
        agent_ui(agent),
        config.shell.hotkey.clone(),
        config.shell.exit_hotkey.clone(),
        config.language.clone(),
        rows,
        cols,
    );
    bar.draw()?;

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
    let copy_agent = thread::spawn(move || copy_pty_output(&mut reader, &stop_copy, &pty_activity_tx));

    let child = Arc::new(Mutex::new(child));
    let (exit_tx, exit_rx) = mpsc::channel();
    let child_wait = Arc::clone(&child);
    thread::spawn(move || {
        let status = child_wait.lock().expect("child mutex").wait();
        let _ = exit_tx.send(status);
    });

    let mut recorder: Option<AudioRecorder> = None;
    let mut awaiting_confirm: Option<String> = None;
    let mut anim_frame = 0_u8;
    let mut last_bar_draw = Instant::now();
    let mut running = true;

    while running {
        if exit_rx.try_recv().is_ok() {
            running = false;
            continue;
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
            if recorder.is_some() {
                anim_frame = anim_frame.wrapping_add(1);
                let level = recorder.as_ref().map_or(0.0, AudioRecorder::input_level);
                bar.set_recording_visuals(anim_frame, level);
            }
            bar.draw()?;
            last_bar_draw = Instant::now();
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
            Event::Key(key) if keys::is_voice_hotkey(&key, &config.shell.hotkey) => {
                if awaiting_confirm.is_some() {
                    continue;
                }
                if let Some(recorder_handle) = recorder.take() {
                    let audio = rt.block_on(recorder_handle.stop())?;
                    handle_finished_recording(
                        config,
                        speech,
                        &rt,
                        &mut bar,
                        &mut writer,
                        &mut awaiting_confirm,
                        audio,
                    )?;
                } else {
                    recorder = Some(AudioRecorder::start(&config.audio)?);
                    bar.set_state(BarState::Recording);
                    anim_frame = 0;
                    bar.set_recording_visuals(0, 0.0);
                    bar.draw()?;
                    last_bar_draw = Instant::now();
                }
            }
            Event::Key(key) => {
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
    let prepared = rt.block_on(prepare_utterance(
        config,
        audio,
        speech.as_ref(),
        None,
        CancellationToken::new(),
    ))?;
    let Some(prepared) = prepared else {
        bar.set_state(BarState::Error("No se detectó voz".into()));
        bar.draw()?;
        std::thread::sleep(Duration::from_millis(900));
        bar.set_state(BarState::Ready);
        bar.draw()?;
        return Ok(());
    };

    let must_confirm = prepared.requires_confirmation && !config.shell.skip_confirmation;
    if must_confirm {
        *awaiting_confirm = Some(prepared.prompt);
        bar.set_state(BarState::Confirm(prepared.transcript));
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
    bar.set_state(BarState::Injected(
        "Transcripción enviada al agente".into(),
    ));
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
) {
    let mut buffer = [0_u8; 8192];
    let mut stdout = io::stdout();
    while !stop.load(Ordering::Acquire) {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                let _ = stdout.write_all(&buffer[..count]);
                let _ = stdout.flush();
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
        write!(stdout, "\x1b[r\x1b[?25h\r\n")?;
        stdout.flush()?;
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

#[allow(dead_code)]
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
