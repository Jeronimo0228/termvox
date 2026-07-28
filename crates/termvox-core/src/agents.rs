use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::AgentKind;

/// How companion-mode prompts reach the active agent TUI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PromptDelivery {
    #[default]
    Clipboard,
    /// Simulate Ctrl+V in the focused window.
    Paste,
    /// Copy then auto-paste (recommended for Cursor).
    Both,
}

impl PromptDelivery {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clipboard => "clipboard",
            Self::Paste => "paste",
            Self::Both => "both",
        }
    }
}

/// How `TermVox` presents status and output for a coding-agent CLI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AgentDisplayMode {
    /// Branded footer and compact status; agent subprocess output uses agent styling.
    #[default]
    Branded,
    /// Voice layer only — transcribe and emit a styled prompt for an external agent TUI.
    Companion,
    /// Integrated mic bar wrapping the upstream agent TUI (`termvox shell`).
    Shell,
    /// Legacy multi-line `TermVox` output.
    Verbose,
}

impl AgentDisplayMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Branded => "branded",
            Self::Companion => "companion",
            Self::Shell => "shell",
            Self::Verbose => "verbose",
        }
    }
}

/// Visual identity borrowed from each upstream agent CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentUiTheme {
    pub brand: &'static str,
    pub prompt_glyph: &'static str,
    pub accent: &'static str,
    pub dim: &'static str,
    pub reset: &'static str,
    pub idle_placeholder: &'static str,
    pub tip: &'static str,
}

/// Per-agent CLI options. Only the selected `agent` uses its profile at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct AgentProfile {
    /// Override the default executable name or path.
    pub executable: Option<String>,
    /// Documented upstream flags inserted before the prompt argument.
    pub extra_args: Vec<String>,
    /// Cursor CLI only: pass `-f` for non-interactive workspace trust.
    pub trust_workspace: bool,
    /// Override the default display mode for this agent.
    pub display: Option<AgentDisplayMode>,
    /// Copy the processed prompt to the clipboard in companion mode.
    pub copy_to_clipboard: Option<bool>,
    /// Companion delivery: clipboard, paste, or both.
    pub delivery: Option<PromptDelivery>,
    /// Focus a window whose title contains this substring before auto-paste.
    pub paste_window_title: Option<String>,
}

/// Invocation options derived from the active agent profile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentInvocationOptions {
    pub extra_args: Vec<String>,
    pub trust_workspace: bool,
    /// Integrated shell mode (`termvox shell`): Cursor trust is implied for the cwd.
    pub shell_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct AgentsConfig {
    pub codex: AgentProfile,
    pub claude: AgentProfile,
    pub cursor: AgentProfile,
    pub gemini: AgentProfile,
    pub aider: AgentProfile,
    pub amp: AgentProfile,
    pub opencode: AgentProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LegacyCursorConfig {
    pub trust_workspace: bool,
}

impl AgentKind {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Cursor => "cursor",
            Self::Gemini => "gemini",
            Self::Aider => "aider",
            Self::Amp => "amp",
            Self::OpenCode => "opencode",
        }
    }

    #[must_use]
    pub const fn default_executable(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Cursor => "agent",
            Self::Gemini => "gemini",
            Self::Aider => "aider",
            Self::Amp => "amp",
            Self::OpenCode => "opencode",
        }
    }
}

impl AgentsConfig {
    #[must_use]
    pub fn profile(&self, kind: AgentKind) -> &AgentProfile {
        match kind {
            AgentKind::Codex => &self.codex,
            AgentKind::Claude => &self.claude,
            AgentKind::Cursor => &self.cursor,
            AgentKind::Gemini => &self.gemini,
            AgentKind::Aider => &self.aider,
            AgentKind::Amp => &self.amp,
            AgentKind::OpenCode => &self.opencode,
        }
    }

    #[must_use]
    pub fn resolve_executable(&self, kind: AgentKind) -> PathBuf {
        if let Some(executable) = self.profile(kind).executable.as_deref() {
            PathBuf::from(executable)
        } else {
            PathBuf::from(kind.default_executable())
        }
    }
}

