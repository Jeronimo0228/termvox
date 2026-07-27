use std::{process::Command, time::Duration};

use anyhow::{Context, Result, bail};

/// Sends Ctrl+V to the focused window (best-effort, platform-specific).
pub(crate) fn paste_focused() -> Result<()> {
    for (program, args, _description) in paste_backends() {
        if !command_exists(program) {
            continue;
        }
        let status = Command::new(program)
            .args(args)
            .status()
            .with_context(|| format!("failed to run {program}"))?;
        if status.success() {
            return Ok(());
        }
        tracing::warn!(program, "paste backend exited with {status}");
    }
    bail!("auto-paste unavailable; install wtype (Wayland), ydotool, or xdotool (X11)");
}

pub(crate) fn paste_after_clipboard_delay() {
    std::thread::sleep(Duration::from_millis(120));
}

fn command_exists(program: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(program);
            if candidate.is_file() {
                return true;
            }
        }
    }
    false
}

fn paste_backends() -> [(&'static str, &'static [&'static str], &'static str); 3] {
    [
        (
            "wtype",
            &["-M", "ctrl", "-P", "v", "-m", "ctrl", "-p", "v"],
            "Wayland",
        ),
        (
            "ydotool",
            &["key", "29:1", "47:1", "47:0", "29:0"],
            "uinput",
        ),
        ("xdotool", &["key", "ctrl+v"], "X11"),
    ]
}
