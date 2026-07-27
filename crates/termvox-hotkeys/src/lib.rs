//! Global-hotkey capability detection with safe terminal/external fallbacks.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyBackend {
    Windows,
    MacOs,
    X11,
    XdgPortal,
    TerminalFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeySupport {
    pub backend: HotkeyBackend,
    pub global_available: bool,
    pub key_release_available: bool,
    pub guidance: Option<String>,
}

#[derive(Debug)]
pub struct HotkeyError(String);

impl fmt::Display for HotkeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for HotkeyError {}

#[must_use]
pub fn detect_support() -> HotkeySupport {
    if cfg!(target_os = "windows") {
        return native_support(HotkeyBackend::Windows);
    }
    if cfg!(target_os = "macos") {
        return native_support(HotkeyBackend::MacOs);
    }
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return HotkeySupport {
            backend: HotkeyBackend::XdgPortal,
            global_available: false,
            key_release_available: false,
            guidance: Some(
                "Wayland requires an XDG GlobalShortcuts portal supported by the compositor; use toggle mode or `termvox record toggle` when unavailable".into(),
            ),
        };
    }
    if std::env::var_os("DISPLAY").is_some() {
        return native_support(HotkeyBackend::X11);
    }
    HotkeySupport {
        backend: HotkeyBackend::TerminalFallback,
        global_available: false,
        key_release_available: true,
        guidance: Some(
            "No graphical session detected; terminal push-to-talk and external record commands remain available".into(),
        ),
    }
}

fn native_support(backend: HotkeyBackend) -> HotkeySupport {
    HotkeySupport {
        backend,
        global_available: cfg!(feature = "global-hotkey"),
        key_release_available: cfg!(feature = "global-hotkey"),
        guidance: (!cfg!(feature = "global-hotkey")).then(|| {
            "This build omits global-hotkey support; install a full release or use terminal mode"
                .into()
        }),
    }
}

#[cfg(feature = "global-hotkey")]
mod native {
    use global_hotkey::{
        GlobalHotKeyEvent, GlobalHotKeyManager,
        hotkey::{Code, HotKey, Modifiers},
    };

    use super::{HotkeyError, TriggerState};

    pub struct HotkeyRegistration {
        _manager: GlobalHotKeyManager,
        id: u32,
    }

    impl HotkeyRegistration {
        /// Registers a system-wide shortcut.
        ///
        /// # Errors
        ///
        /// Returns an error when the shortcut is invalid, occupied, or the
        /// platform denies global-shortcut access.
        pub fn register(specification: &str) -> Result<Self, HotkeyError> {
            let hotkey = parse_hotkey(specification)?;
            let id = hotkey.id();
            let manager =
                GlobalHotKeyManager::new().map_err(|error| HotkeyError(error.to_string()))?;
            manager
                .register(hotkey)
                .map_err(|error| HotkeyError(error.to_string()))?;
            Ok(Self {
                _manager: manager,
                id,
            })
        }

        #[must_use]
        pub fn poll(&self) -> Option<TriggerState> {
            let event = GlobalHotKeyEvent::receiver().try_recv().ok()?;
            if event.id != self.id {
                return None;
            }
            Some(match event.state {
                global_hotkey::HotKeyState::Pressed => TriggerState::Pressed,
                global_hotkey::HotKeyState::Released => TriggerState::Released,
            })
        }
    }

    fn parse_hotkey(specification: &str) -> Result<HotKey, HotkeyError> {
        let mut modifiers = Modifiers::empty();
        let mut code = None;
        for token in specification.split('+').map(str::trim) {
            match token.to_ascii_uppercase().as_str() {
                "ALT" => modifiers |= Modifiers::ALT,
                "CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
                "SHIFT" => modifiers |= Modifiers::SHIFT,
                "SUPER" | "META" | "CMD" => modifiers |= Modifiers::SUPER,
                "SPACE" => code = Some(Code::Space),
                "F8" => code = Some(Code::F8),
                "F9" => code = Some(Code::F9),
                "F10" => code = Some(Code::F10),
                "F11" => code = Some(Code::F11),
                "F12" => code = Some(Code::F12),
                other => {
                    return Err(HotkeyError(format!(
                        "unsupported global hotkey key: {other}"
                    )));
                }
            }
        }
        let code = code.ok_or_else(|| HotkeyError("global hotkey has no key".into()))?;
        Ok(HotKey::new(
            (!modifiers.is_empty()).then_some(modifiers),
            code,
        ))
    }

    pub use HotkeyRegistration as Registration;
}

#[cfg(feature = "global-hotkey")]
pub use native::Registration as HotkeyRegistration;

#[cfg(not(feature = "global-hotkey"))]
pub struct HotkeyRegistration;

#[cfg(not(feature = "global-hotkey"))]
impl HotkeyRegistration {
    /// Reports that this build has no native global-hotkey backend.
    ///
    /// # Errors
    ///
    /// Always returns an error explaining that the feature is disabled.
    pub fn register(_specification: &str) -> Result<Self, HotkeyError> {
        Err(HotkeyError(
            "global-hotkey support is disabled in this build".into(),
        ))
    }

    #[must_use]
    pub const fn poll(&self) -> Option<TriggerState> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_environment_has_actionable_fallback() {
        if std::env::var_os("DISPLAY").is_none()
            && std::env::var_os("WAYLAND_DISPLAY").is_none()
            && !cfg!(any(target_os = "windows", target_os = "macos"))
        {
            let support = detect_support();
            assert_eq!(support.backend, HotkeyBackend::TerminalFallback);
            assert!(support.guidance.is_some());
        }
    }
}
