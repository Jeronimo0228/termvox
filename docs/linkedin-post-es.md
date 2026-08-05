# Borrador — post LinkedIn (español)

> **Cuándo publicar:** después de grabar el video (ver [demo.md](demo.md)) y
> recibir feedback de al menos 2 beta testers.

---

## Texto sugerido

Estoy construyendo **TermVox** en público: una capa de **voz local** para agentes de coding en la terminal.

Hablas → **Whisper** transcribe en tu máquina (sin API key de speech) → revisas el prompt → solo entonces llega a **Cursor, OpenCode, Claude Code, Codex** y otros CLIs que ya usas.

No es otro chatbot. Es push-to-talk dentro de `termvox shell`, con barra de micrófono integrada en el TUI del agente.

```bash
npm install -g termvox
termvox shell --agent cursor
```

**Estado:** alpha temprana (`0.1.0-alpha.11`) — APIs y adapters pueden cambiar.
Busco **early adopters** que ya usen agent CLIs y quieran probar voz offline.

Repo: https://github.com/Jeronimo0228/termvox
npm: https://www.npmjs.com/package/termvox

Si lo pruebas, el feedback en GitHub me ayuda mucho (especialmente macOS / Windows).

#OpenSource #Rust #DeveloperTools #AI #Voice #CLI #Cursor #BuildInPublic

---

## Variante corta (si LinkedIn trunca)

**TermVox** = voz local (Whisper) + confirmación antes de enviar → agentes en terminal (Cursor, OpenCode, Claude…).

Alpha preview. `npm install -g termvox && termvox shell`

https://github.com/Jeronimo0228/termvox

#BuildInPublic #Rust #CLI

---

## Checklist antes de publicar

- [ ] Video de 45–90 s grabado con OBS (ver [demo.md](demo.md))
- [ ] Probaste el happy path hoy en tu máquina
- [ ] 2+ beta testers reportaron (aunque sea “funciona en Linux”)
- [ ] Primer comentario fijado con link a `docs/demo.md` o quick start
- [ ] Respuestas preparadas: “necesitas agent CLI instalado”, “es alpha”, “Whisper descarga ~74 MB la primera vez”

---

## Primer comentario (fijar)

Guía rápida + feedback: https://github.com/Jeronimo0228/termvox/blob/main/docs/demo.md

Checklist beta: https://github.com/Jeronimo0228/termvox/blob/main/docs/beta-test-checklist.md
