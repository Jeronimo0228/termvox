use std::{
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use termvox_core::AppConfig;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};

use crate::{runtime, setup};

#[derive(Debug, Serialize, Deserialize)]
struct IpcRequest {
    cmd: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IpcResponse {
    ok: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    recording: Option<bool>,
}

pub(crate) async fn run(action: DaemonAction, config: AppConfig, background: bool) -> Result<()> {
    match action {
        DaemonAction::Start => start(config, background).await,
        DaemonAction::Stop => stop(),
        DaemonAction::Status => status(),
        DaemonAction::Talk => talk().await,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DaemonAction {
    Start,
    Stop,
    Status,
    Talk,
}

async fn start(config: AppConfig, background: bool) -> Result<()> {
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
    let _ = std::fs::remove_file(socket_path());
    result
}

async fn run_daemon(config: AppConfig) -> Result<()> {
    let toggle_trigger = Arc::new(AtomicBool::new(false));
    let toggle_ipc = Arc::clone(&toggle_trigger);
    let listener = bind_socket()?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_accept = Arc::clone(&shutdown);
    tokio::spawn(async move {
        loop {
            if shutdown_accept.load(Ordering::Acquire) {
                break;
            }
            let accept = listener.accept().await;
            let Ok((stream, _)) = accept else {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                continue;
            };
            let toggle = Arc::clone(&toggle_ipc);
            tokio::spawn(async move {
                let _ = handle_client(stream, toggle).await;
            });
        }
    });
    let mut daemon_config = config;
    if daemon_config.daemon.skip_confirmation {
        daemon_config.auto_send = true;
        daemon_config.confirmation = false;
    }
    let hotkey = daemon_config.daemon.hotkey.clone();
    println!(
        "TermVox daemon ready on {} (hotkey {hotkey})",
        socket_path().display()
    );
    runtime::start_daemon_session(daemon_config, &hotkey, toggle_trigger).await?;
    shutdown.store(true, Ordering::Release);
    Ok(())
}

pub(crate) fn paths() -> (PathBuf, PathBuf) {
    (pid_path(), socket_path())
}

async fn handle_client(stream: UnixStream, toggle: Arc<AtomicBool>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    let Some(line) = lines.next_line().await? else {
        return Ok(());
    };
    let request: IpcRequest = serde_json::from_str(&line).context("invalid daemon request")?;
    let response = match request.cmd.as_str() {
        "toggle" | "talk" => {
            toggle.store(true, Ordering::Release);
            IpcResponse {
                ok: true,
                message: "toggle queued".into(),
                recording: None,
            }
        }
        "status" => IpcResponse {
            ok: true,
            message: "running".into(),
            recording: None,
        },
        other => IpcResponse {
            ok: false,
            message: format!("unknown command: {other}"),
            recording: None,
        },
    };
    writer
        .write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes())
        .await?;
    Ok(())
}

async fn talk() -> Result<()> {
    if !socket_path().exists() {
        bail!("TermVox daemon is not running; start it with `termvox daemon start --background`");
    }
    let mut stream = UnixStream::connect(&socket_path()).await?;
    stream
        .write_all(b"{\"cmd\":\"toggle\"}\n")
        .await
        .context("failed to send talk request")?;
    let mut reader = BufReader::new(stream);
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

fn stop() -> Result<()> {
    let pid_file = pid_path();
    if !pid_file.exists() {
        bail!("TermVox daemon is not running");
    }
    let pid = std::fs::read_to_string(&pid_file)
        .context("read daemon pid")?
        .trim()
        .to_owned();
    Command::new("kill")
        .arg(&pid)
        .status()
        .context("send SIGTERM to daemon")?;
    let _ = std::fs::remove_file(pid_file);
    let _ = std::fs::remove_file(socket_path());
    println!("TermVox daemon stopped");
    Ok(())
}

fn status() -> Result<()> {
    if pid_path().exists() {
        println!(
            "running (pid {}, socket {})",
            std::fs::read_to_string(pid_path())?.trim(),
            socket_path().display()
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

fn bind_socket() -> Result<UnixListener> {
    let path = socket_path();
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    UnixListener::bind(&path).with_context(|| format!("bind {}", path.display()))
}

fn write_pid_file() -> Result<()> {
    if let Some(parent) = pid_path().parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(pid_path(), std::process::id().to_string())?;
    Ok(())
}

fn pid_path() -> PathBuf {
    runtime_dir().join("termvox-daemon.pid")
}

fn socket_path() -> PathBuf {
    runtime_dir().join("termvox-daemon.sock")
}

fn runtime_dir() -> PathBuf {
    dirs::runtime_dir().unwrap_or_else(std::env::temp_dir)
}

pub(crate) fn take_toggle(trigger: &AtomicBool) -> bool {
    trigger.swap(false, Ordering::AcqRel)
}