impl AgentProfile {
    #[must_use]
    pub fn invocation(&self) -> AgentInvocationOptions {
        AgentInvocationOptions {
            extra_args: self.extra_args.clone(),
            trust_workspace: self.trust_workspace,
            shell_mode: false,
        }
    }

    #[must_use]
    pub fn shell_invocation(&self) -> AgentInvocationOptions {
        AgentInvocationOptions {
            extra_args: self.extra_args.clone(),
            trust_workspace: self.trust_workspace,
            shell_mode: true,
        }
    }

    #[must_use]
    pub fn resolved_display(&self, kind: AgentKind) -> AgentDisplayMode {
        self.display.unwrap_or_else(|| default_display_mode(kind))
    }

    #[must_use]
    pub fn resolved_copy_to_clipboard(&self, kind: AgentKind) -> bool {
        self.copy_to_clipboard.unwrap_or_else(|| {
            matches!(
                self.resolved_delivery(kind),
                PromptDelivery::Clipboard | PromptDelivery::Both
            )
        })
    }

    #[must_use]
    pub fn resolved_delivery(&self, kind: AgentKind) -> PromptDelivery {
        if let Some(delivery) = self.delivery {
            return delivery;
        }
        if self.resolved_display(kind) == AgentDisplayMode::Companion && kind == AgentKind::Cursor {
            PromptDelivery::Both
        } else {
            PromptDelivery::Clipboard
        }
    }

    #[must_use]
    pub fn resolved_paste_window_title(&self, kind: AgentKind) -> Option<&str> {
        if let Some(title) = self
            .paste_window_title
            .as_deref()
            .filter(|title| !title.is_empty())
        {
            return Some(title);
        }
        if kind == AgentKind::Cursor
            && self.resolved_display(kind) == AgentDisplayMode::Companion
            && matches!(
                self.resolved_delivery(kind),
                PromptDelivery::Paste | PromptDelivery::Both
            )
        {
            Some("Cursor")
        } else {
            None
        }
    }
}

#[must_use]
pub const fn default_display_mode(kind: AgentKind) -> AgentDisplayMode {
    match kind {
        // Interactive TUIs benefit from integrated shell mode by default.
        AgentKind::Cursor | AgentKind::OpenCode => AgentDisplayMode::Shell,
        _ => AgentDisplayMode::Branded,
    }
}

#[must_use]
pub const fn agent_ui(kind: AgentKind) -> AgentUiTheme {
    const DIM: &str = "\x1b[2m";
    const RESET: &str = "\x1b[0m";
    match kind {
        AgentKind::Cursor => AgentUiTheme {
            brand: "Cursor Agent",
            prompt_glyph: "→",
            accent: "\x1b[38;5;141m",
            dim: DIM,
            reset: RESET,
            idle_placeholder: "Plan, search, build anything",
            tip: "Voice via TermVox — prompt copied and pasted into Cursor automatically.",
        },
        AgentKind::Claude => AgentUiTheme {
            brand: "Claude Code",
            prompt_glyph: "❯",
            accent: "\x1b[38;5;215m",
            dim: DIM,
            reset: RESET,
            idle_placeholder: "Ask Claude anything",
            tip: "Hold to talk; Claude receives the processed prompt.",
        },
        AgentKind::Codex => AgentUiTheme {
            brand: "Codex CLI",
            prompt_glyph: "›",
            accent: "\x1b[38;5;42m",
            dim: DIM,
            reset: RESET,
            idle_placeholder: "Describe a change",
            tip: "Hold to talk; Codex exec receives JSON output.",
        },
        AgentKind::Gemini => AgentUiTheme {
            brand: "Gemini CLI",
            prompt_glyph: "✦",
            accent: "\x1b[38;5;39m",
            dim: DIM,
            reset: RESET,
            idle_placeholder: "Ask Gemini",
            tip: "Hold to talk; Gemini receives stream-json output.",
        },
        AgentKind::Aider => AgentUiTheme {
            brand: "Aider",
            prompt_glyph: ">",
            accent: "\x1b[38;5;51m",
            dim: DIM,
            reset: RESET,
            idle_placeholder: "Voice message for Aider",
            tip: "Plain-text adapter; no structured streaming.",
        },
        AgentKind::Amp => AgentUiTheme {
            brand: "Amp",
            prompt_glyph: "⚡",
            accent: "\x1b[38;5;208m",
            dim: DIM,
            reset: RESET,
            idle_placeholder: "Prompt Amp",
            tip: "Hold to talk; Amp receives stream-json output.",
        },
        AgentKind::OpenCode => AgentUiTheme {
            brand: "OpenCode",
            prompt_glyph: "◆",
            accent: "\x1b[38;5;45m",
            dim: DIM,
            reset: RESET,
            idle_placeholder: "Plan, search, build anything",
            tip: "Voice via TermVox — prompt injected into the OpenCode TUI.",
        },
    }
}

