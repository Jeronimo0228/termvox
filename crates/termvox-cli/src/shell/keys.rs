//! Voice hotkey detection and PTY key forwarding for agent shell mode.

use std::io::Write;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Returns true when the key event matches a configured hotkey string (`F8`, `Ctrl+\\`, …).
#[must_use]
pub fn matches_hotkey(event: &KeyEvent, specification: &str) -> bool {
    if !matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return false;
    }
    let Some((expected_code, expected_modifiers)) = parse_hotkey(specification) else {
        return false;
    };
    if event.code != expected_code {
        return false;
    }
    if matches!(
        expected_code,
        KeyCode::F(_) | KeyCode::Esc | KeyCode::Enter | KeyCode::Tab | KeyCode::Backspace
    ) {
        return true;
    }
    if matches!(expected_code, KeyCode::Char(' ')) {
        return event.modifiers.contains(KeyModifiers::CONTROL)
            == expected_modifiers.contains(KeyModifiers::CONTROL);
    }
    event.modifiers == expected_modifiers
}

/// Returns true when the key event matches any configured shell voice hotkey.
#[must_use]
pub fn is_voice_hotkey(event: &KeyEvent, specifications: &[String]) -> bool {
    specifications
        .iter()
        .any(|spec| matches_hotkey(event, spec))
}

/// Returns true when the key event should leave the integrated shell wrapper.
#[must_use]
pub fn is_shell_exit(event: &KeyEvent, specification: &str) -> bool {
    if matches_hotkey(event, specification) {
        return true;
    }
    // Linux TTYs often deliver Ctrl+\ as ASCII FS (0x1c) without CONTROL set.
    if specification
        .replace(' ', "")
        .eq_ignore_ascii_case("Ctrl+\\")
        && matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat)
        && event.code == KeyCode::Char('\x1c')
    {
        return true;
    }
    false
}

/// Voice hotkeys used in shell mode, including Wayland-friendly fallbacks.
#[must_use]
pub fn shell_voice_hotkeys(config: &termvox_core::AppConfig) -> Vec<String> {
    let mut hotkeys = vec![config.shell.hotkey.clone()];
    for alt in &config.shell.alt_hotkeys {
        if !hotkeys
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(alt))
        {
            hotkeys.push(alt.clone());
        }
    }
    if termvox_core::detect_environment().wayland {
        for fallback in ["Ctrl+Space", "F9"] {
            if !hotkeys
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(fallback))
            {
                hotkeys.push(fallback.into());
            }
        }
    }
    hotkeys
}

fn parse_hotkey(specification: &str) -> Option<(KeyCode, KeyModifiers)> {
    let mut modifiers = KeyModifiers::empty();
    let mut code = None;
    for token in specification.split('+').map(str::trim) {
        match token.to_ascii_uppercase().as_str() {
            "ALT" => modifiers |= KeyModifiers::ALT,
            "CTRL" | "CONTROL" => modifiers |= KeyModifiers::CONTROL,
            "SHIFT" => modifiers |= KeyModifiers::SHIFT,
            "SUPER" | "META" | "CMD" => modifiers |= KeyModifiers::SUPER,
            "SPACE" => code = Some(KeyCode::Char(' ')),
            "ENTER" => code = Some(KeyCode::Enter),
            "ESC" | "ESCAPE" => code = Some(KeyCode::Esc),
            "BACKSPACE" => code = Some(KeyCode::Backspace),
            "TAB" => code = Some(KeyCode::Tab),
            "F1" => code = Some(KeyCode::F(1)),
            "F2" => code = Some(KeyCode::F(2)),
            "F3" => code = Some(KeyCode::F(3)),
            "F4" => code = Some(KeyCode::F(4)),
            "F5" => code = Some(KeyCode::F(5)),
            "F6" => code = Some(KeyCode::F(6)),
            "F7" => code = Some(KeyCode::F(7)),
            "F8" => code = Some(KeyCode::F(8)),
            "F9" => code = Some(KeyCode::F(9)),
            "F10" => code = Some(KeyCode::F(10)),
            "F11" => code = Some(KeyCode::F(11)),
            "F12" => code = Some(KeyCode::F(12)),
            "\\" | "BACKSLASH" => code = Some(KeyCode::Char('\\')),
            _ if token.len() == 1 => {
                code = token.chars().next().map(KeyCode::Char);
            }
            _ => {}
        }
    }
    code.map(|key_code| (key_code, modifiers))
}

