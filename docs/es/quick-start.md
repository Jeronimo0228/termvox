# Inicio rápido

TermVox **0.1.0-alpha.11** está disponible en [npm](https://www.npmjs.com/package/termvox)
y como binarios precompilados para Linux, macOS y Windows en
[GitHub Releases](https://github.com/Jeronimo0228/termvox/releases). También puedes
compilar desde el código fuente.

## Instalación

**Recomendado (npm):**

```bash
npm install -g termvox
termvox doctor
termvox-editor-install   # extensión opcional para Cursor/VS Code
```

**Script de instalación:**

```bash
curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
```

Versión fija:

```bash
TERMVOX_VERSION=v0.1.0-alpha.11 curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
```

**Desde fuente:**

```bash
git clone https://github.com/Jeronimo0228/termvox.git
cd termvox
# Fedora: sudo dnf install alsa-lib-devel
# Debian/Ubuntu: sudo apt-get install libasound2-dev pkg-config
cargo install --path crates/termvox-cli
```

Instala al menos un agente compatible: `codex`, `claude`, `agent` (Cursor),
`gemini`, `aider`, `amp` u `opencode`. La transcripción local (Whisper) no
requiere API key.

## Configuración inicial

```bash
termvox init --preset cursor    # u opencode, claude, codex, gemini, rust-web
termvox models install default
termvox config validate
termvox doctor
```

## Uso recomendado — shell integrado

Para Cursor, OpenCode, Claude Code, Codex y demás TUIs, abre el proyecto y ejecuta:

```bash
cd tu-proyecto
termvox shell
```

Cambiar agente en una sesión:

```bash
termvox shell --agent opencode
termvox shell --agent cursor
```

| Acción | Tecla |
| --- | --- |
| Activar voz | **F8** o **Ctrl+Space** (Wayland) |
| Salir del wrapper TermVox | **Ctrl+\\** |
| Ctrl+C al agente | **Ctrl+C** (no cierra TermVox) |

TermVox guarda el id de chat en `.termvox/session.json` y lo reutiliza al volver
al mismo directorio. Usa `termvox shell --fresh` para ignorar la sesión guardada.

## Modo alternativo — branded / companion

Con `display = "branded"` o `display = "companion"` en `termvox.toml`:

```bash
termvox start
```

Mantén **Espacio**, habla y suelta. Confirma con `y` si se pide. Pulsa `q` o
**Ctrl+C** para salir.

Si `display = "shell"`, `termvox start` delega a `termvox shell`.

## Prueba rápida de micrófono

```bash
termvox test --seconds 3
```

## Transcripción OpenAI (opcional)

```toml
speech_engine = "openai"

[openai]
api_key_env = "OPENAI_API_KEY"
```

```bash
export OPENAI_API_KEY="tu-clave"
```

Documentación en inglés: [quick start](../quick-start.md), [agent shell](../agent-shell.md),
[CLI reference](../cli-reference.md), [configuración](../configuration.md).
