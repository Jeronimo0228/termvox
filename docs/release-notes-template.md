## TermVox v0.1.0-alpha.14 — launch-ready docs, manual demo guide

Pin: `TERMVOX_VERSION=v0.1.0-alpha.14`

### What's new in alpha.11

- Launch materials: `docs/demo.md`, beta checklist, LinkedIn post draft (ES)
- `scripts/launch-smoke.sh` for pre-post verification
- Removed in-repo video pipeline; record LinkedIn demo manually with OBS

### Upgrade

```bash
npm install -g termvox@latest
# or
TERMVOX_VERSION=v0.1.0-alpha.14 curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
termvox doctor
```
