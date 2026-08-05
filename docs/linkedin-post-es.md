# Borrador — post LinkedIn (español)

> **Cuándo publicar:** después de grabar el video (ver [demo.md](demo.md)) y
> recibir feedback de al menos 2 beta testers.

---

## Texto sugerido

(El video es solo demo en terminal — este texto va en el post, no en pantalla.)

Estoy lanzando **TermVox** en alpha: voz local para coding agents en la terminal
(Cursor, OpenCode, Claude Code, Codex…).

Hablas → **Whisper** on-device → confirmas → el agente lo recibe en el mismo TUI
(`termvox shell`, F8 / Ctrl+Space).

```bash
npm install -g termvox
termvox shell --agent opencode
```

Alpha temprana — busco early adopters.

Repo: https://github.com/Jeronimo0228/termvox  
npm: https://www.npmjs.com/package/termvox

#BuildInPublic #OpenSource #Rust #DeveloperTools #AI #Voice #CLI #Cursor #OpenCode

---

## Variante corta (si LinkedIn trunca)

**TermVox** = voz local (Whisper) + confirmación antes de enviar → agentes en terminal (Cursor, OpenCode, Claude…).

Alpha preview. `npm install -g termvox && termvox shell`

https://github.com/Jeronimo0228/termvox

#BuildInPublic #Rust #CLI

---

## Checklist antes de publicar

- [ ] Video de 45–90 s grabado con OBS (ver [demo.md](demo.md))
- [ ] Cut práctico (sin texto en video) con `scripts/render-linkedin-demo.py` → **1080p**
- [ ] Probaste el happy path hoy en tu máquina
- [ ] 2+ beta testers reportaron (aunque sea “funciona en Linux”)
- [ ] Primer comentario fijado con link a `docs/demo.md` o quick start
- [ ] Respuestas preparadas: “necesitas agent CLI instalado”, “es alpha”, “Whisper descarga ~74 MB la primera vez”

---

## Primer comentario (fijar)

Guía rápida + feedback: https://github.com/Jeronimo0228/termvox/blob/main/docs/demo.md

Checklist beta: https://github.com/Jeronimo0228/termvox/blob/main/docs/beta-test-checklist.md
