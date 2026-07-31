## TermVox v0.1.0-alpha.10 — CI green, npm latest, docs refresh

Pin: `TERMVOX_VERSION=v0.1.0-alpha.10`

### What's new in alpha.10

- npm `latest` dist-tag points at hardened alpha.10 (Trusted Publishing / OIDC)
- CI lockfile and rustfmt fixes; all workflows passing on tag push
- README and documentation updated for npm-first install and current release artifacts

### Upgrade

```bash
npm install -g termvox@latest
# or
TERMVOX_VERSION=v0.1.0-alpha.10 curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash
termvox doctor
```
