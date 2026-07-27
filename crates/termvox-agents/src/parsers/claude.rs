use termvox_core::AgentEvent;

use super::{Format, structured};

pub fn parse(line: &str) -> Option<AgentEvent> {
    structured(
        line,
        Format {
            session_keys: &["session_id"],
            text_keys: &["result", "text", "content", "message"],
            started_types: &["system", "init"],
            completed_types: &["result"],
        },
    )
}
