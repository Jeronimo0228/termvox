//! Status bar rendered on the bottom terminal row in shell mode.

use std::io::{self, Write};

use crossterm::{
    cursor,
    style::{Attribute, Print, SetAttribute},
    ExecutableCommand,
};
use termvox_core::AgentUiTheme;

use super::messages;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum BarState {
    Ready,
    Recording,
    Transcribing,
    Partial(String),
    Confirm(String),
    Injected(String),
    Error(String),
    Exiting,
    Notice(String),
}

pub(super) struct ShellBar {
    theme: AgentUiTheme,
    hotkeys: Vec<String>,
    exit_hotkey: String,
    language: String,
    session_hint: Option<String>,
    state: BarState,
    row: u16,
    cols: u16,
    recording_frame: u8,
    input_level: f32,
}

impl ShellBar {
    pub(super) fn new(
        theme: AgentUiTheme,
        hotkeys: Vec<String>,
        exit_hotkey: String,
        language: String,
        row: u16,
        cols: u16,
    ) -> Self {
        Self {
            theme,
            hotkeys,
            exit_hotkey,
            language,
            session_hint: None,
            state: BarState::Ready,
            row,
            cols,
            recording_frame: 0,
            input_level: 0.0,
        }
    }

    pub(super) fn set_session_hint(&mut self, session_id: Option<String>) {
        self.session_hint = session_id;
    }

    pub(super) fn set_size(&mut self, row: u16, cols: u16) {
        self.row = row;
        self.cols = cols;
    }

    pub(super) fn set_state(&mut self, state: BarState) {
        self.state = state;
    }

    pub(super) fn set_recording_visuals(&mut self, frame: u8, input_level: f32) {
        self.recording_frame = frame;
        self.input_level = input_level.clamp(0.0, 1.0);
    }

    pub(super) fn needs_animation(&self) -> bool {
        matches!(
            self.state,
            BarState::Recording | BarState::Transcribing | BarState::Partial(_)
        )
    }

    pub(super) fn draw(&self) -> io::Result<()> {
        let hotkey_hint = self
            .hotkeys
            .iter()
            .take(2)
            .cloned()
            .collect::<Vec<_>>()
            .join("/");
        let message = match &self.state {
            BarState::Ready => format!(
                "{}{} TermVox · {} · {} · {} · {} {hotkey_hint} · {} {}{}",
                self.theme.dim,
                self.theme.prompt_glyph,
                self.theme.brand,
                messages::language_tag(&self.language),
                messages::ready(&self.language, self.session_hint.as_deref()),
                messages::voice_label(&self.language),
                messages::exit_label(&self.language),
                self.exit_hotkey,
                self.theme.reset,
            ),
            BarState::Recording => format!(
                "{}{} {} {} · {hotkey_hint}{}",
                self.theme.accent,
                self.theme.prompt_glyph,
                level_meter(self.input_level, self.recording_frame),
                messages::recording(&self.language),
                self.theme.reset,
            ),
            BarState::Transcribing => format!(
                "{}{} {}{}{}",
                self.theme.accent,
                self.theme.prompt_glyph,
                messages::transcribing(&self.language),
                messages::transcribing_dots(self.recording_frame),
                self.theme.reset
            ),
            BarState::Partial(text) => truncate(
                format!(
                    "{}{} {} {text}{}",
                    self.theme.accent,
                    self.theme.prompt_glyph,
                    messages::partial_prefix(&self.language),
                    self.theme.reset
                ),
                self.cols as usize,
            ),
            BarState::Confirm(prompt) => truncate(
                messages::confirm(&self.language, prompt),
                self.cols as usize,
            ),
            BarState::Injected(_) => truncate(
                format!("✓ {}", messages::injected(&self.language)),
                self.cols as usize,
            ),
            BarState::Error(error) => truncate(format!("✗ {error}"), self.cols as usize),
            BarState::Exiting => format!(
                "{}{}{}",
                self.theme.dim,
                messages::exiting(&self.language),
                self.theme.reset
            ),
            BarState::Notice(text) => truncate(text.clone(), self.cols as usize),
        };
        let mut stdout = io::stdout();
        stdout.execute(cursor::SavePosition)?;
        stdout.execute(cursor::MoveTo(0, self.row.saturating_sub(1)))?;
        stdout.execute(SetAttribute(Attribute::Reverse))?;
        stdout.execute(Print(format!(
            " {:<width$}",
            message,
            width = self.cols.saturating_sub(1) as usize
        )))?;
        stdout.execute(SetAttribute(Attribute::Reset))?;
        stdout.execute(cursor::RestorePosition)?;
        stdout.flush()?;
        Ok(())
    }
}

fn level_meter(level: f32, frame: u8) -> String {
    let bars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let mut out = String::with_capacity(8);
    for index in 0..8 {
        let threshold = (index as f32 + 1.0) / 8.0;
        let animated = (level + (f32::from(frame) * 0.04).sin().abs() * 0.08).clamp(0.0, 1.0);
        out.push_str(if animated >= threshold {
            bars[7]
        } else if animated + 0.08 >= threshold {
            bars[4]
        } else {
            bars[0]
        });
    }
    out
}

fn truncate(value: String, max: usize) -> String {
    if value.len() <= max {
        return value;
    }
    if max <= 3 {
        return value.chars().take(max).collect();
    }
    format!("{}…", value.chars().take(max - 1).collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_meter_reacts_to_input() {
        let quiet = level_meter(0.05, 0);
        let loud = level_meter(0.9, 0);
        assert_ne!(quiet, loud);
    }
}
