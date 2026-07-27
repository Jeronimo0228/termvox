use anyhow::Result;
use termvox_audio::input_devices;
use termvox_core::{AgentAdapter, AppConfig, SpeechEngineKind, agent_config_warnings, agent_hints};
use termvox_hotkeys::detect_support;

use crate::runtime::{all_agents, speech_engine};

pub(crate) async fn run(config: AppConfig, json: bool) -> Result<()> {
    if json {
        return json_report(&config).await;
    }

    println!("TermVox doctor\n");
    check("configuration", config.validate().map(|()| "valid".into()));
    check(
        "microphone",
        input_devices().and_then(|devices| {
            if devices.is_empty() {
                Err(termvox_core::TermVoxError::Audio("no input devices".into()))
            } else {
                Ok(format!(
                    "{} device(s): {}",
                    devices.len(),
                    devices.join(", ")
                ))
            }
        }),
    );
    let engine = speech_engine(&config);
    check(
        &format!("speech/{}", engine.id()),
        engine.healthcheck().await.map(|()| {
            if config.speech_engine == SpeechEngineKind::WhisperCpp {
                format!("ready ({})", config.whisper.model.display())
            } else {
                "ready".into()
            }
        }),
    );
    for agent in all_agents() {
        let info = agent.probe().await;
        let detail = info.version.unwrap_or_else(|| info.executable.clone());
        if info.installed {
            println!("[ok] agent/{:<10} {}", info.id, detail);
        } else {
            println!("[--] agent/{:<10} not installed", info.id);
        }
    }
    println!("\nSelected agent: {}", config.agent.id());
    let profile = config.agents.profile(config.agent);
    if profile.executable.is_some() || profile.trust_workspace || !profile.extra_args.is_empty() {
        println!("Active profile ([agents.{}]):", config.agent.id());
        if let Some(executable) = &profile.executable {
            println!("  executable = {executable}");
        }
        if profile.trust_workspace {
            println!("  trust_workspace = true");
        }
        if !profile.extra_args.is_empty() {
            println!("  extra_args = {:?}", profile.extra_args);
        }
    }
    println!(
        "  display = {}",
        profile.resolved_display(config.agent).as_str()
    );
    for warning in agent_config_warnings(&config) {
        println!("[!!] {warning}");
    }
    for hint in agent_hints(config.agent) {
        println!("  hint: {hint}");
    }
    println!(
        "\nTerminal PTT: {}. Global hotkeys are optional and platform-specific.",
        config.push_to_talk
    );
    let hotkey = detect_support();
    println!(
        "Hotkey backend: {:?} ({})",
        hotkey.backend,
        if hotkey.global_available {
            "available"
        } else {
            "fallback required"
        }
    );
    if let Some(guidance) = hotkey.guidance {
        println!("  {guidance}");
    }
    Ok(())
}

async fn json_report(config: &AppConfig) -> Result<()> {
    let microphone = input_devices().map_or_else(
        |error| serde_json::json!({"ok": false, "error": error.to_string()}),
        |devices| serde_json::json!({"ok": !devices.is_empty(), "devices": devices}),
    );
    let engine = speech_engine(config);
    let model = (config.speech_engine == SpeechEngineKind::WhisperCpp)
        .then(|| config.whisper.model.display().to_string());
    let speech = match engine.healthcheck().await {
        Ok(()) => serde_json::json!({
            "ok": true,
            "provider": engine.id(),
            "model": model,
            "local": config.speech_engine == SpeechEngineKind::WhisperCpp,
        }),
        Err(error) => {
            serde_json::json!({
                "ok": false,
                "provider": engine.id(),
                "model": model,
                "local": config.speech_engine == SpeechEngineKind::WhisperCpp,
                "error": error.to_string()
            })
        }
    };
    let mut agents = Vec::new();
    for agent in all_agents() {
        agents.push(serde_json::to_value(agent.probe().await)?);
    }
    let hotkey = detect_support();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "configuration": {"ok": config.validate().is_ok()},
            "microphone": microphone,
            "speech": speech,
            "agents": agents,
            "hotkey": {
                "backend": format!("{:?}", hotkey.backend),
                "global_available": hotkey.global_available,
                "key_release_available": hotkey.key_release_available,
                "guidance": hotkey.guidance,
            }
        }))?
    );
    Ok(())
}

fn check(label: &str, result: termvox_core::Result<String>) {
    match result {
        Ok(detail) => println!("[ok] {label:<18} {detail}"),
        Err(error) => println!("[!!] {label:<18} {error}"),
    }
}