#[must_use]
pub fn agent_hints(kind: AgentKind) -> &'static [&'static str] {
    match kind {
        AgentKind::Cursor => &[
            "Use `termvox shell` for an integrated mic bar inside the Cursor Agent TUI.",
            "Default display is shell — voice injects directly into the running agent session.",
            "Set agents.cursor.trust_workspace = true when Cursor requires workspace trust.",
            "Legacy companion mode still supports auto-paste into a separate window.",
        ],
        AgentKind::Claude => {
            &["Use `termvox shell --agent claude` for integrated voice in the Claude Code TUI."]
        }
        AgentKind::Codex => {
            &["Use `termvox shell --agent codex` for integrated voice in the Codex CLI TUI."]
        }
        AgentKind::Gemini => {
            &["Use `termvox shell --agent gemini` for integrated voice in the Gemini CLI TUI."]
        }
        AgentKind::Aider => &["Use `termvox shell --agent aider` for integrated voice in Aider."],
        AgentKind::Amp => &["Use `termvox shell --agent amp` for integrated voice in Amp."],
        AgentKind::OpenCode => &[
            "Use `termvox shell --agent opencode` for integrated voice in the OpenCode TUI.",
            "Authenticate providers with `opencode auth login` before first use.",
            "Inside the TUI you can also run `/connect` to add billing/API keys.",
        ],
    }
}

#[must_use]
pub fn agent_config_warnings(config: &crate::AppConfig) -> Vec<String> {
    let mut warnings = Vec::new();
    let active = config.agent;
    let profile = config.agents.profile(active);

    if active == AgentKind::Cursor
        && !profile.trust_workspace
        && profile.resolved_display(active) != AgentDisplayMode::Shell
    {
        warnings.push(
            "agents.cursor.trust_workspace is false; Cursor may reject non-interactive runs."
                .into(),
        );
    }
    if profile.trust_workspace && active != AgentKind::Cursor {
        warnings.push(format!(
            "agents.{}.trust_workspace is ignored; it only applies to Cursor.",
            active.id()
        ));
    }
    if active == AgentKind::Cursor
        && profile.resolved_display(active) == AgentDisplayMode::Companion
        && profile.trust_workspace
    {
        warnings.push(
            "agents.cursor.trust_workspace only applies when display is branded or verbose.".into(),
        );
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_defaults_to_shell_display() {
        assert_eq!(
            default_display_mode(AgentKind::Cursor),
            AgentDisplayMode::Shell
        );
        assert_eq!(
            AgentProfile::default().resolved_display(AgentKind::Cursor),
            AgentDisplayMode::Shell
        );
    }

    #[test]
    fn codex_defaults_to_branded_display() {
        assert_eq!(
            default_display_mode(AgentKind::Codex),
            AgentDisplayMode::Branded
        );
    }

    #[test]
    fn cursor_companion_defaults_to_both_delivery() {
        let profile = AgentProfile {
            display: Some(AgentDisplayMode::Companion),
            ..Default::default()
        };
        assert_eq!(
            profile.resolved_delivery(AgentKind::Cursor),
            PromptDelivery::Both
        );
    }

    #[test]
    fn cursor_companion_defaults_to_cursor_window_title() {
        let profile = AgentProfile {
            display: Some(AgentDisplayMode::Companion),
            ..Default::default()
        };
        assert_eq!(
            profile.resolved_paste_window_title(AgentKind::Cursor),
            Some("Cursor")
        );
    }
}
