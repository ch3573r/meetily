# Codex Auth

ClawScribe supports OpenAI login through Codex. This is Codex-managed
ChatGPT/OpenAI sign-in, not a generic OAuth client inside ClawScribe.

ClawScribe does not implement private OpenAI OAuth endpoints, scrape ChatGPT
cookies, or extract Codex access or refresh tokens. The boundary is the Codex
runtime:

- `codex login`
- `codex login --device-auth`
- `codex login status`
- `codex logout`
- `codex exec`

API-key mode remains available separately as the `OpenAI API Key` provider.
OpenClaw Gateway mode remains available separately as the `OpenClaw` provider
and existing OpenClaw handoff is unchanged.

## Provider Shape

Codex is one processing backend, not the only backend:

```json
{
  "processing": {
    "provider": "codex",
    "codex": {
      "codexHomeMode": "clawscribe-isolated",
      "codexHomePath": "%APPDATA%\\ClawScribe\\codex",
      "useExistingUserCodexSession": false,
      "model": "gpt-5.1-codex",
      "timeoutSeconds": 600
    }
  }
}
```

The existing direct API-key and OpenClaw paths are preserved:

```json
{ "processing": { "provider": "api-key" } }
```

```json
{ "processing": { "provider": "openclaw" } }
```

## Codex Discovery

ClawScribe discovers Codex in this order:

1. Bundled Codex binary in app resources.
2. User-configured Codex binary path.
3. `codex` on `PATH`.
4. A clear UI error with install instructions.

ClawScribe does not silently install Codex.

## CODEX_HOME

The default mode is isolated:

```text
%APPDATA%\ClawScribe\codex
```

In isolated mode, ClawScribe creates the directory if needed, writes a minimal
`config.toml`, sets `CODEX_HOME` only for Codex child processes, and treats any
file-backed auth material as secret.

The user may explicitly choose `Existing user Codex session`. In that mode,
ClawScribe removes `CODEX_HOME` from the Codex child process environment so
Codex uses the normal user profile instead of an inherited ClawScribe override.

Never share:

- `auth.json`
- access tokens
- refresh tokens
- API keys
- bearer tokens
- full command environments

Safe to share:

- Codex version
- Codex binary path
- selected CODEX_HOME path
- sanitized stderr/stdout
- processing-log excerpts after redaction

## Processing Flow

```text
recording finished
 -> transcript normalized
 -> CodexProcessingProvider invoked
 -> codex exec runs with prompt + transcript
 -> structured JSON validated
 -> meeting-output.json written
 -> meeting-notes.md written
 -> follow-up-email.md written
 -> OpenClaw handoff remains available
```

Codex run folders are created under:

```text
%LOCALAPPDATA%\ClawScribe\codex-runs\<meeting-id>\
```

Each run writes:

- `transcript.md`
- `metadata.json`
- `output-schema.json`
- `prompt.md`
- `codex-output.json`
- `codex-final.md`
- `codex-events.jsonl`

Meeting output folders receive:

- `meeting-output.json`
- `meeting-notes.md`
- `follow-up-email.md`
- `processing-log.json`

## Settings

Open Settings -> AI Provider and choose `Codex / ChatGPT login`.

Available controls:

- Check Codex installation
- CODEX_HOME mode
- Sign in with OpenAI via Codex
- Sign in with device code
- Use existing Codex session
- Use OpenAI API key instead, by selecting `OpenAI API Key`
- Test OpenAI/Codex processing
- Logout / clear Codex auth

## Verification Status Wording

Use this wording until Alex completes the Windows checklist:

```text
Codex provider implemented and fake-tested. Linux Codex smoke tested if
available. Windows login/runtime verification pending Alex after Windows build.
```

Do not claim Windows Codex login/runtime verification from Linux checks.
Windows verification requires `docs/verification/alex-windows-codex-checklist.md`
to pass on the Windows meeting device.

## Logout

ClawScribe first uses the supported Codex command:

```powershell
codex logout
```

For isolated mode, this operates on ClawScribe's `CODEX_HOME`. ClawScribe must
not delete the user's global `~\.codex` unless the user explicitly selected the
existing user session mode and confirmed that ClawScribe should manage it.
