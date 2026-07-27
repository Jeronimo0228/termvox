use std::io::{self, Write};

use termvox_core::{AgentDisplayMode, AgentKind, AgentUiTheme, agent_ui};

use crate::delivery::DeliveryOutcome;

pub(crate) struct SessionUi {
    theme: AgentUiTheme,
    mode: AgentDisplayMode,
    ptt_label: String,
    toggle: bool,
}

impl Clone for SessionUi {
    fn clone(&self) -> Self {
        Self {
            theme: self.theme,
            mode: self.mode,
            ptt_label: self.ptt_label.clone(),
            toggle: self.toggle,
        }
    }
}

impl SessionUi {
    pub(crate) fn new(
        kind: AgentKind,
        mode: AgentDisplayMode,
        ptt_label: &str,
        toggle: bool,
    ) -> Self {
        Self {
            theme: agent_ui(kind),
            mode,
            ptt_label: ptt_label.to_owned(),
            toggle,
        }
    }

    pub(crate) fn mode(&self) -> AgentDisplayMode {
        self.mode
    }

    pub(crate) fn theme(&self) -> &AgentUiTheme {
        &self.theme
    }

    pub(crate) fn show_startup(&self, agent_version: Option<&str>) {
        match self.mode {
            AgentDisplayMode::Verbose => {
                println!(
                    "TermVox ready. {} {} to talk; press q or Ctrl+C to quit.",
                    self.toggle_action(),
                    self.ptt_label
                );
            }
            AgentDisplayMode::Branded | AgentDisplayMode::Companion => {
                let version = agent_version
                    .map(|value| format!(" {value}"))
                    .unwrap_or_default();
                eprintln!(
                    "{}{}{}{}",
                    self.theme.dim, self.theme.brand, version, self.theme.reset
                );
                eprintln!("{}{}{}", self.theme.dim, self.theme.tip, self.theme.reset);
                self.write_status(&self.idle_line());
            }
        }
    }

    pub(crate) fn show_global_ready(&self, shortcut: &str) {
        match self.mode {
            AgentDisplayMode::Verbose => {
                println!(
                    "TermVox ready. {} {shortcut} globally to talk; Ctrl+C quits.",
                    self.toggle_action()
                );
            }
            AgentDisplayMode::Branded | AgentDisplayMode::Companion => {
                eprintln!(
                    "{}{}{}{}",
                    self.theme.dim, self.theme.brand, self.theme.reset, ""
                );
                self.write_status(&format!(
                    "{}{} {shortcut} globally · Ctrl+C quits{}",
                    self.theme.dim,
                    self.toggle_action(),
                    self.theme.reset
                ));
            }
        }
    }

    pub(crate) fn show_recording(&self) {
        match self.mode {
            AgentDisplayMode::Verbose => print!("Recording...\r"),
            AgentDisplayMode::Branded | AgentDisplayMode::Companion => self.write_status(&format!(
                "{}{} Listening…{}",
                self.theme.accent, self.theme.prompt_glyph, self.theme.reset
            )),
        }
        let _ = io::stdout().flush();
    }

    pub(crate) fn show_transcribing(&self) {
        match self.mode {
            AgentDisplayMode::Verbose => {
                print!("\rTranscribing...\r");
                let _ = io::stdout().flush();
            }
            AgentDisplayMode::Branded | AgentDisplayMode::Companion => self.write_status(&format!(
                "{}{} Transcribing…{}",
                self.theme.accent, self.theme.prompt_glyph, self.theme.reset
            )),
        }
    }

    pub(crate) fn show_no_speech(&self) {
        match self.mode {
            AgentDisplayMode::Verbose => println!("No speech detected."),
            AgentDisplayMode::Branded | AgentDisplayMode::Companion => {
                self.write_status(&format!(
                    "{}{} No speech detected{}",
                    self.theme.dim, self.theme.prompt_glyph, self.theme.reset
                ));
            }
        }
    }

    pub(crate) fn show_partial_transcript(&self, partial: &str) {
        let partial = partial.trim();
        if partial.is_empty() {
            return;
        }
        let preview = truncate_preview(partial, 72);
        match self.mode {
            AgentDisplayMode::Verbose => {
                print!("\rPartial: {preview}\r");
                let _ = io::stdout().flush();
            }
            AgentDisplayMode::Branded | AgentDisplayMode::Companion => self.write_status(&format!(
                "{}{} {preview}…{}",
                self.theme.accent, self.theme.prompt_glyph, self.theme.reset
            )),
        }
    }

