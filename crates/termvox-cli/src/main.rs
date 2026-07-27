mod bench;
mod cli;
mod clipboard;
mod commands;
mod daemon;
mod delivery;
mod doctor;
mod paste;
mod presets;
mod runtime;
mod session_ui;
mod setup;
mod telemetry;
mod ui;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();
    cli::run().await
}
