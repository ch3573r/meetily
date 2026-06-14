# OpenClaw Handoff

ClawScribe can submit completed Meetily-format recording folders directly to
the OpenClaw meeting ingest endpoint. The handoff runs after recording stops
and final `transcripts.json` and `metadata.json` artifacts have been written.

## Configuration

ClawScribe reads this JSON file from its Tauri app config directory:

```json
{
  "enabled": true,
  "endpoint": "http://127.0.0.1:8765/meetings/completed",
  "bearer_token": "replace-me",
  "source": "ClawScribe",
  "include_audio_path": false
}
```

The same values can be overridden with environment variables:

- `MEETILY_OPENCLAW_ENABLED`
- `MEETILY_OPENCLAW_ENDPOINT`
- `MEETILY_OPENCLAW_BEARER_TOKEN`
- `MEETILY_OPENCLAW_SOURCE`

The `MEETILY_OPENCLAW_*` names are intentionally retained as implementation
keys for reset-safe compatibility with the fork's current handoff code. Rename
package identifiers, config filenames, or environment variables only as a
coordinated build/install change.

## Behavior

When a recording stops, ClawScribe builds a `meeting.completed` payload from the
recording folder and posts it to the configured endpoint with bearer auth.

On success, ClawScribe writes `.openclaw-submitted.json` into the recording
folder. On failure, it writes `.openclaw-failed.json`.

The companion Windows tray/agent is no longer needed for the happy path once
this fork is used as the recorder. It can remain as a diagnostic/manual
processor while the fork-native handoff is being hardened.
