use serde_json::Value;
use termvox_core::AgentEvent;

pub fn parse(line: &str) -> Option<AgentEvent> {
    let value: Value = serde_json::from_str(line).ok()?;
    let event_type = value.get("type")?.as_str()?;
    let session_id = value
        .get("sessionID")
        .or_else(|| value.get("session_id"))
        .and_then(Value::as_str)
        .map(str::to_owned);

    match event_type {
        "step_start" => Some(AgentEvent::Started { session_id }),
        "text" => {
            let text = value
                .pointer("/part/text")
                .or_else(|| value.get("text"))
                .and_then(Value::as_str)?;
            Some(AgentEvent::Message {
                text: text.to_owned(),
            })
        }
        "step_finish" => Some(AgentEvent::Completed {
            result: value
                .pointer("/part/text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
        }),
        "tool_use" => Some(AgentEvent::Tool {
            name: value
                .pointer("/part/tool")
                .and_then(Value::as_str)
                .unwrap_or("tool")
                .to_owned(),
            status: value
                .pointer("/part/state/status")
                .and_then(Value::as_str)
                .unwrap_or("tool_use")
                .to_owned(),
        }),
        "error" => Some(AgentEvent::Failed {
            message: value
                .pointer("/error/data/message")
                .or_else(|| value.pointer("/error/message"))
                .or_else(|| value.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("OpenCode error")
                .to_owned(),
        }),
        _ if event_type.contains("error") => Some(AgentEvent::Failed {
            message: value.to_string(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_event() {
        let line = r#"{"type":"text","sessionID":"ses_abc","part":{"type":"text","text":"Hello"}}"#;
        let event = parse(line).expect("event");
        assert!(matches!(event, AgentEvent::Message { text } if text == "Hello"));
    }
}
