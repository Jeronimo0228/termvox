## TermVox v0.1.0-alpha.16 — STT docs, rich CLI help, demo tooling

Pin: `TERMVOX_VERSION=v0.1.0-alpha.16`

### What's new in alpha.16

- STT documentation: performance profiles, VAD tuning, model install, Spanish guide
- Richer CLI `--help` / `termvox manpage` (shell, models, config, doctor, STT tips)
- LinkedIn demo render script: `scripts/render-linkedin-demo.py`

### Upgrade

```bash
npm install -g termvox@latest
# or
TERMVOX_VERSION=v0.1.0-alpha.16 curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
termvox doctor
```
