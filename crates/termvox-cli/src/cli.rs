use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::{bench, commands, daemon, doctor, runtime, setup, shell, shim};

#[derive(Debug, Parser)]
#[command(
    name = "termvox",
    version,
    about = "Universal voice bridge for coding agents",
    long_about = "TermVox adds local speech-to-text (Whisper by default) to coding-agent CLIs.\n\n\
Recommended entry point: `termvox shell` (mic bar inside the agent TUI).\n\n\
STT quality: default profile is `fast` (ggml-tiny). For better accuracy use\n\
`performance_profile = \"balanced\"` and `termvox models install accurate`.\n\
Docs: https://github.com/Jeronimo0228/termvox/blob/main/docs/performance.md\n\
Español: https://github.com/Jeronimo0228/termvox/blob/main/docs/es/stt.md",
    after_help = "Tips:\n  termvox doctor                 Check mic, Whisper model, and agent auth\n  \
termvox models install accurate  Install ggml-base (~142 MiB) for better STT\n  \
termvox config path              Show global and project config files\n  \
termvox shell --agent opencode   Integrated shell + voice (F8 / Ctrl+Space)"
)]
pub(crate) struct Cli {
    /// Path to a termvox.toml (overrides project/global merge for this run)
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a termvox.toml (project or --global)
    #[command(long_about = "Write a starter termvox.toml.\n\n\
Use --global for ~/.config/termvox/termvox.toml (Linux).\n\
Use --preset cursor|opencode|claude|... for agent-oriented defaults.\n\
After init, set language and performance_profile for STT quality — see docs/es/stt.md.")]
    Init {
        /// Write the global config instead of ./termvox.toml
        #[arg(long)]
        global: bool,
        /// Overwrite an existing file
        #[arg(long)]
        force: bool,
        /// Agent preset: cursor, opencode, claude, codex, gemini, aider, amp
        #[arg(long, value_name = "PRESET")]
        preset: Option<String>,
    },
    /// Interactive setup wizard (config + prompts)
    Setup {
        /// Write the global config instead of ./termvox.toml
        #[arg(long)]
        global: bool,
        /// Overwrite an existing file
        #[arg(long)]
        force: bool,
        /// Agent preset: cursor, opencode, claude, codex, gemini, aider, amp
        #[arg(long, value_name = "PRESET")]
        preset: Option<String>,
    },
    /// Start a voice session (branded / companion / or shell when display=shell)
    #[command(long_about = "Run TermVox with the configured agent display mode.\n\n\
When agents.<name>.display = \"shell\", this delegates to `termvox shell`.\n\
Otherwise starts the push-to-talk UI (Space by default).")]
    Start {
        /// Toggle recording mode instead of hold-to-talk
        #[arg(long)]
        toggle: bool,
        /// Optional global hotkey shortcut for this session
        #[arg(long, value_name = "SHORTCUT")]
        global_hotkey: Option<String>,
    },
    /// Background voice daemon (Unix; global hotkey)
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    /// Toggle recording on a running daemon
    Talk,
    /// Benchmark Whisper latency (JSON report)
    Bench {
        /// Number of timed runs
        #[arg(long, default_value_t = 5)]
        runs: u32,
    },
    /// Diagnose mic, speech engine, config, and agent CLIs
    #[command(
        long_about = "Print readiness checks for microphone, Whisper/model path,\n\
configuration, and each supported agent (installed + auth).\n\n\
Use --json for scripting. See also: termvox config show"
    )]
    Doctor {
        /// Emit machine-readable JSON (includes hints)
        #[arg(long)]
        json: bool,
    },
    /// List / inspect / test JSON-RPC plugins
    Plugins {
        #[command(subcommand)]
        command: Option<PluginCommand>,
    },
    /// Show config paths, merged config, or validate
    #[command(long_about = "Inspect TermVox configuration.\n\n\
  path      Print global and project termvox.toml paths\n\
  show      Print the merged effective configuration\n\
  validate  Parse and validate the merged config\n\n\
STT keys: performance_profile, language, [whisper], [audio].\n\
Guide: docs/performance.md · docs/es/stt.md")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommand>,
    },
    /// Record from the mic and print a transcript (no agent)
    #[command(long_about = "Smoke-test microphone + speech engine.\n\n\
