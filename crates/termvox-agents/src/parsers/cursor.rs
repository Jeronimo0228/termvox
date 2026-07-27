use termvox_core::AgentEvent;

use super::{Format, structured};

pub fn parse(line: &str) -> Option<AgentEvent> {
    structured(
        line,
        Format {
            session_keys: &["session_id", "chat_id"],
            text_keys: &["result", "text", "content", "message", "delta"],
            started_types: &["system", "started", "init"],
            completed_types: &["result", "completed", "final"],
        },
    )
}
