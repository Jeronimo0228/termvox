use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use termvox_core::{AgentKind, AppConfig, SpeechEngineKind};

pub(crate) fn load_config(explicit: Option<&Path>) -> Result<AppConfig> {
    let global = global_config_path();
    let project = explicit.map_or_else(|| PathBuf::from("termvox.toml"), Path::to_path_buf);
    let config = AppConfig::load(Some(&global), Some(&project))?;
    config.validate()?;
    Ok(config)
}

pub(crate) fn init_config(global: bool, force: bool, interactive: bool) -> Result<()> {
    let path = if global {
        global_config_path()
    } else {
        PathBuf::from("termvox.toml")
    };
    if path.exists() && !force {
        bail!(
            "{} already exists; pass --force to replace it",
            path.display()
        );
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let mut config = AppConfig::default();
    if interactive && io::stdin().is_terminal() {
        config.language = prompt_default("Language", &config.language)?;
        config.push_to_talk = prompt_default("Push-to-talk key", &config.push_to_talk)?;
        let engine = prompt_default(
            "Speech engine (whispercpp/openai/parakeet/vosk)",
            "whispercpp",
        )?;
        config.speech_engine = match engine.to_lowercase().as_str() {
            "openai" => SpeechEngineKind::OpenAi,
            "parakeet" => SpeechEngineKind::Parakeet,
            "vosk" => SpeechEngineKind::Vosk,
            _ => SpeechEngineKind::WhisperCpp,
        };
        let agent = prompt_default("Agent (codex/claude/cursor/gemini/aider/amp)", "codex")?;
        config.agent = parse_agent(&agent)?;
    }
    let text = toml::to_string_pretty(&config)?;
    std::fs::write(&path, text)?;
    println!("Created {}", path.display());
    Ok(())
}

pub(crate) fn global_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("termvox/termvox.toml")
}

fn parse_agent(value: &str) -> Result<AgentKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "codex" => Ok(AgentKind::Codex),
        "claude" => Ok(AgentKind::Claude),
        "cursor" => Ok(AgentKind::Cursor),
        "gemini" => Ok(AgentKind::Gemini),
        "aider" => Ok(AgentKind::Aider),
        "amp" => Ok(AgentKind::Amp),
        _ => bail!("unsupported agent: {value}"),
    }
}

fn prompt_default(label: &str, default: &str) -> Result<String> {
    print!("{label} [{default}]: ");
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    let value = value.trim();
    Ok(if value.is_empty() {
        default.to_owned()
    } else {
        value.to_owned()
    })
}
