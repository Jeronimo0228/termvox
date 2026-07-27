# Plugin system

Official agent integrations implement `AgentAdapter` inside
`termvox-agents`. The extension boundary is designed as a child process
speaking newline-delimited JSON-RPC 2.0 on standard input and output.

> **Current status:** configured plugins can be listed, initialized, inspected,
> and probed. They are not selectable as the coding agent for `termvox start`.
> Protocol version `1` remains alpha-level and may change.

## Configuration

TermVox never discovers plugins automatically. Add an explicit entry:

```toml
[[plugins]]
id = "example"
executable = "/usr/bin/python3"
args = ["/absolute/path/to/example_plugin.py"]
enabled = true
env_allowlist = []
timeout_seconds = 30
max_frame_bytes = 1048576
```

Then run:

```bash
termvox plugins list
termvox plugins inspect example
termvox plugins test example
```

The child starts in a per-plugin local data directory. Its environment is
cleared, then only variables named in `env_allowlist` are copied. Use an
absolute native executable path. Scripts with `#!/usr/bin/env ...` need `PATH`
in the allowlist, or can be passed as an argument to an absolute interpreter as
shown above. Adapt the interpreter path on Windows.

## Handshake

The client starts an executable and arguments supplied by its caller, then
sends:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocol_version":1}}
```

The response result must be a manifest containing `id`, `name`, `version`,
`protocol_version`, and capability flags. A version mismatch terminates
initialization:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "id": "example",
    "name": "Example Agent",
    "version": "0.1.0",
    "protocol_version": 1,
    "capabilities": {
      "streaming": false,
      "resume": false,
      "cancellation": false
    }
  }
}
```

Each request is one UTF-8 JSON object followed by a newline. Each response must
use the same numeric `id` and contain a `result` or an `error` with numeric
`code` and string `message`. Requests and responses exceeding the configured
frame size fail, and each call has a timeout.

## Lifecycle

| Method | Parameters | Result |
| --- | --- | --- |
| `initialize` | `{ "protocol_version": 1 }` | Plugin manifest |
| `probe` | `null` | Plugin-defined diagnostic JSON |
| `start` | `null` | `{ "session_id": "..." }` |
| `send` | `session_id`, `prompt`, `cwd` | Plugin-defined JSON |
| `cancel` | `{ "session_id": "..." }` | Acknowledgement |
| `shutdown` | `null` | Acknowledgement |

The SDK provides matching Rust types, a `PluginHandler` trait, and a `serve`
loop. A plugin with `streaming = true` may write `event` notifications:

```json
{
  "jsonrpc": "2.0",
  "method": "event",
  "params": {
    "session_id": "example-session",
    "event": {"type": "message", "text": "working"}
  }
}
```

The client buffers recognized event notifications while waiting for a response.
Unknown response IDs are skipped. Capability flags describe plugin behavior;
the caller must still choose which lifecycle methods to use.

## Trust model

- There is no filesystem auto-discovery or automatic execution.
- The executable path and arguments must be supplied explicitly.
- Standard output is reserved for protocol frames; diagnostics use stderr.
- The child starts with a cleared environment and an explicit working
  directory, but inherits the invoking user's filesystem and network authority.
- There is no plugin registry, package signature scheme, or verified installer.
- Review source and pin executable hashes before running third-party plugins.

The Rust crate exposes `PluginClient` and manifest types; it deliberately does
not expose a dynamic-library ABI. See
[`examples/example_plugin.py`](https://github.com/Jeronimo0228/termvox/blob/main/examples/example_plugin.py)
for an educational fixture, not a declaration that CLI plugin loading is
complete.
