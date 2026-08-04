#!/usr/bin/env bash
# Happy-path smoke checks before a public launch post (non-interactive).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== launch smoke: build =="
cargo build --release -p termvox --locked -q
export PATH="$ROOT/target/release:${PATH:-}"

echo "== launch smoke: version =="
VERSION="$(termvox --version)"
echo "$VERSION"
case "$VERSION" in
  *alpha.1[0-9]*|*alpha.[2-9][0-9]*) ;;
  *)
    echo "warning: expected alpha.10+ for launch; got $VERSION"
    ;;
esac

echo "== launch smoke: doctor json =="
termvox doctor --json | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert 'configuration' in d, 'missing configuration'
assert 'hints' in d, 'missing hints'
mic = d.get('microphone', {})
if not mic.get('ok'):
    raise SystemExit('microphone check failed — fix before launch post')
speech = d.get('speech', {})
if not speech.get('ok'):
    raise SystemExit('speech/whisper check failed — run: termvox models install default')
agents = d.get('agents', [])
ok_agents = [a for a in agents if a.get('installed') and a.get('auth', {}).get('ok', True)]
if not ok_agents:
    print('warning: no fully-ready agent CLI — demo should use an installed agent')
else:
    print('ready agents:', ', '.join(a['id'] for a in ok_agents))
"

echo "== launch smoke: config validate =="
termvox config validate

echo "== launch smoke: npm pack =="
if command -v npm >/dev/null 2>&1; then
  npm test --prefix packages/termvox --silent
  (cd packages/termvox && TERMVOX_SKIP_BINARY_INSTALL=1 npm pack >/dev/null)
  echo "npm pack: OK"
else
  echo "npm not installed — skipping npm pack"
fi

echo
echo "launch smoke: OK (interactive termvox shell not exercised — record demo manually)"
