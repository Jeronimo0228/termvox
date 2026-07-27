use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub(crate) enum DaemonAction {
    Start,
    Stop,
    Status,
    Talk,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct IpcRequest {
    pub cmd: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct IpcResponse {
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recording: Option<bool>,
}

pub(super) fn dispatch_request(cmd: &str, toggle: &AtomicBool) -> IpcResponse {
    match cmd {
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
    }
}

pub(crate) fn take_toggle(trigger: &AtomicBool) -> bool {
    trigger.swap(false, Ordering::AcqRel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn dispatch_toggle_sets_flag() {
        let toggle = AtomicBool::new(false);
        let response = dispatch_request("talk", &toggle);
        assert!(response.ok);
        assert!(toggle.load(Ordering::Relaxed));
    }
}

pub(crate) async fn run(
    action: DaemonAction,
    config: termvox_core::AppConfig,
    background: bool,
) -> anyhow::Result<()> {
    match action {
        DaemonAction::Start => platform::start(config, background).await,
        DaemonAction::Stop => platform::stop(),
        DaemonAction::Status => platform::status(),
        DaemonAction::Talk => platform::talk().await,
    }
}

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix as platform;
#[cfg(windows)]
use windows as platform;
