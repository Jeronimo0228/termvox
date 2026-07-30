# Shell integrado (`termvox shell`)

Capa de voz unificada para **los siete** agentes CLI: Cursor, Claude Code, Codex,
Gemini, Aider, Amp y OpenCode. Una sola terminal, TUI real del agente y barra de
micrófono persistente abajo.

## Comandos

```bash
termvox shell                         # agente de termvox.toml
termvox shell --agent opencode        # OpenCode + barra TermVox
termvox shell --agent cursor          # Cursor Agent + barra TermVox
termvox shell --fresh                 # ignorar sesión guardada
termvox shell --agent claude -- --model sonnet
```

## Teclas

| Tecla | Efecto |
| --- | --- |
| **F8** | Toggle voz (configurable en `[shell].hotkey`) |
| **Ctrl+Space** | Alternativa en Wayland (`[shell].alt_hotkeys`) |
| **Ctrl+\\** | Salir del wrapper TermVox (`[shell].exit_hotkey`) |
| **Ctrl+C** | Va al agente, no mata TermVox |

## Sesión por workspace

Con `[workspace].persist_session = true` (default), TermVox guarda
`.termvox/session.json` en tu proyecto y reanuda el chat al volver. Si no hay id
guardado, `discover_session` busca en almacenes del agente (transcripts de Cursor,
sqlite de OpenCode, proyectos de Claude).

## Configuración mínima

```toml
agent = "cursor"
language = "es"

[agents.cursor]
display = "shell"

[shell]
hotkey = "F8"
alt_hotkeys = ["Ctrl+Space"]
exit_hotkey = "Ctrl+\\"
auto_submit = true
skip_confirmation = true

[workspace]
persist_session = true
discover_session = true
```

Cursor en modo shell confía el workspace automáticamente (sin diálogo de trust).

## Shim (opcional)

```bash
termvox install-shim --agent cursor --force
# ~/.local/bin/agent → termvox shell
```

Guía completa en inglés: [agent-shell.md](../agent-shell.md).