Speak during the countdown; TermVox prints the transcript.\n\
Useful after changing performance_profile or the Whisper model.")]
    Test {
        /// Seconds to record
        #[arg(long, default_value_t = 3)]
        seconds: u64,
    },
    /// Print version and where to get updates (does not auto-install)
    Update,
    /// Control an in-progress recording session
    Record {
        #[arg(value_enum)]
        action: RecordAction,
    },
    /// Install and manage Whisper (and related) model artifacts
    #[command(long_about = "Manage reviewed speech model downloads.\n\n\
  default   → ggml-tiny.bin  (~74 MiB)  — performance_profile = fast\n\
  accurate  → ggml-base.bin (~142 MiB)  — balanced / accurate (better STT)\n\n\
After installing accurate, set in termvox.toml:\n\
  performance_profile = \"balanced\"\n\
  language = \"es\"   # or your language\n\
  [whisper]\n\
  model = \"/path/from/termvox models status accurate\"\n\n\
Docs: docs/performance.md · docs/es/stt.md")]
    Models {
        #[command(subcommand)]
        command: ModelCommand,
    },
    /// Generate shell completions (bash, zsh, fish, powershell, elvish)
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Generate the termvox(1) man page (from this CLI definition)
    Manpage {
        /// Write to a file instead of stdout
        #[arg(long, value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Integrated agent TUI + `TermVox` mic bar (recommended)
    #[command(
        long_about = "Launch a coding-agent CLI inside a PTY with a persistent mic bar.\n\n\
Voice: F8 or Ctrl+Space (Wayland). Exit wrapper: Ctrl+\\ (Ctrl+C goes to the agent).\n\
--fresh starts without resuming a saved workspace session.\n\n\
STT uses your termvox.toml speech settings. For better accuracy:\n\
  termvox models install accurate\n\
  performance_profile = \"balanced\"\n\
See docs/agent-shell.md and docs/es/stt.md."
    )]
    Shell {
        /// Agent override: cursor, opencode, claude, codex, gemini, aider, amp
        #[arg(long, value_name = "AGENT")]
        agent: Option<String>,
        /// Ignore saved .termvox/session.json and skip session discovery
        #[arg(long)]
        fresh: bool,
        /// Extra args forwarded to the upstream agent after `--`
        #[arg(last = true, allow_hyphen_values = true)]
        agent_args: Vec<String>,
    },
    /// Install `~/.local/bin/<agent>` wrapper that runs termvox shell (Unix)
    InstallShim {
        /// Agent to wrap (default: configured agent)
        #[arg(long, value_name = "AGENT")]
        agent: Option<String>,
        /// Replace an existing shim
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
enum DaemonCommand {
    /// Start the daemon (optionally in the background)
    Start {
        #[arg(long)]
        background: bool,
    },
    /// Stop the running daemon
    Stop,
    /// Show daemon status
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
    /// Print global and project config file paths
    Path,
    /// Print the merged effective configuration
    Show,
    /// Validate the merged configuration
    Validate,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ModelCommand {
    /// List reviewed model artifacts from the bundled manifest
    List,
    /// Download and verify a model (`default` = tiny, `accurate` = base)
    Install {
        /// Artifact id: default | accurate | …
        #[arg(default_value = "default")]
        id: String,
    },
    /// Show install path and checksum status
    Status {
        #[arg(default_value = "default")]
        id: String,
    },
    /// Remove an installed model artifact
    Remove {
        #[arg(default_value = "default")]
        id: String,
    },
    /// Download a custom URL with required SHA-256 verification
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
    /// List configured plugins
    List,
    /// Inspect one enabled plugin by id
    Inspect { id: String },
    /// Run the plugin test handshake
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
        Commands::Shell {
            agent,
            fresh,
            agent_args,
        } => {
            let config = setup::load_config(cli.config.as_deref())?;
            let kind = match agent {
                Some(value) => shim::parse_agent_kind(&value)?,
                None => config.agent,
            };
            shell::run(config, kind, agent_args, fresh).await?;
        }
        Commands::InstallShim { agent, force } => {
            let config = setup::load_config(cli.config.as_deref())?;
            let kind = match agent {
                Some(value) => shim::parse_agent_kind(&value)?,
                None => config.agent,
            };
            shim::install(kind, force)?;
        }
    }
    Ok(())
}
