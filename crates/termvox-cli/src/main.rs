#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::collapsible_if,
    clippy::empty_line_after_outer_attr,
    clippy::needless_pass_by_ref_mut,
    clippy::semicolon_if_nothing_returned,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unused_self,
    dead_code
)]
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
mod shell;
mod shim;
mod telemetry;
mod ui;
mod workspace;

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
