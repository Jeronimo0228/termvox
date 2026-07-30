# Solución de problemas

Empieza con:

```bash
termvox config path
termvox config validate
termvox doctor
```

Lee cada línea del doctor; un check fallido no siempre produce código de salida
distinto de cero. El JSON incluye `hints` con sugerencias contextuales.

## Build sin ALSA

En Debian/Ubuntu: `libasound2-dev` y `pkg-config`. En Fedora/RHEL:
`alsa-lib-devel` y `pkgconf-pkg-config`, luego recompila.

## Sin micrófono o permiso denegado

- Confirma que el SO ve el micrófono.
- Concede acceso al micrófono a la terminal.
- Cierra apps con acceso exclusivo al dispositivo.
- Quita `audio.device` para usar el default del SO, o copia el nombre exacto
  que reporta `termvox doctor`.
- El audio suele fallar en contenedores o SSH remoto sin reenvío explícito.

## Push-to-talk no suelta la tecla

Usa una terminal local y enfocada; prueba `ENTER`, `TAB`, una tecla `F` o un
carácter simple. Algunos emuladores no entregan eventos de soltar tecla. Los
acordes como `ALT+SPACE` no están soportados.

## “audio contains no voice”

Baja `audio.vad_threshold_db` (más negativo), acércate al micrófono o verifica
el dispositivo de entrada.

## Modelo Whisper no encontrado

```bash
termvox models install default    # fast → ggml-tiny.bin
termvox models install accurate   # balanced/accurate → ggml-base.bin
termvox models status default
```

Usa rutas absolutas en `termvox.toml`; `~` entre comillas no siempre se
expande. En uso no interactivo, TermVox no descarga modelos sin consentimiento.

## Perfil vs modelo

Si `termvox doctor` sugiere un modelo distinto al instalado:

| Perfil      | Modelo esperado   |
|-------------|-------------------|
| `fast`      | `ggml-tiny.bin`   |
| `balanced`  | `ggml-base.bin`   |
| `accurate`  | `ggml-base.bin`+  |

## OpenAI: autenticación o HTTP

Confirma que la variable en `openai.api_key_env` está exportada. Verifica
`openai.endpoint`: ese host recibe audio y credenciales.

## Agente “not installed”

```bash
codex --version
claude --version
agent --version
gemini --version
aider --version
amp --version
opencode --version
```

## Agente no autenticado

Busca `[!!]` bajo `agent/...` en el doctor. TermVox sugiere el comando de login
cuando puede detectar credenciales faltantes.

```bash
opencode auth login
claude login
codex login
gemini auth login
export OPENAI_API_KEY=...   # Aider y algunos proveedores OpenCode
```

`termvox shell` y `termvox start` (excepto companion) fallan rápido con el
mismo mensaje.

## Parakeet o Vosk no listos

TermVox define el contrato del sidecar, no el ejecutable. Confirma que
`termvox-parakeet` o `termvox-vosk` está en `PATH` y el modelo existe.

## Plugin inspect/test falla

Usa ruta absoluta al ejecutable, habilita el plugin y revisa stderr. Solo JSON-RPC
por línea en stdout. Variables de entorno solo las listadas en `env_allowlist`.

## Shell integrado (`termvox shell`)

- **Barra desaparece** — los TUI del agente redibujan; TermVox redibuja la barra.
  Actualiza a alpha.8+.
- **F8 no funciona en Wayland** — usa **Ctrl+Space** (`[shell].alt_hotkeys`) o
  `termvox daemon start` para hotkey global.
- **No puedo salir** — **Ctrl+\\** (`[shell].exit_hotkey`), no Ctrl+C.
- **Diálogo de confianza Cursor** — `termvox shell` auto-confía, o
  `agents.cursor.trust_workspace = true` en branded.
- **Sesión no reanuda** — revisa `.termvox/session.json`; prueba `--fresh`;
  confirma `discover_session = true`. OpenCode ya no requiere `sqlite3` en PATH.

## Companion vs shell

Si usas Cursor u OpenCode con `display = "companion"`, el doctor sugerirá
`termvox shell` o `display = "shell"` para la barra integrada.

## `termvox update` no actualiza

Es informativo. Descarga desde
<https://github.com/Jeronimo0228/termvox/releases> o recompila desde fuente.

Al pedir ayuda, incluye revisión de TermVox, SO, versión de Rust, configuración
(sin secretos), salida del comando y pasos exactos. Ver
[política de soporte](https://github.com/Jeronimo0228/termvox/blob/main/SUPPORT.md).
