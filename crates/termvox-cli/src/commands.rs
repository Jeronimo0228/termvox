use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Result, anyhow};
use clap::CommandFactory;
use clap_complete::Shell;
use termvox_core::{AgentAdapter, AppConfig};
use termvox_plugin_sdk::{PluginClient, PluginSpawnOptions};
use termvox_speech::{DownloadProgress, ModelArtifact, ModelManager, ModelManifest};
use tokio_util::sync::CancellationToken;

use crate::{
    cli::{Cli, ConfigCommand, ModelCommand, PluginCommand},
    runtime::all_agents,
    setup::{global_config_path, load_config},
};

pub(crate) async fn plugins(config: AppConfig, command: Option<PluginCommand>) -> Result<()> {
    match command.unwrap_or(PluginCommand::List) {
        PluginCommand::List => {
            println!("Official agent adapters:");
            for agent in all_agents() {
                let info = agent.probe().await;
                println!(
                    "- {:<8} {:<13} {}",
                    info.id,
                    if info.installed {
                        "installed"
                    } else {
                        "not installed"
                    },
                    info.version.unwrap_or_default()
                );
            }
            println!("\nConfigured external plugins:");
            for plugin in &config.plugins {
                println!(
                    "- {:<16} {} {}",
                    plugin.id,
                    if plugin.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    },
                    plugin.executable.display()
                );
            }
        }
        PluginCommand::Inspect { id } => {
            let plugin = configured_plugin(&config, &id)?;
            let client = spawn_plugin(plugin).await?;
            println!("{}", serde_json::to_string_pretty(client.manifest())?);
            client.shutdown().await?;
        }
        PluginCommand::Test { id } => {
            let plugin = configured_plugin(&config, &id)?;
            let mut client = spawn_plugin(plugin).await?;
            let result = client.probe().await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            client.shutdown().await?;
            println!("Plugin conforms to initialize/probe/shutdown");
        }
    }
    Ok(())
}

pub(crate) fn config(command: Option<ConfigCommand>, explicit: Option<&Path>) -> Result<()> {
    match command.unwrap_or(ConfigCommand::Show) {
        ConfigCommand::Path => {
            println!("global:  {}", global_config_path().display());
            println!(
                "project: {}",
                explicit
                    .unwrap_or_else(|| Path::new("termvox.toml"))
                    .display()
            );
        }
        ConfigCommand::Show => {
            println!("{}", toml::to_string_pretty(&load_config(explicit)?)?);
        }
        ConfigCommand::Validate => {
            load_config(explicit)?;
            println!("Configuration is valid");
        }
    }
    Ok(())
}

pub(crate) async fn model(config: AppConfig, command: ModelCommand) -> Result<()> {
    match command {
        ModelCommand::List => {
            let manifest = ModelManifest::bundled()?;
            for artifact in manifest.artifacts {
                println!(
                    "{}\t{}\t{}\t{}\t{} bytes",
                    artifact.id,
                    artifact.provider,
                    artifact.version,
                    artifact.license,
                    artifact.size_bytes
                );
            }
        }
        ModelCommand::Install { id } => {
            let manifest = ModelManifest::bundled()?;
            let artifact = resolve_model(&manifest, &id)?;
            let destination = model_destination(&config, artifact)?;
            if ModelManager::verify_file(&destination, &artifact.sha256).await? {
                println!(
                    "{} is already installed and verified at {}",
                    artifact.id,
                    destination.display()
                );
                return Ok(());
            }
            if destination.exists() {
                tokio::fs::remove_file(&destination).await?;
            }
            println!(
                "Downloading {} ({} bytes) to {}",
                artifact.id,
                artifact.size_bytes,
                destination.display()
            );
            let mut last_percent = 0_u64;
            ModelManager::default()
                .download_verified_with(
                    &artifact.url,
                    &destination,
                    &artifact.sha256,
                    CancellationToken::new(),
                    move |progress| print_download_progress(progress, &mut last_percent),
                )
                .await?;
            println!("Verified model saved to {}", destination.display());
        }
        ModelCommand::Status { id } => {
            let manifest = ModelManifest::bundled()?;
            let artifact = resolve_model(&manifest, &id)?;
            let destination = model_destination(&config, artifact)?;
            if !destination.is_file() {
                println!("{}: not installed ({})", artifact.id, destination.display());
            } else if ModelManager::verify_file(&destination, &artifact.sha256).await? {
                println!("{}: installed and verified", artifact.id);
            } else {
                println!(
                    "{}: installed but checksum verification failed",
                    artifact.id
                );
            }
        }
        ModelCommand::Remove { id } => {
            let manifest = ModelManifest::bundled()?;
            let artifact = resolve_model(&manifest, &id)?;
            let destination = model_destination(&config, artifact)?;
            match tokio::fs::remove_file(&destination).await {
                Ok(()) => println!("Removed {} from {}", artifact.id, destination.display()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    println!("{} is not installed", artifact.id);
                }
                Err(error) => return Err(error.into()),
            }
        }
        ModelCommand::Download {
            url,
            sha256,
            destination,
        } => {
            let destination = destination.unwrap_or(config.whisper.model);
            ModelManager::default()
                .download_verified(&url, &destination, &sha256)
                .await?;
            println!("Verified model saved to {}", destination.display());
        }
    }
    Ok(())
}

