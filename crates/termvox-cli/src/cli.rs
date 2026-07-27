use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::{commands, doctor, runtime, setup};

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
    },
    Setup {
        #[arg(long)]
        global: bool,
        #[arg(long)]
        force: bool,
    },
    Start {
        #[arg(long)]
        toggle: bool,
        #[arg(long, value_name = "SHORTCUT")]
        global_hotkey: Option<String>,
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
        Commands::Init { global, force } => setup::init_config(global, force, false)?,
        Commands::Setup { global, force } => setup::init_config(global, force, true)?,
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
