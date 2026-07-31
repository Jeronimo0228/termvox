use anyhow::{Context, Result, bail};
use termvox_core::AgentKind;

pub(crate) fn parse_agent_kind(value: &str) -> Result<AgentKind> {
    match value.to_ascii_lowercase().as_str() {
        "codex" => Ok(AgentKind::Codex),
        "claude" => Ok(AgentKind::Claude),
        "cursor" => Ok(AgentKind::Cursor),
        "gemini" => Ok(AgentKind::Gemini),
        "aider" => Ok(AgentKind::Aider),
        "amp" => Ok(AgentKind::Amp),
        "opencode" | "open-code" => Ok(AgentKind::OpenCode),
        other => bail!(
            "unknown agent: {other} (expected: codex, claude, cursor, gemini, aider, amp, opencode)"
        ),
    }
}

#[cfg(unix)]
pub(crate) fn install(agent: AgentKind, force: bool) -> Result<()> {
    use std::{fs, os::unix::fs::PermissionsExt};

    let default_name = agent.default_executable();
    let home = dirs::home_dir().context("home directory")?;
    let shim_path = home.join(".local/bin").join(default_name);
    if shim_path.is_file() && !force {
        bail!(
            "{} already exists; re-run with --force to replace",
            shim_path.display()
        );
    }
    if let Some(parent) = shim_path.parent() {
        fs::create_dir_all(parent).with_context(|| parent.display().to_string())?;
    }
    let script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\nexec termvox shell --agent {} \"$@\"\n",
        agent.id()
    );
    fs::write(&shim_path, script).with_context(|| shim_path.display().to_string())?;
    let mut permissions = fs::metadata(&shim_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&shim_path, permissions)?;
    println!("Installed shim: {}", shim_path.display());
    println!("Runs: termvox shell --agent {}", agent.id());
    Ok(())
}

#[cfg(windows)]
pub(crate) fn install(agent: AgentKind, force: bool) -> Result<()> {
    let _ = (agent, force);
    bail!("install-shim is supported on Unix only; use `termvox shell --agent <name>` on Windows")
}
