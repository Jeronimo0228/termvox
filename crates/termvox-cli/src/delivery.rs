use anyhow::Result;
use termvox_core::PromptDelivery;

use crate::{clipboard, paste};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeliveryOutcome {
    pub clipboard: bool,
    pub paste: bool,
}

pub(crate) fn deliver_prompt(text: &str, mode: PromptDelivery) -> Result<DeliveryOutcome> {
    let mut outcome = DeliveryOutcome {
        clipboard: false,
        paste: false,
    };
    match mode {
        PromptDelivery::Clipboard | PromptDelivery::Both => {
            clipboard::copy_text(text)?;
            outcome.clipboard = true;
        }
        PromptDelivery::Paste => {}
    }
    if matches!(mode, PromptDelivery::Paste | PromptDelivery::Both) {
        if outcome.clipboard {
            paste::paste_after_clipboard_delay();
        }
        paste::paste_focused()?;
        outcome.paste = true;
    }
    Ok(outcome)
}
