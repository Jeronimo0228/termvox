use crate::PerformanceProfile;

/// Host environment hints used to tune defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentHints {
    pub wayland: bool,
    pub total_ram_mb: Option<u64>,
    pub low_ram: bool,
    pub suggest_toggle: bool,
    pub suggest_profile: PerformanceProfile,
}

#[must_use]
pub fn detect_environment() -> EnvironmentHints {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let total_ram_mb = total_system_ram_mb();
    let low_ram = total_ram_mb.is_some_and(|mb| mb < 8_192);
    EnvironmentHints {
        suggest_toggle: wayland,
        suggest_profile: if low_ram {
            PerformanceProfile::Fast
        } else {
            PerformanceProfile::Balanced
        },
        wayland,
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
    #[cfg(not(target_os = "linux"))]
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
}
