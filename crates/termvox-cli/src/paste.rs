use std::{process::Command, time::Duration};

use anyhow::{Context, Result, bail};

/// Sends Ctrl+V to the focused window (best-effort, platform-specific).
pub(crate) fn paste_focused() -> Result<()> {
    #[cfg(windows)]
    {
        return windows::paste_focused();
    }
    #[cfg(not(windows))]
    {
        paste_with_backends(paste_backends_unix())
    }
}

pub(crate) fn paste_to_target(window_title: Option<&str>) -> Result<()> {
    if let Some(title) = window_title {
        focus_window_title(title)?;
    }
    paste_focused()
}

pub(crate) fn focus_window_title(substring: &str) -> Result<()> {
    #[cfg(windows)]
    {
        return windows::focus_window_title(substring);
    }
    #[cfg(not(windows))]
    {
        focus_window_title_unix(substring)
    }
}

pub(crate) fn paste_after_clipboard_delay() {
    std::thread::sleep(Duration::from_millis(120));
}

#[cfg(not(windows))]
fn focus_window_title_unix(substring: &str) -> Result<()> {
    for (program, args) in focus_backends_unix(substring) {
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

#[cfg(not(windows))]
fn focus_backends_unix(substring: &str) -> Vec<(&'static str, Vec<String>)> {
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

#[cfg(not(windows))]
fn paste_backends_unix() -> [(&'static str, &'static [&'static str], &'static str); 3] {
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

#[cfg(windows)]
mod windows {
    use super::*;

    pub(super) fn focus_window_title(substring: &str) -> Result<()> {
        let escaped = substring.replace('\'', "''");
        let script = format!(
            "$shell = New-Object -ComObject WScript.Shell; if (-not $shell.AppActivate('{escaped}')) {{ exit 1 }}"
        );
        let status = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .context("failed to run PowerShell for window focus")?;
        if status.success() {
            std::thread::sleep(Duration::from_millis(120));
            return Ok(());
        }
        bail!(
            "could not focus a window titled like '{substring}'; adjust agents.<agent>.paste_window_title"
        );
    }

    pub(super) fn paste_focused() -> Result<()> {
        let script = "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('^v')";
        let status = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .status()
            .context("failed to run PowerShell for auto-paste")?;
        if status.success() {
            Ok(())
        } else {
            bail!("auto-paste failed; ensure PowerShell is available")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(windows))]
    fn focus_backends_include_wmctrl_and_xdotool() {
        let backends = focus_backends_unix("Cursor");
        assert_eq!(backends.len(), 3);
        assert_eq!(backends[0].0, "wmctrl");
        assert_eq!(backends[0].1[1], "Cursor");
    }
}
