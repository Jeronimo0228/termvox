use termvox_core::AgentEvent;

use super::{Format, structured};

pub fn parse(line: &str) -> Option<AgentEvent> {
    structured(
        line,
        Format {
            session_keys: &["thread_id"],
            text_keys: &["result", "text", "content", "message", "delta"],
            started_types: &["thread.started"],
            completed_types: &["result", "completed"],
        },
    )
}
