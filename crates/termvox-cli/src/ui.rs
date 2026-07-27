use std::io::{self, Write};

use anyhow::{Result, bail};
use crossterm::{
    event::KeyCode,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use termvox_core::AgentEvent;

pub(crate) fn confirm(message: &str) -> Result<bool> {
    print!("{message}");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"))
}

pub(crate) fn print_agent_event(event: &AgentEvent) {
    match event {
        AgentEvent::Message { text } | AgentEvent::Completed { result: text } => {
            println!("{text}");
        }
        AgentEvent::Failed { message } => eprintln!("Agent failed: {message}"),
        AgentEvent::Tool { name, status } => tracing::info!(%name, %status, "agent tool"),
        AgentEvent::Started { .. } => {}
    }
}

pub(crate) fn parse_key(value: &str) -> Result<KeyCode> {
    let upper = value.trim().to_uppercase();
    match upper.as_str() {
        "SPACE" => Ok(KeyCode::Char(' ')),
        "ENTER" => Ok(KeyCode::Enter),
        "TAB" => Ok(KeyCode::Tab),
        _ if upper.starts_with('F') => upper[1..]
            .parse::<u8>()
            .ok()
            .filter(|number| (1..=24).contains(number))
            .map(KeyCode::F)
            .ok_or_else(|| anyhow::anyhow!("unsupported push_to_talk key: {value}")),
        _ if value.chars().count() == 1 => Ok(KeyCode::Char(
            value.chars().next().expect("length checked above"),
        )),
        _ => bail!("unsupported push_to_talk key: {value}"),
    }
}

pub(crate) struct RawMode;

impl RawMode {
    pub(crate) fn enter() -> Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_configured_ptt_keys() {
        assert_eq!(parse_key("SPACE").unwrap(), KeyCode::Char(' '));
        assert_eq!(parse_key("F8").unwrap(), KeyCode::F(8));
        assert!(parse_key("ALT+SPACE").is_err());
    }
}
