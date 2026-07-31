# npm installation

TermVox publishes an npm package that downloads the native CLI for your platform
and exposes every command (`shell`, `daemon`, `doctor`, `models`, etc.).

## Install globally (recommended)

```bash
npm install -g termvox
```

During `postinstall`, TermVox:

1. Downloads the matching GitHub Release binary (Linux/macOS/Windows x64/arm64)
2. Verifies SHA-256 when the checksum file is available
3. Runs `termvox models install default` (Whisper tiny, ~74 MiB)
4. Writes `termvox init --preset cursor` when no config exists

Then verify:

```bash
termvox --version
termvox doctor
termvox shell
```

## Project-local install

```bash
npm install termvox
npx termvox doctor
npx termvox shell --agent opencode
```

## Editor extension (Cursor / VS Code)

The npm package bundles a minimal editor extension:

```bash
termvox-editor-install
```

This tries `cursor`, `code`, `codium`, and `code-insiders` in order. Manual
install: **Extensions → Install from Location** →
`$(npm root -g)/termvox/editor` (global) or `node_modules/termvox/editor`.

## Skip bootstrap steps

| Variable | Effect |
| --- | --- |
| `TERMVOX_SKIP_BINARY_INSTALL=1` | Do not download the native binary (CI/packaging) |
| `TERMVOX_SKIP_BOOTSTRAP=1` | Skip Whisper model download and `termvox init` |
| `TERMVOX_NPM_PRESET=opencode` | Preset for first-time init |

See [npm-security.md](npm-security.md) for supply-chain controls.

Example for air-gapped install:

```bash
TERMVOX_SKIP_BINARY_INSTALL=1 npm install -g termvox
# place your own binary at $(npm root -g)/termvox/vendor/termvox
```

## Publish (maintainers)

### First release (manual, one time)

Trusted Publishing **only works after the package exists on npm**. The first
version must be published once with `npm login` + `npm publish` from a
maintainer machine. See [npm trusted publishers](https://docs.npmjs.com/trusted-publishers/).

```bash
npm login
cd packages/termvox
npm publish --access public
```

### Trusted Publishing (GitHub Actions, recommended after first release)

1. Open https://www.npmjs.com/package/termvox → **Settings** → **Trusted publishing**
2. Choose **GitHub Actions** and set:

| Field | Value |
| --- | --- |
| Organization or user | `Jeronimo0228` |
| Repository | `termvox` |
| Workflow filename | `publish-npm.yml` |
| Environment name | *(leave empty)* |
| Allowed actions | `npm publish` |

3. Push a tag `v*` or run the **Publish npm** workflow manually.

The workflow uses OIDC (`id-token: write`) — no `NPM_TOKEN` secret required.

Optional hardening: **Settings → Publishing access → Require 2FA and disallow tokens**
(after Trusted Publishing works).

Local dry-run:

```bash
cd packages/termvox
npm test
npm run build:editor
TERMVOX_SKIP_BINARY_INSTALL=1 npm pack
```

## Requirements

- Node.js 18+
- Supported platform triple (see `packages/termvox/lib/platform.js`)
- Microphone permissions for the terminal or daemon
- At least one supported agent CLI on `PATH` for `termvox shell`

See also [installation.md](installation.md) for source builds and shell script install.
