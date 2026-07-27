use crate::PerformanceProfile;

/// Host environment hints used to tune defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentHints {
    pub wayland: bool,
    pub windows: bool,
    pub total_ram_mb: Option<u64>,
    pub low_ram: bool,
    pub suggest_toggle: bool,
    pub suggest_profile: PerformanceProfile,
}

#[must_use]
pub fn detect_environment() -> EnvironmentHints {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let windows = cfg!(target_os = "windows");
    let total_ram_mb = total_system_ram_mb();
    let low_ram = total_ram_mb.is_some_and(|mb| mb < 8_192);
    EnvironmentHints {
        suggest_toggle: wayland || windows,
        suggest_profile: if low_ram {
            PerformanceProfile::Fast
        } else {
            PerformanceProfile::Balanced
        },
        wayland,
        windows,
        total_ram_mb,
        low_ram,
    }
}

fn total_system_ram_mb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        for line in text.lines() {
            if let Some(kb) = line.strip_prefix("MemTotal:") {
                let kb = kb.trim().trim_end_matches(" kB").parse::<u64>().ok()?;
                return Some(kb / 1024);
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let bytes = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u64>()
            .ok()?;
        Some(bytes / (1024 * 1024))
    }
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
            ])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let bytes = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u64>()
            .ok()?;
        Some(bytes / (1024 * 1024))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_wayland_flag_from_env() {
        let hints = detect_environment();
        assert_eq!(hints.wayland, std::env::var_os("WAYLAND_DISPLAY").is_some());
    }

    #[test]
    fn linux_reads_memtotal_when_available() {
        if std::path::Path::new("/proc/meminfo").is_file() {
            assert!(total_system_ram_mb().is_some());
        }
    }
}
