use std::{process::Command, time::Duration};

use anyhow::{Context, Result, bail};

/// Sends Ctrl+V to the focused window (best-effort, platform-specific).
pub(crate) fn paste_focused() -> Result<()> {
    paste_with_backends(paste_backends())
}

pub(crate) fn paste_to_target(window_title: Option<&str>) -> Result<()> {
    if let Some(title) = window_title {
        focus_window_title(title)?;
    }
    paste_focused()
}

pub(crate) fn focus_window_title(substring: &str) -> Result<()> {
    for (program, args) in focus_backends(substring) {
        if !command_exists(program) {
            continue;
        }
        let status = Command::new(program)
            .args(&args)
            .status()
            .with_context(|| format!("failed to run {program}"))?;
        if status.success() {
            std::thread::sleep(Duration::from_millis(120));
            return Ok(());
        }
        tracing::warn!(program, "window focus backend exited with {status}");
    }
    bail!(
        "could not focus a window titled like '{substring}'; install wmctrl or xdotool, or set agents.<agent>.paste_window_title"
    );
}

pub(crate) fn paste_after_clipboard_delay() {
    std::thread::sleep(Duration::from_millis(120));
}

fn paste_with_backends(
    backends: [(&'static str, &'static [&'static str], &'static str); 3],
) -> Result<()> {
    for (program, args, _description) in backends {
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

fn focus_backends(substring: &str) -> Vec<(&'static str, Vec<String>)> {
    vec![
        ("wmctrl", vec!["-a".into(), substring.into()]),
        (
            "xdotool",
            vec![
                "search".into(),
                "--name".into(),
                substring.into(),
                "windowactivate".into(),
            ],
        ),
        (
            "xdotool",
            vec![
                "search".into(),
                "--onlyvisible".into(),
                "--name".into(),
                substring.into(),
                "windowactivate".into(),
            ],
        ),
    ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_backends_include_wmctrl_and_xdotool() {
        let backends = focus_backends("Cursor");
        assert_eq!(backends.len(), 3);
        assert_eq!(backends[0].0, "wmctrl");
        assert_eq!(backends[0].1[1], "Cursor");
    }
}
