# npm package security

This document describes supply-chain controls for the `termvox` npm package and
residual risks for installers.

## Threat model

Attackers may try to:

- Publish a malicious package version to npm (account takeover, token leak)
- Redirect binary downloads during `postinstall`
- Ship a trojaned native binary inside release archives
- Abuse lifecycle scripts when TermVox is installed as a dependency
- Trick the editor extension into running an arbitrary executable via settings

## Controls implemented

### npm publishing

- **Trusted Publishing (OIDC)** from GitHub Actions workflow `publish-npm.yml`
- **Provenance attestations** via `publishConfig.provenance`
- Workflow pinned to known action SHAs; **npm CLI pinned** to 11.5.1+
- Editor build uses **`npm ci`** with a committed lockfile

### Install-time binary download

| Control | Detail |
| --- | --- |
| Fixed release origin | Only `Jeronimo0228/termvox` GitHub Releases (no env override) |
| HTTPS only | Plain HTTP blocked |
| Host allowlist | `github.com`, `*.githubusercontent.com` |
| Path validation | Asset names must match `termvox-v{version}-{target}.tar.gz\|zip` |
| Redirect cap | Maximum 5 redirects, each re-validated |
| **Mandatory SHA-256** | Install fails if `.sha256` missing or mismatch |
| Archive hardening | Only `termvox` / `termvox.exe` at archive root is installed; path traversal blocked |

### JavaScript surface

- **Zero runtime npm dependencies** in the published tarball (only Node built-ins)
- `bin/termvox.js` executes only `vendor/termvox` inside the package directory
- Editor extension rejects dangerous characters in `termvox.cliPath`

### Skip flags (CI / air-gapped)

| Variable | Purpose |
| --- | --- |
| `TERMVOX_SKIP_BINARY_INSTALL=1` | Skip network download (packaging pipelines) |
| `TERMVOX_SKIP_BOOTSTRAP=1` | Skip Whisper model + config init |

## Residual risks

1. **npm account compromise** — Mitigate with Trusted Publishing, disable long-lived tokens, enable 2FA, restrict publishing access on npmjs.com.
2. **Compromised GitHub release** — Mitigate with signed tags, protected `main`, release workflow review, cosign bundles on native archives (see `scripts/verify-release.sh`).
3. **postinstall on dependency install** — Installing `termvox` inside an untrusted project runs `postinstall`. Prefer `npm install -g termvox` or audit lockfiles; use `npm install --ignore-scripts` if you only need the JS shim without downloading binaries.
4. **Native binary trust** — SHA-256 verifies integrity of the archive from GitHub, not authorship. Verify Sigstore/cosign bundles for stronger guarantees (future npm installer enhancement).
5. **Editor `cliPath`** — Workspace settings can point to another binary path (by design). Only trust workspace settings from repositories you control.

## Maintainer checklist before each npm release

- [ ] GitHub release assets and `.sha256` companions present
- [ ] `npm test --prefix packages/termvox` passes
- [ ] Publish only via tagged `publish-npm.yml` workflow (OIDC)
- [ ] Confirm provenance on npm package page after publish
- [ ] Bump npm version; republish cannot overwrite an existing version

## Reporting

See [SECURITY.md](../SECURITY.md) in the repository root.
