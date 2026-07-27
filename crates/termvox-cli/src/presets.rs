use std::collections::BTreeMap;

use anyhow::{Result, bail};
use clap::ValueEnum;
use termvox_core::{AgentDisplayMode, AgentKind, AppConfig, PerformanceProfile, PromptDelivery};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum InitPreset {
    Cursor,
    Codex,
    Claude,
    Gemini,
    RustWeb,
}

pub(crate) fn apply_preset(config: &mut AppConfig, preset: InitPreset) {
    match preset {
        InitPreset::Cursor => {
            config.performance_profile = PerformanceProfile::Fast;
            config.agent = AgentKind::Cursor;
            config.language = "es".into();
            config.agents.cursor.display = Some(AgentDisplayMode::Companion);
            config.agents.cursor.delivery = Some(PromptDelivery::Both);
            config.agents.cursor.trust_workspace = false;
        }
        InitPreset::Codex => {
            config.performance_profile = PerformanceProfile::Fast;
            config.agent = AgentKind::Codex;
            config.agents.codex.display = Some(AgentDisplayMode::Branded);
        }
        InitPreset::Claude => {
            config.performance_profile = PerformanceProfile::Fast;
            config.agent = AgentKind::Claude;
            config.agents.claude.display = Some(AgentDisplayMode::Branded);
        }
        InitPreset::Gemini => {
            config.performance_profile = PerformanceProfile::Fast;
            config.agent = AgentKind::Gemini;
            config.agents.gemini.display = Some(AgentDisplayMode::Branded);
        }
        InitPreset::RustWeb => {
            apply_preset(config, InitPreset::Cursor);
            config.pipeline.dictionary = BTreeMap::from([
                ("a pi rest".into(), "REST API".into()),
                ("j w t".into(), "JWT".into()),
                ("react".into(), "React".into()),
                ("rust".into(), "Rust".into()),
            ]);
            config.pipeline.prefix = Some(
                "Proyecto Rust/web. Responde de forma concisa y no modifiques archivos sin confirmar."
                    .into(),
            );
        }
    }
    termvox_core::apply_performance_profile(config);
}

pub(crate) fn parse_preset(value: &str) -> Result<InitPreset> {
    match value.to_ascii_lowercase().as_str() {
        "cursor" => Ok(InitPreset::Cursor),
        "codex" => Ok(InitPreset::Codex),
        "claude" => Ok(InitPreset::Claude),
        "gemini" => Ok(InitPreset::Gemini),
        "rust-web" | "rust_web" | "rustweb" => Ok(InitPreset::RustWeb),
        other => bail!("unknown preset: {other}"),
    }
}
