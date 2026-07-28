# Frequently asked questions

## Is TermVox production-ready?

No. The workspace is an alpha release and its CLI, configuration, adapters, and
plugin protocol can change.

## Where can I download a binary?

This documentation does not claim that binaries are published. Build from
source unless the canonical repository later publishes release instructions
with verifiable artifacts.

## Are release artifacts signed?

No signing claim is made. Signing, checksums, provenance, or SBOMs must be
verified from an actual release before relying on them.

## Which agents work now?

Codex CLI, Claude Code, Cursor CLI, Gemini CLI, Aider, and Amp have built-in
adapters. Actual compatibility depends on the installed upstream version.

## Which speech engines work now?

Whisper.cpp and OpenAI have direct adapters. Parakeet and Vosk use a generic
local sidecar contract; TermVox does not bundle their executable or model.

## Does local transcription mean TermVox is fully offline?

No. Whisper.cpp can transcribe locally, but the selected coding agent may use
network services. Installation and model acquisition may also require network
access.

## Does TermVox store recordings or transcripts?

The current source does not intentionally keep history. Whisper.cpp uses
temporary files that are deleted on handled paths, and OpenAI mode uploads
audio. Terminal scrollback, crash remnants, agents, and providers can still
retain data.

## Can TermVox prevent an agent from running dangerous commands?

No. It previews prompts, requires confirmation by default, avoids adding broad
auto-approval flags, and warns on a small phrase list. The coding agent's own
sandbox and authorization policy remain the enforcement layer.

## Can I disable confirmation?

Setting both `confirmation = false` and `auto_send = true` removes routine
confirmation. Prompts matching built-in risk signals still require it. This
mode is not recommended.

## Can I use a global hotkey?

TermVox itself currently reads keys from its focused terminal. You can bind an
OS or window-manager shortcut to `termvox record start`, `stop`, or `toggle`,
but TermVox does not install a global hook.

## Can I configure a different microphone?

Yes. Set `audio.device` to an exact input-device name reported by
`termvox doctor`. Omit it to use the default input.

## Can I write a plugin?

Yes, experimentally. The CLI can initialize, inspect, and probe explicitly
configured JSON-RPC plugins. It cannot yet select a plugin as the interactive
coding agent, and the protocol remains unstable.

## Does one interactive run preserve the agent session?

Yes. `termvox start` keeps one local session and structured adapters can reuse a
remote session ID across utterances. With `[workspace].persist_session = true`
(default), that upstream id is also saved under `.termvox/session.json` in the
project so the next `termvox start` or `termvox shell` in the same directory can
resume the chat. Separate `termvox record` processes do not share that session.

## Where do I ask for help or report a vulnerability?

Use the
[support policy](https://github.com/Jeronimo0228/termvox/blob/main/SUPPORT.md)
for public help and the
[security policy](https://github.com/Jeronimo0228/termvox/blob/main/SECURITY.md)
for private vulnerability reporting.
