//! Shared argv fragments for subprocess and interactive agent launches.

use crate::SupportedAgent;

/// Resume arguments inserted before trailing CLI args in interactive shell mode.
#[must_use]
pub fn interactive_resume_args(kind: SupportedAgent, remote_id: &str) -> Vec<String> {
    match kind {
        SupportedAgent::Codex => vec!["resume".into(), remote_id.into()],
        SupportedAgent::Claude
        | SupportedAgent::Cursor
        | SupportedAgent::Gemini
        | SupportedAgent::Amp => vec!["--resume".into(), remote_id.into()],
        SupportedAgent::OpenCode => vec!["--session".into(), remote_id.into()],
        SupportedAgent::Aider => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_args_match_subprocess_contract() {
        assert_eq!(
            interactive_resume_args(SupportedAgent::OpenCode, "ses_1"),
            vec!["--session", "ses_1"]
        );
        assert!(interactive_resume_args(SupportedAgent::Aider, "x").is_empty());
    }
}
