//! Voice hotkey detection and PTY key forwarding for agent shell mode.

use std::io::Write;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Returns true when the key event matches the configured shell voice hotkey.
#[must_use]
pub fn is_voice_hotkey(event: &KeyEvent, specification: &str) -> bool {
    if event.kind != KeyEventKind::Press {
        return false;
    }
    let mut modifiers = KeyModifiers::empty();
    let mut code = None;
    for token in specification.split('+').map(str::trim) {
        match token.to_ascii_uppercase().as_str() {
            "ALT" => modifiers |= KeyModifiers::ALT,
            "CTRL" | "CONTROL" => modifiers |= KeyModifiers::CONTROL,
            "SHIFT" => modifiers |= KeyModifiers::SHIFT,
            "SUPER" | "META" | "CMD" => modifiers |= KeyModifiers::SUPER,
            "SPACE" => code = Some(KeyCode::Char(' ')),
            "F8" => code = Some(KeyCode::F(8)),
            "F9" => code = Some(KeyCode::F(9)),
            "F10" => code = Some(KeyCode::F(10)),
            "F11" => code = Some(KeyCode::F(11)),
            "F12" => code = Some(KeyCode::F(12)),
            _ => {}
        }
    }
    let Some(expected) = code else {
        return false;
    };
    event.code == expected && event.modifiers == modifiers
}

/// Forwards a terminal key event to the agent PTY as bytes.
///
/// Returns `Ok(true)` when the event was consumed (not forwarded).
pub fn forward_key(event: KeyEvent, writer: &mut impl Write) -> std::io::Result<bool> {
    match event {
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            kind: KeyEventKind::Press,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) => Ok(true),
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Char(c),
            ..
        } => {
            writer.write_all(c.encode_utf8(&mut [0; 4]).as_bytes())?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Enter,
            ..
        } => {
            writer.write_all(b"\r")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Backspace,
            ..
        } => {
            writer.write_all(b"\x7f")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Tab,
            ..
        } => {
            writer.write_all(b"\t")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Esc,
            ..
        } => {
            writer.write_all(b"\x1b")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Up,
            ..
        } => {
            writer.write_all(b"\x1b[A")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Down,
            ..
        } => {
            writer.write_all(b"\x1b[B")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Right,
            ..
        } => {
            writer.write_all(b"\x1b[C")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Left,
            ..
        } => {
            writer.write_all(b"\x1b[D")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Home,
            ..
        } => {
            writer.write_all(b"\x1b[H")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::End,
            ..
        } => {
            writer.write_all(b"\x1b[F")?;
            Ok(false)
        }
        KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Delete,
            ..
        } => {
            writer.write_all(b"\x1b[3~")?;
            Ok(false)
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_f8_hotkey() {
        let event = KeyEvent::new(KeyCode::F(8), KeyModifiers::NONE);
        assert!(is_voice_hotkey(&event, "F8"));
        assert!(!is_voice_hotkey(&event, "F9"));
    }
}
