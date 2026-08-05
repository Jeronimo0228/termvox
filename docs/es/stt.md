# Configurar el reconocimiento de voz (STT)

Guía práctica para mejorar la precisión de TermVox (Whisper local) y saber
dónde se edita cada opción.

## Resumen rápido

Por defecto TermVox usa el perfil **`fast`** con el modelo **`ggml-tiny`**:
rápido, pero impreciso (sobre todo en español y frases largas).

**Recomendado para uso diario / español:**

```bash
termvox models install accurate
termvox config path
```

Edita el archivo **global** (`~/.config/termvox/termvox.toml` en Linux) o el
`termvox.toml` del proyecto:

```toml
performance_profile = "balanced"
language = "es"

[whisper]
model = "/home/TU_USUARIO/.local/share/termvox/models/ggml-base.bin"
optimize_for_latency = false
prewarm_on_start = true
max_threads = 6

[audio]
max_seconds = 60
vad_silence_ms = 700
vad_threshold_db = -42.0
auto_stop_on_silence = true
```

Luego:

```bash
termvox config validate
termvox doctor
termvox test --seconds 4
```

## Dónde se guarda la configuración

```bash
termvox config path      # rutas global y de proyecto
termvox config show      # configuración efectiva (ya mezclada)
termvox config validate
```

Orden de prioridad (gana el último):

1. Defaults del binario  
2. Config **global** (`termvox init --global`)  
3. Config de **proyecto** (`./termvox.toml` o `termvox init`)  
4. Variables de entorno `TERMVOX_*`

Si tienes `whisper.model = "...tiny..."` en el proyecto, **pisa** un perfil
`balanced` del global. Usa la misma ruta `ggml-base.bin` en ambos, o deja el
modelo solo en el global.

## Perfiles de rendimiento

| Perfil | Modelo | Velocidad | Precisión | Cuándo usarlo |
| --- | --- | --- | --- | --- |
| `fast` | tiny (~74 MiB) | Muy rápida | Baja | Comandos cortos, poca RAM |
| `balanced` | base (~142 MiB) | Media | Buena | **Uso diario / español** |
| `accurate` | base (~142 MiB) | Más lenta | Mejor | Ruido, términos técnicos |
| `custom` | El que indiques | Manual | Manual | Control total |

```toml
performance_profile = "balanced"
# performance_profile = "accurate"
```

### Qué cambia cada perfil

| Opción | `fast` | `balanced` | `accurate` |
| --- | --- | --- | --- |
| Modelo | tiny | base | base |
| `max_seconds` | 30 | 60 | 120 |
| `vad_silence_ms` | 400 | 600 | (default) |
| Auto-stop por silencio | sí | sí | **no** (paras tú con F8) |
| Precarga del modelo | no | sí | sí |
| Optimizar latencia | sí | sí | **no** |

### Instalar modelos

```bash
termvox models install default      # tiny (perfil fast)
termvox models install accurate     # base (balanced / accurate)
termvox models list
termvox models status accurate
```

Los archivos quedan en `~/.local/share/termvox/models/` (Linux). Usa **ruta
absoluta** en el TOML; `~` entre comillas no siempre se expande.

## Idioma

```toml
language = "es"
```

Si hablas español y dejas `en` (o vacío mal configurado), la calidad cae mucho.
Ese valor se envía a Whisper en cada transcripción.

## Micrófono y VAD (detector de voz)

```toml
[audio]
# device = "Nombre exacto de termvox doctor"
sample_rate = 16000
max_seconds = 60
vad_threshold_db = -42.0
vad_silence_ms = 700
auto_stop_on_silence = true
```

| Problema | Qué probar |
| --- | --- |
| Corta a mitad de frase | Sube `vad_silence_ms` (700–1000) o `auto_stop_on_silence = false` |
| Captura ruido / “música” | Sube `vad_threshold_db` (p. ej. `-40`) o acércate al mic |
| “audio contains no voice” | Baja `vad_threshold_db` (p. ej. `-50`); revisa el dispositivo |
| Se pierden las primeras palabras | Habla un instante después de F8 |
| Prompts largos cortados | Sube `max_seconds` o usa `accurate` |

Lista de dispositivos:

```bash
termvox doctor
```

Copia el nombre exacto del micrófono a `audio.device` si el default no es el correcto.

## Opciones de Whisper

```toml
[whisper]
model = "/ruta/absoluta/ggml-base.bin"
threads = 0
max_threads = 6
prewarm_on_start = true
optimize_for_latency = false
use_gpu = false
streaming = true
```

- `optimize_for_latency = true` → más rápido, algo peor en frases largas.  
- `streaming = true` → parciales en la barra de mic mientras decodifica.  
- En `termvox shell`, la precarga del modelo se retrasa hasta el **primer F8**
  para no ensuciar el TUI del agente.

### Mensaje “AMX is not ready to be used!”

Aviso inofensivo de ggml en algunas CPUs Linux. Desde **alpha.14** TermVox lo
oculta para que no rompa la UI de OpenCode/Cursor. El STT sigue funcionando.

## Diccionario (correcciones después del STT)

```toml
[pipeline]
dictionary = { "api rest" = "REST API", "open code" = "OpenCode", "curso" = "Cursor" }
```

Reemplazos literales sobre el texto ya transcrito. Útil para nombres propios.

## STT en la nube (opcional)

```toml
speech_engine = "openai"
language = "es"

[openai]
api_key_env = "OPENAI_API_KEY"
```

```bash
export OPENAI_API_KEY="sk-..."
```

El audio se sube al endpoint configurado. No guardes la API key en el TOML.
Detalle: [Speech engines](../speech-engines.md).

## Comprobar que todo quedó bien

```bash
termvox config show | head -40
termvox doctor
# Debe verse: performance_profile = balanced y ggml-base.bin

termvox test --seconds 4
# Habla una frase de código; no debería salir solo "[Música]"

termvox shell --agent opencode --fresh
# F8 o Ctrl+Space → hablar → confirmar
```

## Ejemplo completo (español + calidad)

```toml
performance_profile = "balanced"
speech_engine = "whisper"
language = "es"
agent = "opencode"
confirmation = true

[audio]
sample_rate = 16000
max_seconds = 60
vad_threshold_db = -42.0
vad_silence_ms = 700
auto_stop_on_silence = true

[whisper]
model = "/home/TU_USUARIO/.local/share/termvox/models/ggml-base.bin"
threads = 0
max_threads = 6
prewarm_on_start = true
optimize_for_latency = false
streaming = true

[shell]
hotkey = "F8"
alt_hotkeys = ["Ctrl+Space"]
exit_hotkey = "Ctrl+\\"
auto_submit = true
skip_confirmation = false

[pipeline]
dictionary = { "open code" = "OpenCode" }
```

## Más documentación

- [Performance (EN)](../performance.md) — referencia completa de perfiles  
- [Configuration (EN)](../configuration.md) — todas las claves  
- [Speech engines (EN)](../speech-engines.md) — Whisper / OpenAI / sidecars  
- [Solución de problemas](troubleshooting.md)