    pub(crate) fn show_transcript(
        &self,
        heard: &str,
        prompt: &str,
        duration_ms: u64,
        risk_matches: &[String],
    ) {
        match self.mode {
            AgentDisplayMode::Verbose => {
                println!("\nHeard:  {heard}");
                println!("Prompt: {prompt}");
                if !risk_matches.is_empty() {
                    println!("Risk signals: {}", risk_matches.join(", "));
                }
                println!("({duration_ms} ms)");
            }
            AgentDisplayMode::Branded => {
                println!();
                println!(
                    "{}{}{} {heard}",
                    self.theme.accent, self.theme.prompt_glyph, self.theme.reset
                );
                if prompt != heard {
                    eprintln!(
                        "{}{} Prompt: {prompt}{}",
                        self.theme.dim, self.theme.prompt_glyph, self.theme.reset
                    );
                }
                if !risk_matches.is_empty() {
                    eprintln!(
                        "{}{} Risk: {}{}",
                        self.theme.dim,
                        self.theme.prompt_glyph,
                        risk_matches.join(", "),
                        self.theme.reset
                    );
                }
                eprintln!("{}{duration_ms} ms{}", self.theme.dim, self.theme.reset);
            }
            AgentDisplayMode::Companion => {
                println!();
                println!(
                    "{}{}{} {prompt}",
                    self.theme.accent, self.theme.prompt_glyph, self.theme.reset
                );
                if !risk_matches.is_empty() {
                    eprintln!(
                        "{}{} Risk: {}{}",
                        self.theme.dim,
                        self.theme.prompt_glyph,
                        risk_matches.join(", "),
                        self.theme.reset
                    );
                }
                eprintln!(
                    "{}{duration_ms} ms · copied and pasted into {}{}",
                    self.theme.dim, self.theme.brand, self.theme.reset
                );
            }
        }
    }

    pub(crate) fn show_delivery(&self, outcome: &DeliveryOutcome, window_title: Option<&str>) {
        if self.mode == AgentDisplayMode::Companion {
            let detail = match (outcome.clipboard, outcome.paste) {
                (true, true) => match window_title {
                    Some(title) => format!("Copied and pasted into {title}"),
                    None => "Copied and pasted".into(),
                },
                (true, false) => "Copied to clipboard".into(),
                (false, true) => match window_title {
                    Some(title) => format!("Pasted into {title}"),
                    None => "Pasted into focused window".into(),
                },
                (false, false) => "Delivered".into(),
            };
            self.write_status(&format!(
                "{}{} {detail}{}",
                self.theme.accent, self.theme.prompt_glyph, self.theme.reset
            ));
        }
    }

    pub(crate) fn show_delivery_failed(&self, error: &str) {
        eprintln!(
            "{}{} Delivery failed: {error}{}",
            self.theme.dim, self.theme.prompt_glyph, self.theme.reset
        );
    }

    pub(crate) fn show_clipboard_copied(&self) {
        if self.mode == AgentDisplayMode::Companion {
            self.write_status(&format!(
                "{}{} Copied to clipboard — paste with Ctrl+V{}",
                self.theme.accent, self.theme.prompt_glyph, self.theme.reset
            ));
        }
    }

    pub(crate) fn show_clipboard_failed(&self, error: &str) {
        eprintln!(
            "{}{} Clipboard failed: {error}{}",
            self.theme.dim, self.theme.prompt_glyph, self.theme.reset
        );
    }

    pub(crate) fn show_confirm_prompt(&self) -> String {
        match self.mode {
            AgentDisplayMode::Verbose => "Send to agent? [y/N] ".into(),
            AgentDisplayMode::Branded | AgentDisplayMode::Companion => format!(
                "{}{} Send? [y/N] {}",
                self.theme.accent, self.theme.prompt_glyph, self.theme.reset
            ),
        }
    }

    pub(crate) fn show_cancelled(&self) {
        match self.mode {
            AgentDisplayMode::Verbose => println!("Cancelled."),
            AgentDisplayMode::Branded | AgentDisplayMode::Companion => {
                self.write_status(&format!(
                    "{}{} Cancelled{}",
                    self.theme.dim, self.theme.prompt_glyph, self.theme.reset
                ));
            }
        }
    }

    pub(crate) fn show_idle(&self) {
        match self.mode {
            AgentDisplayMode::Verbose => {
                print!("\rHold {} to talk; q quits.\r", self.ptt_label);
                let _ = io::stdout().flush();
            }
            AgentDisplayMode::Branded | AgentDisplayMode::Companion => {
                self.write_status(&self.idle_line());
            }
        }
    }

    pub(crate) fn show_error(&self, error: &str) {
        eprintln!("TermVox: {error}");
        self.show_idle();
    }

    fn idle_line(&self) -> String {
        let action = self.toggle_action();
        format!(
            "{}{}{} {}{} · {action} {} · q quit{}",
            self.theme.accent,
            self.theme.prompt_glyph,
            self.theme.reset,
            self.theme.dim,
            self.theme.idle_placeholder,
            self.ptt_label,
            self.theme.reset,
        )
    }

    fn toggle_action(&self) -> &'static str {
        if self.toggle { "Press" } else { "Hold" }
    }

    fn write_status(&self, line: &str) {
        print!("\r\x1b[K{line}");
        let _ = io::stdout().flush();
    }
}

fn truncate_preview(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    text.chars().take(max_chars).collect()
}
