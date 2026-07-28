//! Status bar rendered on the bottom terminal row in shell mode.

use std::io::{self, Write};

use crossterm::{
    cursor,
    style::{Print, SetAttribute, Attribute},
    ExecutableCommand,
};
use termvox_core::AgentUiTheme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum BarState {
    Ready,
    Recording,
    Transcribing,
    Partial(String),
    Confirm(String),
    Injected(String),
    Error(String),
}

pub(super) struct ShellBar {
    theme: AgentUiTheme,
    hotkey: String,
    language: String,
    state: BarState,
    row: u16,
    cols: u16,
}

impl ShellBar {
    pub(super) fn new(
        theme: AgentUiTheme,
        hotkey: String,
        language: String,
        row: u16,
        cols: u16,
    ) -> Self {
        Self {
            theme,
            hotkey,
            language,
            state: BarState::Ready,
            row,
            cols,
        }
    }

    pub(super) fn set_size(&mut self, row: u16, cols: u16) {
        self.row = row;
        self.cols = cols;
    }

    pub(super) fn set_state(&mut self, state: BarState) {
        self.state = state;
    }

    pub(super) fn draw(&self) -> io::Result<()> {
        let message = match &self.state {
            BarState::Ready => format!(
                "🎤 TermVox · {} · {} · {} · listo",
                self.theme.brand, self.hotkey, self.language
            ),
            BarState::Recording => format!("🔴 Grabando… {} para terminar", self.hotkey),
            BarState::Transcribing => "⏳ Transcribiendo…".into(),
            BarState::Partial(text) => truncate(format!("… {text}"), self.cols as usize),
            BarState::Confirm(prompt) => truncate(
                format!("¿Enviar? [y/N] {prompt}"),
                self.cols as usize,
            ),
            BarState::Injected(detail) => truncate(format!("✓ {detail}"), self.cols as usize),
            BarState::Error(error) => truncate(format!("✗ {error}"), self.cols as usize),
        };
        let mut stdout = io::stdout();
        stdout.execute(cursor::SavePosition)?;
        stdout.execute(cursor::MoveTo(0, self.row.saturating_sub(1)))?;
        stdout.execute(SetAttribute(Attribute::Reverse))?;
        stdout.execute(Print(format!(" {:<width$}", message, width = self.cols as usize - 1)))?;
        stdout.execute(SetAttribute(Attribute::Reset))?;
        stdout.execute(cursor::RestorePosition)?;
        stdout.flush()?;
        Ok(())
    }
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