fn resolve_model<'a>(manifest: &'a ModelManifest, requested: &str) -> Result<&'a ModelArtifact> {
    let id = match requested {
        "default" | "fast" => "whisper-tiny",
        "balanced" => "whisper-tiny",
        "accurate" | "base" => "whisper-base",
        other => other,
    };
    manifest
        .find(id, std::env::consts::OS)
        .ok_or_else(|| anyhow!("reviewed model not found for this platform: {id}"))
}

fn model_destination(config: &AppConfig, artifact: &ModelArtifact) -> Result<PathBuf> {
    if artifact.id == "whisper-tiny" || artifact.id == "whisper-base" {
        return Ok(config.whisper.model.clone());
    }
    let filename = artifact
        .url
        .rsplit('/')
        .next()
        .filter(|filename| !filename.is_empty())
        .ok_or_else(|| anyhow!("model URL has no filename: {}", artifact.url))?;
    let configured = match artifact.provider.as_str() {
        "whisper.cpp" => &config.whisper.model,
        "vosk" => &config.vosk.model,
        "parakeet" => &config.parakeet.model,
        provider => return Err(anyhow!("unsupported model provider: {provider}")),
    };
    Ok(configured
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(filename))
}

fn print_download_progress(progress: DownloadProgress, last_percent: &mut u64) {
    let Some(total) = progress.total_bytes.filter(|total| *total > 0) else {
        return;
    };
    let percent = progress.downloaded_bytes.saturating_mul(100) / total;
    if percent == 100 || percent >= last_percent.saturating_add(5) {
        println!(
            "{percent:>3}% ({}/{total} bytes){}",
            progress.downloaded_bytes,
            if progress.resumed { " resumed" } else { "" }
        );
        *last_percent = percent;
    }
}

pub(crate) fn update() {
    println!("TermVox {}", env!("CARGO_PKG_VERSION"));
    println!("Check verified releases at:");
    println!("https://github.com/Jeronimo0228/termvox/releases");
    println!("TermVox will not replace its own executable.");
}

pub(crate) fn completions(shell: Shell) {
    clap_complete::generate(shell, &mut Cli::command(), "termvox", &mut io::stdout());
}

pub(crate) fn manpage(output: Option<&Path>) -> Result<()> {
    let man = clap_mangen::Man::new(Cli::command());
    if let Some(path) = output {
        let mut file = std::fs::File::create(path)?;
        man.render(&mut file)?;
        println!("Wrote {}", path.display());
    } else {
        man.render(&mut io::stdout())?;
    }
    Ok(())
}

fn configured_plugin<'a>(
    config: &'a AppConfig,
    id: &str,
) -> Result<&'a termvox_core::PluginConfig> {
    config
        .plugins
        .iter()
        .find(|plugin| plugin.id == id && plugin.enabled)
        .ok_or_else(|| anyhow::anyhow!("enabled plugin not found: {id}"))
}

async fn spawn_plugin(plugin: &termvox_core::PluginConfig) -> Result<PluginClient> {
    Ok(PluginClient::spawn_with(
        &plugin.executable,
        &plugin.args,
        PluginSpawnOptions {
            cwd: dirs::data_local_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("termvox/plugins")
                .join(&plugin.id),
            env_allowlist: plugin.env_allowlist.clone(),
            timeout: Duration::from_secs(plugin.timeout_seconds),
            max_frame_bytes: plugin.max_frame_bytes,
        },
    )
    .await?)
}
