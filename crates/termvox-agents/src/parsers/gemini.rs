use termvox_core::AgentEvent;

use super::{Format, structured};

pub fn parse(line: &str) -> Option<AgentEvent> {
    structured(
        line,
        Format {
            session_keys: &["session_id"],
            text_keys: &["response", "text", "content", "message", "delta"],
            started_types: &["init", "started"],
            completed_types: &["result", "completed", "final"],
        },
    )
}
