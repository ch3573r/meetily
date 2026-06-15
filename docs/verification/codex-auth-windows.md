# Codex Auth Windows Verification

Status: implemented with fake-Codex tests on Linux. Real Windows Codex login and
Windows credential-store behavior are pending runtime verification.

Do not report `Codex login verified on Windows` until this checklist has been
run on a Windows machine with Codex installed and a real OpenAI/ChatGPT login.

For the Alex-facing release gate, use
`docs/verification/alex-windows-codex-checklist.md`. This file keeps the
lower-level command and behavior details.

## 1. Install Or Locate Codex

Open PowerShell:

```powershell
codex --version
codex login status
```

Expected:

- `codex --version` prints a Codex CLI version.
- `codex login status` prints a safe login state or says login is needed.

If Codex is not on `PATH`, note the full binary path. ClawScribe can use a
configured Codex binary path.

## 2. Open ClawScribe Settings

1. Launch ClawScribe.
2. Open Settings.
3. Open AI Provider / Model Settings.
4. Select `Codex / ChatGPT login`.
5. Confirm the label says `OpenAI login via Codex`, not generic OpenAI OAuth.

## 3. Check Installation

Click `Check Codex installation`.

Expected:

- Codex found: yes.
- Version is shown.
- Path is shown.
- CODEX_HOME is shown.
- Auth status is shown if Codex can determine it.

Default CODEX_HOME should be:

```text
%APPDATA%\ClawScribe\codex
```

## 4. Browser Login

With CODEX_HOME mode set to `ClawScribe isolated`, click:

```text
Sign in with OpenAI via Codex
```

Expected:

- Codex opens its normal browser login flow.
- No token values appear in ClawScribe UI or logs.
- After login, `Check Codex installation` reports an authenticated or usable
  Codex status.

Inspect:

```powershell
dir "$env:APPDATA\ClawScribe\codex"
```

Auth files may exist there. Do not share their contents.

## 5. Device-Code Login

Click:

```text
Sign in with device code
```

Expected:

- ClawScribe shows Codex's device-code instructions/output.
- Complete the device-code flow in the browser.
- The command exits successfully.
- `Check Codex installation` reports usable status.

## 6. Test Processing

Click:

```text
Test OpenAI/Codex processing
```

Expected:

- ClawScribe runs a scratch `codex exec`.
- Prompt expects exactly `CLASCRIBE_CODEX_OK`.
- UI reports success only when that response is present.

## 7. Process A Tiny Transcript

Create or import a short test meeting with transcript text similar to:

```text
[00:01] Alex: We will ship the Codex provider after Windows verification.
[00:05] Nora: Action item for Nora: run fake-Codex and Windows checks.
```

Generate/process the summary using provider `Codex / ChatGPT login`.

Expected output files in the meeting folder:

- `meeting-output.json`
- `meeting-notes.md`
- `follow-up-email.md`
- `processing-log.json`

Expected output files under the run folder:

- `transcript.md`
- `metadata.json`
- `output-schema.json`
- `prompt.md`
- `codex-output.json`
- `codex-final.md`
- `codex-events.jsonl`

Validate JSON:

```powershell
$json = Get-Content .\meeting-output.json -Raw | ConvertFrom-Json
$json.executive_summary
$json.action_items
```

Expected:

- JSON parses.
- Summary does not invent owners or due dates.
- Unknown owners/due dates are `null`.
- Timestamps are included when available.

## 8. Existing User Codex Session

Switch CODEX_HOME mode to:

```text
Existing user Codex session
```

Save settings, then click `Check Codex installation` and `Test OpenAI/Codex
processing`.

Expected:

- ClawScribe does not use `%APPDATA%\ClawScribe\codex`.
- Codex uses the normal user Codex session.
- No global `~\.codex` files are deleted.

## 9. Logout

Switch back to `ClawScribe isolated`, then click:

```text
Logout / clear Codex auth
```

Expected:

- ClawScribe invokes `codex logout` with isolated CODEX_HOME.
- Isolated Codex auth is cleared.
- User global Codex auth remains untouched.

## 10. Safe Logs

Search logs and generated processing files for secret-like material:

```powershell
Select-String -Path "$env:LOCALAPPDATA\ClawScribe\**\*.json","$env:LOCALAPPDATA\ClawScribe\**\*.jsonl" `
  -Pattern "sk-proj-|sk-|refresh_token|access_token|Authorization: Bearer" `
  -CaseSensitive:$false
```

Expected:

- No raw secrets.
- Redacted entries may appear as `[REDACTED]`.

## 11. OpenClaw Handoff

Keep OpenClaw handoff enabled if it was previously configured, then process a
short recording.

Expected:

- Codex summary generation works independently.
- OpenClaw handoff still submits completed recordings when enabled.
- No Codex tokens are sent to OpenClaw.
