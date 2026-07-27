use anyhow::{Context, Result, bail};

/// Copies UTF-8 text to the system clipboard.
pub(crate) fn copy_text(text: &str) -> Result<()> {
    if text.is_empty() {
        bail!("nothing to copy");
    }
    arboard::Clipboard::new()
        .context("clipboard unavailable; on Linux install wl-clipboard or xclip; on Windows ensure the clipboard service is running")?
        .set_text(text)
        .context("failed to write to the system clipboard")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_text() {
        assert!(copy_text("").is_err());
    }
}
