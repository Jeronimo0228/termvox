use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::{bench, commands, doctor, runtime, setup};
#[cfg(unix)]
use crate::daemon;

#[derive(Debug, Parser)]
#[command(
    name = "termvox",
    version,
    about = "Universal voice bridge for coding agents"
)]
pub(crate) struct Cli {
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {
        #[arg(long)]
        global: bool,
        #[arg(long)]
        force: bool,
        #[arg(long, value_name = "PRESET")]
        preset: Option<String>,
    },
    Setup {
        #[arg(long)]
        global: bool,
        #[arg(long)]
        force: bool,
        #[arg(long, value_name = "PRESET")]
        preset: Option<String>,
    },
    Start {
        #[arg(long)]
        toggle: bool,
        #[arg(long, value_name = "SHORTCUT")]
        global_hotkey: Option<String>,
    },
    #[cfg(unix)]
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    #[cfg(unix)]
    Talk,
    Bench {
        #[arg(long, default_value_t = 5)]
        runs: u32,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
    Plugins {
        #[command(subcommand)]
        command: Option<PluginCommand>,
    },
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommand>,
    },
    Test {
        #[arg(long, default_value_t = 3)]
        seconds: u64,
    },
    Update,
    Record {
        #[arg(value_enum)]
        action: RecordAction,
    },
    Models {
        #[command(subcommand)]
        command: ModelCommand,
    },
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    Manpage {
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[cfg(unix)]
#[derive(Debug, Subcommand)]
enum DaemonCommand {
    Start {
        #[arg(long)]
        background: bool,
    },
    Stop,
    Status,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum RecordAction {
    Start,
    Stop,
    Toggle,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigCommand {
    Path,
    Show,
    Validate,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ModelCommand {
    List,
    Install {
        #[arg(default_value = "default")]
        id: String,
    },
    Status {
        #[arg(default_value = "default")]
        id: String,
    },
    Remove {
        #[arg(default_value = "default")]
        id: String,
    },
    Download {
        url: String,
        #[arg(long)]
        sha256: String,
        #[arg(long)]
        destination: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum PluginCommand {
    List,
    Inspect { id: String },
    Test { id: String },
}

pub(crate) async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init {
            global,
            force,
            preset,
        } => setup::init_config(global, force, false, preset.as_deref())?,
        Commands::Setup {
            global,
            force,
            preset,
        } => setup::init_config(global, force, true, preset.as_deref())?,
        Commands::Start {
            toggle,
            global_hotkey,
        } => {
            runtime::start(
                setup::load_config(cli.config.as_deref())?,
                toggle,
                global_hotkey.as_deref(),
            )
            .await?;
        }
        #[cfg(unix)]
        Commands::Daemon { command } => match command {
            DaemonCommand::Start { background } => {
                daemon::run(
                    daemon::DaemonAction::Start,
                    setup::load_config(cli.config.as_deref())?,
                    background,
                )
                .await?;
            }
            DaemonCommand::Stop => {
                daemon::run(
                    daemon::DaemonAction::Stop,
                    setup::load_config(cli.config.as_deref())?,
                    false,
                )
                .await?
            }
            DaemonCommand::Status => {
                daemon::run(
                    daemon::DaemonAction::Status,
                    setup::load_config(cli.config.as_deref())?,
                    false,
                )
                .await?
            }
        },
        #[cfg(unix)]
        Commands::Talk => {
            daemon::run(
                daemon::DaemonAction::Talk,
                setup::load_config(cli.config.as_deref())?,
                false,
            )
            .await?;
        }
        Commands::Bench { runs } => {
            bench::run(setup::load_config(cli.config.as_deref())?, runs).await?;
        }
        Commands::Doctor { json } => {
            doctor::run(setup::load_config(cli.config.as_deref())?, json).await?;
        }
        Commands::Plugins { command } => {
            commands::plugins(setup::load_config(cli.config.as_deref())?, command).await?;
        }
        Commands::Config { command } => commands::config(command, cli.config.as_deref())?,
        Commands::Test { seconds } => {
            runtime::test_audio(setup::load_config(cli.config.as_deref())?, seconds).await?;
        }
        Commands::Update => commands::update(),
        Commands::Record { action } => {
            runtime::record(setup::load_config(cli.config.as_deref())?, action).await?;
        }
        Commands::Models { command } => {
            commands::model(setup::load_config(cli.config.as_deref())?, command).await?;
        }
        Commands::Completions { shell } => commands::completions(shell),
        Commands::Manpage { output } => commands::manpage(output.as_deref())?,
    }
    Ok(())
}
