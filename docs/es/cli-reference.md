# Referencia CLI

```text
termvox [--config RUTA] COMANDO
```

`--config RUTA` selecciona un archivo de configuración del proyecto. La
configuración global se carga primero.

La ayuda integrada (`termvox --help`, `termvox <cmd> --help`) y
`termvox manpage` salen de las mismas definiciones Clap. Para calidad de STT
(`performance_profile`, modelos Whisper), ver [STT](stt.md) y
[Performance](../performance.md).

## Comandos

### `termvox init [--global] [--force] [--preset PRESET]`

Escribe los valores por defecto en `termvox.toml`, o en el directorio de
configuración del SO con `--global`. Los archivos existentes se conservan
salvo que uses `--force`.

Presets: `cursor`, `codex`, `claude`, `gemini`, `opencode`, `rust-web`.

### `termvox setup [--global] [--force]`

En una terminal interactiva, pregunta idioma, tecla push-to-talk, motor de voz
y agente, luego escribe la configuración. Con stdin no interactivo, escribe los
defaults. Para Cursor/OpenCode puede ofrecer `display = "shell"`.

### `termvox start [--toggle] [--global-hotkey ATAJO]`

Inicia la sesión interactiva según `[agents.<nombre>].display`:

- **`shell`** — delega a `termvox shell` (barra de mic + TUI del agente)
- **`branded`** — pie TermVox; el agente corre como subproceso JSONL por frase
- **`companion`** — transcribe y pega en otra ventana vía portapapeles

En modo branded, mantén la tecla push-to-talk (por defecto **Espacio**), suelta
para transcribir. `q` o **Ctrl+C** salen. Con `--toggle`, una pulsación inicia
y otra detiene la grabación.

Las sesiones de workspace se hidratan desde `.termvox/session.json` y se guardan
al salir.

### `termvox doctor [--json]`

Valida configuración, lista dispositivos de entrada, comprueba el motor de voz,
explora los **siete** agentes CLI integrados y reporta capacidad de hotkeys.
Los fallos aparecen como `[!!]`; agentes opcionales no instalados como `[--]`.

Lee toda la salida; el código de salida no siempre refleja el estado. `--json`
incluye el campo `hints` con sugerencias (shell vs companion, modelos Whisper,
Wayland, ruta de sesión).

### `termvox plugins [list|inspect ID|test ID]`

- `list` — adaptadores integrados y plugins configurados
- `inspect ID` — arranca el plugin, muestra manifiesto y lo apaga
- `test ID` — también llama `probe`

No descubre ni instala plugins.

### `termvox config [path|show|validate]`

- `path` — rutas global y de proyecto
- `show` — TOML fusionado sin secretos
- `validate` — parsea, fusiona y valida restricciones

Claves útiles de STT: `performance_profile`, `language`, `[whisper]`, `[audio]`.
Ver [STT](stt.md).

### `termvox test [--seconds N]`

Graba N segundos (default 3), recorta silencio, transcribe e imprime el texto.
No envía nada al agente.

### `termvox record start|stop|toggle`

Flujo para atajos del gestor de ventanas: `start` crea un marcador por usuario;
`stop` lo elimina; `toggle` alterna.

### `termvox models list|install|status|remove|download`

Gestiona artefactos Whisper revisados. `default` instala `whisper-tiny` (~74 MiB);
`accurate` instala `whisper-base` (~142 MiB). El perfil `balanced` espera
`ggml-base.bin`; `fast` usa `ggml-tiny.bin`.

### `termvox shell [--agent AGENTE] [--fresh] [--] [ARGS...]`

Lanza el CLI del agente en un PTY con barra de mic integrada. Compatible con
Codex, Claude, Cursor, Gemini, Aider, Amp y OpenCode.

```bash
termvox shell
termvox shell --agent opencode
termvox shell --fresh
termvox shell --agent cursor -- --model gpt-5
```

**Voz:** **F8** (default), más `[shell].alt_hotkeys` (p. ej. **Ctrl+Space** en
Wayland). **Salir del wrapper:** **Ctrl+\\** — **Ctrl+C** va al agente.

Persiste ids de sesión en `.termvox/session.json` cuando
`[workspace].persist_session` está activo (default). Falla antes de lanzar si el
agente no está autenticado.

### `termvox install-shim [--agent AGENTE] [--force]`

Instala `~/.local/bin/<agente>` como wrapper de `termvox shell` (Unix).

### `termvox daemon start [--background]`

Servicio de voz en segundo plano con hotkey global (default `ALT+SPACE`). Ver
[Modo daemon](../daemon.md).

### `termvox talk`

Alterna grabación en un daemon en ejecución (IPC).

### `termvox bench [--runs N]`

Reporte JSON de latencia (P50/P95) del modelo Whisper configurado.

### `termvox update`

Información de versión y releases; no instala actualizaciones automáticamente.

### `termvox completions SHELL`

Escribe completions a stdout para bash/zsh/fish/powershell/elvish.

### `termvox manpage [--output RUTA]`

Genera la página man `termvox(1)` desde la definición Clap. Ejemplo:

```bash
termvox manpage --output ~/.local/share/man/man1/termvox.1
man termvox
```

## Registro

```bash
RUST_LOG=info termvox doctor
RUST_LOG=debug termvox start
```

Los logs pueden exponer nombres de dispositivos o errores de agentes. Revísalos
antes de compartirlos.
