//! Strip keyboard/mouse capture sequences from agent PTY output before they hit the real terminal.

/// Remove CSI/`>…u` sequences that would steal keyboard focus from `TermVox` on the host TTY.
#[must_use]
pub fn filter_agent_output(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut index = 0;
    while index < data.len() {
        if data[index] == 0x1b {
            if let Some((sequence, end)) = parse_escape_sequence(data, index) {
                if should_block_sequence(sequence) {
                    index = end;
                    continue;
                }
            }
        }
        out.push(data[index]);
        index += 1;
    }
    out
}

fn parse_escape_sequence(data: &[u8], start: usize) -> Option<(&[u8], usize)> {
    if start + 1 >= data.len() {
        return None;
    }
    match data[start + 1] {
        b'[' => {
            let mut end = start + 2;
            while end < data.len() && !data[end].is_ascii_alphabetic() && data[end] != b'~' {
                end += 1;
            }
            if end < data.len() {
                end += 1;
            }
            Some((&data[start..end], end))
        }
        b'>' if start + 2 < data.len() => {
            let mut end = start + 2;
            while end < data.len() && data[end] != b'u' {
                end += 1;
            }
            if end < data.len() {
                end += 1;
                Some((&data[start..end], end))
            } else {
                None
            }
        }
        b'O' if start + 2 < data.len() => Some((&data[start..=start + 2], start + 3)),
        _ => None,
    }
}

fn should_block_sequence(sequence: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(sequence) else {
        return false;
    };
    if text.starts_with("\x1b>") && text.ends_with('u') {
        return true;
    }
    if !text.starts_with("\x1b[") {
        return false;
    }
    let mode = text.chars().last().unwrap_or(' ');
    if mode != 'h' && mode != 'l' {
        return false;
    }
    let body = text
        .trim_start_matches("\x1b[")
        .trim_end_matches(['h', 'l']);
    body.split(';').any(|part| {
        matches!(
            part.trim_start_matches('?'),
            "9001" | "9002" | "1000" | "1002" | "1003" | "1006" | "884" | "2004"
        )
    })
}

/// Reclaim host keyboard from agent protocols after PTY output is rendered.
pub fn reclaim_host_keyboard(out: &mut impl std::io::Write) -> std::io::Result<()> {
    out.write_all(b"\x1b[?9001l\x1b[?9002l\x1b[<u")?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_kitty_keyboard_enable() {
        let input = b"hello\x1b[?9001hworld";
        let filtered = filter_agent_output(input);
        assert_eq!(filtered, b"helloworld");
    }

    #[test]
    fn blocks_bracketed_paste_enable() {
        let input = b"prompt\x1b[?2004h";
        let filtered = filter_agent_output(input);
        assert_eq!(filtered, b"prompt");
    }

    #[test]
    fn keeps_alternate_screen_toggle() {
        let input = b"\x1b[?1049h\x1b[2J";
        let filtered = filter_agent_output(input);
        assert_eq!(filtered, input);
    }
}
