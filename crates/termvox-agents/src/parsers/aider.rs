use termvox_core::AgentEvent;

pub fn parse(line: &str) -> Option<AgentEvent> {
    (!line.trim().is_empty()).then(|| AgentEvent::Message {
        text: line.to_owned(),
    })
}
