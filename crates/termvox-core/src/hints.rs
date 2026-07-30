//! Actionable usage hints for `termvox doctor` and setup flows.

use std::path::Path;

use crate::{
    AgentDisplayMode, AgentKind, AppConfig, PerformanceProfile, SpeechEngineKind,
    WorkspaceSession, detect_environment,
};

/// Non-fatal suggestions based on the active configuration and host environment.
#[must_use]
pub fn usage_hints(config: &AppConfig) -> Vec<String> {
    let mut hints = Vec::new();
    let profile = config.agents.profile(config.agent);
    let display = profile.resolved_display(config.agent);
    let env = detect_environment();

    if matches!(config.agent, AgentKind::Cursor | AgentKind::OpenCode)
        && display == AgentDisplayMode::Companion
    {
        hints.push(
            "agents.{}.display = \"shell\" (or run `termvox shell`) gives an integrated mic bar instead of paste-into-another-window."
                .replace("{}", config.agent.id()),
        );
    }

    if config.speech_engine == SpeechEngineKind::WhisperCpp {
        let model_name = config
            .whisper
            .model
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        match config.performance_profile {
            PerformanceProfile::Balanced if model_name.contains("tiny") => {
                hints.push(
                    "performance_profile = \"balanced\" pairs best with ggml-base.bin; run `termvox models install accurate`."
                        .into(),
                );
            }
            PerformanceProfile::Accurate if model_name.contains("tiny") => {
                hints.push(
                    "performance_profile = \"accurate\" expects ggml-base.bin or larger; run `termvox models install accurate`."
                        .into(),
                );
            }
            PerformanceProfile::Fast if model_name.contains("base") => {
                hints.push(
                    "performance_profile = \"fast\" uses ggml-tiny.bin for lower latency; run `termvox models install default` if intentional."
                        .into(),
                );
            }
            _ => {}
        }
    }

    if env.wayland {
        hints.push(
            "Wayland: use `termvox shell` with F8 or Ctrl+Space inside the terminal; for global hotkeys run `termvox daemon start`."
                .into(),
        );
    }

    if config.workspace.persist_session
        && let Ok(cwd) = std::env::current_dir()
    {
        let path = WorkspaceSession::session_path(&cwd, config.workspace.session_file.as_deref());
        hints.push(format!(
            "Workspace session file: {} (use `termvox shell --fresh` to ignore).",
            path.display()
        ));
    }

    hints
}

/// Returns true when the cwd already has a project-layer config file.
#[must_use]
pub fn project_config_exists(cwd: &Path) -> bool {
    cwd.join("termvox.toml").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_companion_cursor_suggests_shell() {
        let mut config = AppConfig::default();
        config.agent = AgentKind::Cursor;
        config.agents.cursor.display = Some(AgentDisplayMode::Companion);
        let hints = usage_hints(&config);
        assert!(hints.iter().any(|hint| hint.contains("termvox shell")));
    }
}
