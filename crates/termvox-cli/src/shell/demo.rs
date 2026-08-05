//! Scripted demo flow for marketing recordings (`termvox shell --demo --demo-auto`).

use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use termvox_agents::InteractiveLaunch;
use termvox_core::AgentKind;

use super::bar::{BarState, ShellBar};
use super::messages;

const DEFAULT_PROMPT: &str = "Refactor the auth module to use structured errors";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Settle,
    Recording,
    Transcribing,
    Confirm,
    Hold,
    Exit,
}

pub(super) struct DemoAuto {
    phase: Phase,
    phase_until: Instant,
    prompt: String,
    anim_frame: u8,
}

impl DemoAuto {
    pub(super) fn new() -> Self {
        Self {
            phase: Phase::Settle,
            phase_until: Instant::now() + Duration::from_secs(3),
            prompt: std::env::var("TERMVOX_DEMO_PROMPT").unwrap_or_else(|_| DEFAULT_PROMPT.into()),
            anim_frame: 0,
        }
    }

    pub(super) fn tick(
        &mut self,
        bar: &mut ShellBar,
        writer: &mut impl Write,
        language: &str,
    ) -> Result<bool> {
        let now = Instant::now();
        if now < self.phase_until {
            if self.phase == Phase::Recording {
                self.anim_frame = self.anim_frame.wrapping_add(1);
                bar.set_recording_visuals(self.anim_frame, 0.35 + (self.anim_frame % 5) as f32 * 0.12);
                bar.set_state(BarState::Recording);
                bar.draw()?;
            } else if self.phase == Phase::Transcribing {
                self.anim_frame = self.anim_frame.wrapping_add(1);
                bar.set_state(BarState::Transcribing);
                bar.draw()?;
            }
            return Ok(false);
        }

        match self.phase {
            Phase::Settle => {
                self.phase = Phase::Recording;
                self.phase_until = now + Duration::from_millis(3500);
                bar.set_state(BarState::Recording);
                bar.set_recording_visuals(0, 0.2);
                bar.draw()?;
            }
            Phase::Recording => {
                self.phase = Phase::Transcribing;
                self.phase_until = now + Duration::from_millis(2200);
                bar.set_state(BarState::Transcribing);
                bar.draw()?;
            }
            Phase::Transcribing => {
                self.phase = Phase::Confirm;
                self.phase_until = now + Duration::from_millis(3200);
                bar.set_state(BarState::Confirm(messages::confirm(
                    language,
                    &self.prompt,
                )));
                bar.draw()?;
            }
            Phase::Confirm => {
                inject_prompt_demo(writer, bar, language, &self.prompt)?;
                self.phase = Phase::Hold;
                self.phase_until = now + Duration::from_secs(10);
            }
            Phase::Hold => {
                self.phase = Phase::Exit;
                self.phase_until = now + Duration::from_millis(500);
            }
            Phase::Exit => return Ok(true),
        }
        Ok(false)
    }
}

pub(super) fn demo_script_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("TERMVOX_DEMO_SCRIPT") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        bail!("TERMVOX_DEMO_SCRIPT not found: {}", path.display());
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("../share/termvox/demo-agent-tui.sh");
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidate = manifest.join("../../scripts/demo-agent-tui.sh");
    if candidate.is_file() {
        return Ok(candidate.canonicalize().unwrap_or(candidate));
    }

    bail!(
        "demo script not found; set TERMVOX_DEMO_SCRIPT to scripts/demo-agent-tui.sh"
    );
}

pub(super) fn demo_launch(agent: AgentKind, cwd: &Path) -> Result<InteractiveLaunch> {
    let script = demo_script_path()?;
    Ok(InteractiveLaunch {
        executable: PathBuf::from("bash"),
        args: vec![
            script.to_string_lossy().into_owned(),
            agent.id().to_string(),
        ],
        cwd: cwd.to_path_buf(),
    })
}

fn inject_prompt_demo(
    writer: &mut impl Write,
    bar: &mut ShellBar,
    language: &str,
    prompt: &str,
) -> Result<()> {
    writer
        .write_all(prompt.as_bytes())
        .context("write demo prompt to agent")?;
    writer.write_all(b"\r").context("submit demo prompt")?;
    writer.flush().context("flush agent stdin")?;
    bar.set_state(BarState::Injected(messages::injected(language)));
    bar.draw()?;
    Ok(())
}
