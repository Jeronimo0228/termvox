# Inicio rápido

TermVox es una interfaz de voz para agentes de programación en la terminal. El
proyecto está en fase alfa. Incluye adaptadores para Codex, Claude, Cursor,
Gemini, Aider y Amp; integra Whisper.cpp y transcripción de OpenAI; y ofrece un
contrato de procesos auxiliares para Parakeet y Vosk. Debes instalar por
separado los agentes, ejecutables y modelos compatibles.

La única instalación verificada por esta documentación es compilar el código
fuente. No se afirma que existan binarios publicados o firmados.

## Instalación

Instala Rust 1.86 o posterior, Git, las dependencias de audio del sistema y un
agente compatible.

```bash
git clone https://github.com/Jeronimo0228/termvox.git
cd termvox

# Fedora/RHEL: sudo dnf install alsa-lib-devel
# Debian/Ubuntu: sudo apt-get install libasound2-dev pkg-config
cargo install --path crates/termvox-cli

termvox init
termvox doctor
```

En macOS instala primero las herramientas de línea de comandos de Xcode y
autoriza el micrófono para la aplicación de terminal. En Windows usa Rust con
la cadena MSVC y Microsoft C++ Build Tools.

## Transcripción

Para Whisper.cpp instala `whisper-cli`, consigue un modelo GGML de una fuente
confiable y usa una ruta absoluta:

```toml
speech_engine = "whispercpp"

[whisper]
executable = "whisper-cli"
model = "/ruta/absoluta/ggml-base.bin"
```

Para OpenAI:

```toml
speech_engine = "openai"

[openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o-mini-transcribe"
endpoint = "https://api.openai.com/v1/audio/transcriptions"
```

```bash
export OPENAI_API_KEY="tu-clave"
```

Este modo envía cada grabación al endpoint configurado. Revisa las políticas de
privacidad, retención y costos del proveedor.

## Uso

Selecciona `agent = "codex"`, `"claude"`, `"cursor"`, `"gemini"`, `"aider"` o
`"amp"` en `termvox.toml` y ejecuta:

```bash
termvox config validate
termvox doctor
termvox start
```

Mantén presionada la barra espaciadora, habla y suéltala. TermVox muestra la
transcripción y el prompt procesado. Por defecto debes responder `y` o `yes`
antes de enviarlo. Pulsa `q` o `Ctrl+C` para salir.

TermVox no es un sandbox. El agente conserva sus propios permisos, credenciales
y políticas. No dictes secretos a un servicio remoto y conserva la
confirmación manual.

Documentación completa en inglés: [inicio rápido](../quick-start.md),
[instalación](../installation.md), [configuración](../configuration.md),
[seguridad y privacidad](../privacy-security.md) y
[solución de problemas](../troubleshooting.md).
