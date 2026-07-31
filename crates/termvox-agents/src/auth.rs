use std::path::{Path, PathBuf};

use termvox_core::AgentAuthStatus;
use tokio::{process::Command, time::Duration};

use crate::SupportedAgent;

const PROBE_TIMEOUT: Duration = Duration::from_secs(4);

/// Non-interactive authentication probe for a supported agent CLI.
pub async fn check_auth(kind: SupportedAgent, executable: &Path) -> AgentAuthStatus {
    match kind {
        SupportedAgent::OpenCode => check_opencode_auth(executable).await,
        SupportedAgent::Claude => check_claude_auth(),
        SupportedAgent::Codex => check_codex_auth(),
        SupportedAgent::Cursor => check_cursor_auth(),
        SupportedAgent::Gemini => check_gemini_auth(),
        SupportedAgent::Aider => check_aider_auth(),
        SupportedAgent::Amp => check_amp_auth(),
    }
}

async fn check_opencode_auth(executable: &Path) -> AgentAuthStatus {
    if opencode_auth_file_configured() {
        return AgentAuthStatus::authenticated("provider credentials on disk");
    }
    if env_any(&[
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "GOOGLE_API_KEY",
        "GEMINI_API_KEY",
        "GROQ_API_KEY",
        "MISTRAL_API_KEY",
        "XAI_API_KEY",
    ]) {
        return AgentAuthStatus::authenticated("provider API key in environment");
    }
    if let Some(output) = run_command(executable, &["auth", "list"]).await
        && looks_like_auth_list(&output)
    {
        return AgentAuthStatus::authenticated("provider credentials configured");
    }
    AgentAuthStatus::unauthenticated(
        "OpenCode has no configured providers",
        "opencode auth login",
    )
}

fn check_claude_auth() -> AgentAuthStatus {
    if env_set("ANTHROPIC_API_KEY") {
        return AgentAuthStatus::authenticated("ANTHROPIC_API_KEY is set");
    }
    if file_nonempty(&home_path(".claude/.credentials.json")) {
        return AgentAuthStatus::authenticated("Claude credentials file present");
    }
    AgentAuthStatus::unauthenticated("Claude Code is not authenticated", "claude login")
}

fn check_codex_auth() -> AgentAuthStatus {
    if env_set("OPENAI_API_KEY") {
        return AgentAuthStatus::authenticated("OPENAI_API_KEY is set");
    }
    if file_nonempty(&home_path(".codex/auth.json"))
        || file_nonempty(&home_path(".config/codex/auth.json"))
    {
        return AgentAuthStatus::authenticated("Codex credentials file present");
    }
    AgentAuthStatus::unauthenticated("Codex CLI is not authenticated", "codex login")
}

fn check_cursor_auth() -> AgentAuthStatus {
    if file_nonempty(&home_path(".cursor/auth.json"))
        || file_nonempty(&config_path("cursor/auth.json"))
    {
        return AgentAuthStatus::authenticated("Cursor credentials present");
    }
    AgentAuthStatus::unknown(
        "Cursor Agent uses your Cursor account; sign in via Cursor IDE if prompts fail",
    )
}

fn check_gemini_auth() -> AgentAuthStatus {
    if env_any(&["GEMINI_API_KEY", "GOOGLE_API_KEY"]) {
        return AgentAuthStatus::authenticated("Gemini API key in environment");
    }
    if file_nonempty(&home_path(".gemini/auth.json"))
        || file_nonempty(&config_path("gemini/auth.json"))
    {
        return AgentAuthStatus::authenticated("Gemini credentials file present");
    }
    AgentAuthStatus::unauthenticated("Gemini CLI is not authenticated", "gemini auth login")
}

fn check_aider_auth() -> AgentAuthStatus {
    if env_any(&[
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "DEEPSEEK_API_KEY",
        "GEMINI_API_KEY",
        "GROQ_API_KEY",
    ]) {
        return AgentAuthStatus::authenticated("LLM API key in environment");
    }
    AgentAuthStatus::unauthenticated(
        "Aider needs an LLM API key (for example OPENAI_API_KEY)",
        "export OPENAI_API_KEY=... && aider",
    )
}

fn check_amp_auth() -> AgentAuthStatus {
    if env_any(&["AMP_API_KEY", "OPENAI_API_KEY", "ANTHROPIC_API_KEY"]) {
        return AgentAuthStatus::authenticated("Amp API credentials in environment");
    }
    AgentAuthStatus::unknown("Amp authentication depends on your Amp account setup")
}

fn opencode_auth_file_configured() -> bool {
    let path = data_local_path("opencode/auth.json");
    json_file_has_entries(&path)
}

fn json_file_has_entries(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed == "{}" || trimmed == "[]" {
        return false;
    }
    serde_json::from_str::<serde_json::Value>(trimmed)
        .ok()
        .is_some_and(|value| match value {
            serde_json::Value::Object(map) => !map.is_empty(),
            serde_json::Value::Array(items) => !items.is_empty(),
            _ => true,
        })
}

fn looks_like_auth_list(output: &str) -> bool {
    let lines = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    !lines.is_empty()
        && !lines
            .iter()
            .any(|line| line.to_ascii_lowercase().contains("no provider"))
}

async fn run_command(executable: &Path, args: &[&str]) -> Option<String> {
    let output = tokio::time::timeout(PROBE_TIMEOUT, Command::new(executable).args(args).output())
        .await
        .ok()?
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        }
    } else {
        Some(stdout)
    }
}

fn env_set(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

fn env_any(names: &[&str]) -> bool {
    names.iter().copied().any(env_set)
}

fn file_nonempty(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.len() > 0)
}

fn home_path(relative: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(relative)
}

fn config_path(relative: &str) -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(relative)
}

fn data_local_path(relative: &str) -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_list_detection_ignores_empty_markers() {
        assert!(!looks_like_auth_list(""));
        assert!(!looks_like_auth_list("No providers configured"));
        assert!(looks_like_auth_list("anthropic\nopenai"));
    }
}
