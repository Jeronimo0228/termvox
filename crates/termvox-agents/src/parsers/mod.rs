use serde_json::Value;
use termvox_core::AgentEvent;

pub mod aider;
pub mod amp;
pub mod claude;
pub mod codex;
pub mod cursor;
pub mod gemini;
pub mod opencode;

#[derive(Clone, Copy)]
pub(super) struct Format {
    pub session_keys: &'static [&'static str],
    pub text_keys: &'static [&'static str],
    pub started_types: &'static [&'static str],
    pub completed_types: &'static [&'static str],
}

pub(super) fn structured(line: &str, format: Format) -> Option<AgentEvent> {
    let value: Value = serde_json::from_str(line).ok()?;
    let event_type = value
        .get("type")
        .or_else(|| value.get("event"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let session_id = find_string(&value, format.session_keys);
    if format
        .started_types
        .iter()
        .any(|candidate| event_type == *candidate || event_type.contains(candidate))
    {
        return Some(AgentEvent::Started { session_id });
    }
    if event_type.contains("error") || event_type.contains("failed") {
        return Some(AgentEvent::Failed {
            message: find_string(&value, &["message", "error", "detail"])
                .unwrap_or_else(|| value.to_string()),
        });
    }
    if event_type.contains("tool") || event_type == "item.started" {
        return Some(AgentEvent::Tool {
            name: find_string(&value, &["name", "tool_name"]).unwrap_or_else(|| "tool".into()),
            status: event_type.to_owned(),
        });
    }
    let text = find_string(&value, format.text_keys)?;
    if format
        .completed_types
        .iter()
        .any(|candidate| event_type == *candidate || event_type.contains(candidate))
    {
        Some(AgentEvent::Completed { result: text })
    } else {
        Some(AgentEvent::Message { text })
    }
}

fn find_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(object) => {
            for key in keys {
                if let Some(value) = object.get(*key) {
                    if let Some(text) = value.as_str() {
                        return Some(text.to_owned());
                    }
                    if let Some(found) = find_string(value, keys) {
                        return Some(found);
                    }
                }
            }
            object.values().find_map(|value| find_string(value, keys))
        }
        Value::Array(values) => values.iter().find_map(|value| find_string(value, keys)),
        _ => None,
    }
}