/// Forwards a terminal key event to the agent PTY as bytes.
pub fn forward_key(event: KeyEvent, writer: &mut impl Write) -> std::io::Result<()> {
    if event.kind != KeyEventKind::Press {
        return Ok(());
    }

    match event.code {
        KeyCode::Char(ch) if event.modifiers.contains(KeyModifiers::CONTROL) => {
            writer.write_all(&[control_byte(ch)])?;
        }
        KeyCode::Char(ch) if event.modifiers.contains(KeyModifiers::ALT) => {
            writer.write_all(&[0x1b, ch as u8])?;
        }
        KeyCode::Char(ch) => writer.write_all(ch.encode_utf8(&mut [0; 4]).as_bytes())?,
        KeyCode::Enter => writer.write_all(b"\r")?,
        KeyCode::Backspace => writer.write_all(b"\x7f")?,
        KeyCode::Tab => writer.write_all(b"\t")?,
        KeyCode::Esc => writer.write_all(b"\x1b")?,
        KeyCode::Up => writer.write_all(b"\x1b[A")?,
        KeyCode::Down => writer.write_all(b"\x1b[B")?,
        KeyCode::Right => writer.write_all(b"\x1b[C")?,
        KeyCode::Left => writer.write_all(b"\x1b[D")?,
        KeyCode::Home => writer.write_all(b"\x1b[H")?,
        KeyCode::End => writer.write_all(b"\x1b[F")?,
        KeyCode::PageUp => writer.write_all(b"\x1b[5~")?,
        KeyCode::PageDown => writer.write_all(b"\x1b[6~")?,
        KeyCode::Insert => writer.write_all(b"\x1b[2~")?,
        KeyCode::Delete => writer.write_all(b"\x1b[3~")?,
        KeyCode::F(number) => writer.write_all(function_key_sequence(number))?,
        _ => {}
    }
    writer.flush()
}

fn control_byte(ch: char) -> u8 {
    match ch.to_ascii_lowercase() {
        'a'..='z' => ch.to_ascii_lowercase() as u8 - b'a' + 1,
        '@' | ' ' => 0,
        '[' => 27,
        '\\' => 28,
        ']' => 29,
        '^' => 30,
        '_' => 31,
        _ => ch as u8,
    }
}

fn function_key_sequence(number: u8) -> &'static [u8] {
    match number {
        1 => b"\x1bOP",
        2 => b"\x1bOQ",
        3 => b"\x1bOR",
        4 => b"\x1bOS",
        5 => b"\x1b[15~",
        6 => b"\x1b[17~",
        7 => b"\x1b[18~",
        8 => b"\x1b[19~",
        9 => b"\x1b[20~",
        10 => b"\x1b[21~",
        11 => b"\x1b[23~",
        12 => b"\x1b[24~",
        _ => b"",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_f8_hotkey() {
        let event = KeyEvent::new(KeyCode::F(8), KeyModifiers::NONE);
        assert!(is_voice_hotkey(&event, &["F8".into()]));
    }

    #[test]
    fn detects_ctrl_space_on_wayland_fallback() {
        let event = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL);
        assert!(matches_hotkey(&event, "Ctrl+Space"));
    }

    #[test]
    fn detects_shell_exit_hotkey() {
        let event = KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::CONTROL);
        assert!(is_shell_exit(&event, "Ctrl+\\"));
    }

    #[test]
    fn detects_shell_exit_from_raw_fs_byte() {
        let event = KeyEvent::new(KeyCode::Char('\x1c'), KeyModifiers::NONE);
        assert!(is_shell_exit(&event, "Ctrl+\\"));
    }

    #[test]
    fn forwards_control_c_to_agent() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let mut buffer = Vec::new();
        forward_key(event, &mut buffer).expect("forward");
        assert_eq!(buffer, vec![3]);
    }
}
