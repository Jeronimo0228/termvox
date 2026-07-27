use std::{
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Result, bail};
use termvox_core::AppConfig;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::windows::named_pipe::{ClientOptions, ServerOptions},
};

use crate::{runtime, setup};

use super::{IpcRequest, IpcResponse, dispatch_request};

const PIPE_NAME: &str = r"\\.\pipe\termvox-daemon";

pub(super) async fn start(config: AppConfig, background: bool) -> Result<()> {
    if pid_path().exists() {
        bail!("TermVox daemon already running (pid file exists); run `termvox daemon stop`");
    }
    if background {
        spawn_background_daemon()?;
        println!("TermVox daemon started in the background");
        return Ok(());
    }
    write_pid_file()?;
    let result = run_daemon(config).await;
    let _ = std::fs::remove_file(pid_path());
    result
}

async fn run_daemon(config: AppConfig) -> Result<()> {
    let toggle_trigger = Arc::new(AtomicBool::new(false));
    let toggle_ipc = Arc::clone(&toggle_trigger);
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_accept = Arc::clone(&shutdown);
    tokio::spawn(async move {
        let mut server = match ServerOptions::new()
            .first_pipe_instance(true)
            .create(PIPE_NAME)
        {
            Ok(server) => server,
            Err(error) => {
                tracing::warn!("daemon pipe server failed: {error}");
                return;
            }
        };
        loop {
            if shutdown_accept.load(Ordering::Acquire) {
                break;
            }
            if server.connect().await.is_err() {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                continue;
            }
            let connected = server;
            server = match ServerOptions::new().create(PIPE_NAME) {
                Ok(server) => server,
                Err(error) => {
                    tracing::warn!("daemon pipe recreate failed: {error}");
                    break;
                }
            };
            let toggle = Arc::clone(&toggle_ipc);
            tokio::spawn(async move {
                let _ = handle_client(connected, toggle).await;
            });
        }
    });
    let mut daemon_config = config;
    if daemon_config.daemon.skip_confirmation {
        daemon_config.auto_send = true;
        daemon_config.confirmation = false;
    }
    let hotkey = daemon_config.daemon.hotkey.clone();
    println!("TermVox daemon ready on {PIPE_NAME} (hotkey {hotkey})");
    runtime::start_daemon_session(daemon_config, &hotkey, toggle_trigger).await?;
    shutdown.store(true, Ordering::Release);
    Ok(())
}

pub(super) fn paths() -> (PathBuf, PathBuf) {
    (pid_path(), PathBuf::from(PIPE_NAME))
}

async fn handle_client(
    mut stream: tokio::net::windows::named_pipe::NamedPipeServer,
    toggle: Arc<AtomicBool>,
) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut lines = BufReader::new(reader).lines();
    let Some(line) = lines.next_line().await? else {
        return Ok(());
    };
    let request: IpcRequest = serde_json::from_str(&line).context("invalid daemon request")?;
    let response = dispatch_request(&request.cmd, &toggle);
    writer
        .write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes())
        .await?;
    Ok(())
}

pub(super) async fn talk() -> Result<()> {
    if !pid_path().exists() {
        bail!("TermVox daemon is not running; start it with `termvox daemon start --background`");
    }
    let mut client = ClientOptions::new()
        .open(PIPE_NAME)
        .context("failed to connect to TermVox daemon pipe")?;
    client
        .write_all(b"{\"cmd\":\"toggle\"}\n")
        .await
        .context("failed to send talk request")?;
    let mut reader = BufReader::new(client);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let response: IpcResponse = serde_json::from_str(line.trim())?;
    if response.ok {
        println!("{}", response.message);
        Ok(())
    } else {
        bail!(response.message)
    }
}

pub(super) fn stop() -> Result<()> {
    let pid_file = pid_path();
    if !pid_file.exists() {
        bail!("TermVox daemon is not running");
    }
    let pid = std::fs::read_to_string(&pid_file)
        .context("read daemon pid")?
        .trim()
        .to_owned();
    Command::new("taskkill")
        .args(["/PID", &pid, "/T", "/F"])
        .status()
        .context("terminate daemon process")?;
    let _ = std::fs::remove_file(pid_file);
    println!("TermVox daemon stopped");
    Ok(())
}

pub(super) fn status() -> Result<()> {
    if pid_path().exists() {
        println!(
            "running (pid {}, pipe {PIPE_NAME})",
            std::fs::read_to_string(pid_path())?.trim(),
        );
    } else {
        println!("not running");
    }
    Ok(())
}

fn spawn_background_daemon() -> Result<()> {
    let exe = std::env::current_exe()?;
    let mut command = Command::new(exe);
    command
        .arg("daemon")
        .arg("start")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let project = PathBuf::from("termvox.toml");
    if project.is_file() {
        command.arg("--config").arg(project);
    } else {
        let global = setup::global_config_path();
        if global.is_file() {
            command.arg("--config").arg(global);
        }
    }
    command.spawn()?;
    Ok(())
}

fn write_pid_file() -> Result<()> {
    if let Some(parent) = pid_path().parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(pid_path(), std::process::id().to_string())?;
    Ok(())
}

fn pid_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("termvox-daemon.pid")
}
