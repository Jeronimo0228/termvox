//! Localized status strings for the integrated shell bar.

pub(super) fn ready(language: &str, session_hint: Option<&str>) -> String {
    let base = match language {
        "es" => "listo",
        _ => "ready",
    };
    match session_hint {
        Some(id) if language == "es" => format!("{base} · sesión {}", short_id(id)),
        Some(id) => format!("{base} · session {}", short_id(id)),
        None => base.into(),
    }
}

pub(super) fn recording(language: &str) -> &'static str {
    match language {
        "es" => "Escuchando",
        _ => "Listening",
    }
}

pub(super) fn transcribing(language: &str) -> &'static str {
    match language {
        "es" => "Transcribiendo",
        _ => "Transcribing",
    }
}

pub(super) fn no_speech(language: &str) -> String {
    match language {
        "es" => "No se detectó voz".into(),
        _ => "No speech detected".into(),
    }
}

pub(super) fn injected(language: &str) -> String {
    match language {
        "es" => "Transcripción enviada al agente".into(),
        _ => "Transcript sent to agent".into(),
    }
}

pub(super) fn confirm(language: &str, prompt: &str) -> String {
    match language {
        "es" => format!("¿Enviar? [y/N] {prompt}"),
        _ => format!("Send? [y/N] {prompt}"),
    }
}

pub(super) fn exiting(language: &str) -> &'static str {
    match language {
        "es" => "TermVox · cerrando sesión",
        _ => "TermVox · closing session",
    }
}

pub(super) fn resume_notice(language: &str, id: &str) -> String {
    match language {
        "es" => format!("Reanudando sesión {id}"),
        _ => format!("Resuming session {id}"),
    }
}

pub(super) fn voice_label(language: &str) -> &'static str {
    match language {
        "es" => "voz",
        _ => "voice",
    }
}

pub(super) fn exit_label(language: &str) -> &'static str {
    match language {
        "es" => "salir",
        _ => "exit",
    }
}

pub(super) fn language_tag(language: &str) -> &str {
    match language {
        "es" => "ES",
        _ => "EN",
    }
}

pub(super) fn partial_prefix(_language: &str) -> &'static str {
    "…"
}

pub(super) fn transcribing_dots(frame: u8) -> String {
    match frame % 4 {
        0 => String::new(),
        1 => ".".into(),
        2 => "..".into(),
        _ => "...".into(),
    }
}

fn short_id(id: &str) -> String {
    if id.len() <= 12 {
        return id.to_string();
    }
    format!("{}…", &id[..10])
}
