# Alex Windows Codex Checklist

Status before this checklist:

> Codex provider implemented and fake-tested. Linux Codex smoke tested if
> available. Windows login/runtime verification pending Alex after Windows
> build.

Do not mark Windows Codex auth as verified until this checklist passes on the
Windows meeting device.

## 1. Install Or Start ClawScribe

Install the current ClawScribe Windows build, or run a developer build from the
repo:

```powershell
cd <clawscribe-repo>\frontend
pnpm install --frozen-lockfile
pnpm tauri dev
```

If using the installer, launch `ClawScribe` from the Start menu.

Record:

```text
ClawScribe commit:
Installer/artifact name:
Windows version:
Codex version:
Verification date:
```

## 2. Open AI Provider Settings

1. Open **Settings**.
2. Open **AI Provider** / **Model Settings**.
3. Select **Codex / ChatGPT login**.

Expected:

- The provider is labeled `Codex / ChatGPT login`.
- The panel says `OpenAI login via Codex`.
- It does not call this generic OpenAI OAuth.
- It shows controls for:
  - Check Codex installation
  - CODEX_HOME mode
  - Sign in with OpenAI via Codex
  - Sign in with device code
  - Use existing Codex session
  - Test OpenAI/Codex processing
  - Logout / clear Codex auth

## 3. Check Codex Installation

Click **Check Codex installation**.

Expected if Codex is installed:

- Codex found: yes.
- Codex path and version are shown.
- CODEX_HOME mode/path are shown.

If Codex is missing, install Codex using OpenAI's supported installer or install
instructions for Windows, then reopen ClawScribe and retry:

```powershell
codex --version
```

If Codex is installed but not on `PATH`, set the Codex binary path in
ClawScribe settings and retry **Check Codex installation**.

## 4. Use Isolated CODEX_HOME First

Set CODEX_HOME mode to:

```text
ClawScribe isolated
```

Expected isolated path:

```text
%APPDATA%\ClawScribe\codex
```

This keeps ClawScribe's Codex login separate from the user's normal `~\.codex`
profile.

## 5. Browser Login

Click:

```text
Sign in with OpenAI via Codex
```

Expected:

- Codex opens its normal browser login flow.
- Complete login in the browser.
- ClawScribe reports the Codex command completed or gives a human-readable
  Codex error.
- No access token, refresh token, auth code, API key, `auth.json` content, or
  full command environment appears in the UI or logs.

After login, click **Check Codex installation** again.

Expected:

- The Codex status is usable/authenticated, or at least no longer reports that
  login is missing.

## 6. Device-Code Login Fallback

If browser login fails, click:

```text
Sign in with device code
```

Expected:

- ClawScribe shows Codex's device-code instructions/output.
- Complete the device-code flow in the browser.
- ClawScribe reports success or a clear Codex error.
- **Test OpenAI/Codex processing** can run afterward.

## 7. Test Processing

Click:

```text
Test OpenAI/Codex processing
```

Expected:

- ClawScribe creates a temporary scratch workspace.
- Codex runs a small `codex exec` test.
- Success is reported only if Codex returns:

```text
CLASCRIBE_CODEX_OK
```

Record the result:

```text
Test processing result:
Any error text:
```

## 8. Process A Tiny Synthetic Transcript

Create or import a tiny test meeting/transcript with content like:

```text
[00:01] Alex: We will verify Codex processing on Windows today.
[00:05] Nora: Action item for Nora: document the Windows Codex result.
```

Use **Codex / ChatGPT login** as the summary provider and generate notes.

Expected meeting output files:

- `meeting-output.json`
- `meeting-notes.md`
- `follow-up-email.md`
- `processing-log.json`

Expected Codex run folder under:

```text
%LOCALAPPDATA%\ClawScribe\codex-runs\<meeting-id>\
```

Expected run files:

- `transcript.md`
- `metadata.json`
- `output-schema.json`
- `prompt.md`
- `codex-output.json`
- `codex-final.md`
- `codex-events.jsonl`

Validate the structured JSON:

```powershell
$json = Get-Content "<meeting-folder>\meeting-output.json" -Raw | ConvertFrom-Json
$json.executive_summary
$json.action_items
```

Expected:

- JSON parses.
- Action items do not invent owners or due dates.
- Unknown owners/due dates are `null`.
- Source timestamps are included when available.

## 9. Process One Real Recording

1. Record a short real ClawScribe/Meetily meeting with spoken content.
2. Stop recording.
3. Wait for transcription to complete.
4. Generate notes using **Codex / ChatGPT login**.

Expected:

- Transcript content is non-empty.
- `meeting-output.json` is valid JSON.
- `meeting-notes.md` contains a useful summary.
- `follow-up-email.md` is present.
- `processing-log.json` reports the Codex provider and sanitized command status.

## 10. Secret Hygiene Check

Run this in PowerShell:

```powershell
$paths = @(
  "$env:LOCALAPPDATA\ClawScribe",
  "$env:APPDATA\ClawScribe"
)
foreach ($path in $paths) {
  if (Test-Path $path) {
    Select-String -Path "$path\**\*.json","$path\**\*.jsonl","$path\**\*.log","$path\**\*.md" `
      -Pattern "sk-proj-|sk-|refresh_token|access_token|Authorization: Bearer|auth.json" `
      -CaseSensitive:$false -ErrorAction SilentlyContinue
  }
}
```

Expected:

- No raw OpenAI/Codex access tokens.
- No refresh tokens.
- No API keys.
- No full `Authorization: Bearer ...` headers.
- Redacted placeholders such as `[REDACTED]` are allowed.

Do not paste auth files, tokens, or screenshots containing credentials into
chat or issues.

## 11. Provider Switching

Verify switching still works:

1. Select **Codex / ChatGPT login** and run **Test OpenAI/Codex processing**.
2. Select **OpenAI API Key**, save settings, and confirm the UI asks for an API
   key instead of Codex login.
3. Select **OpenClaw Gateway**, save settings, and confirm the OpenClaw
   endpoint/bearer-token fields are shown.
4. Switch back to **Codex / ChatGPT login** and confirm Codex settings are still
   present.

Expected:

- No provider switch exposes saved secrets.
- API-key mode does not require Codex.
- Codex mode does not require OpenClaw.
- OpenClaw mode does not require Codex tokens in ClawScribe.

## 12. Logout

With CODEX_HOME mode set to **ClawScribe isolated**, click:

```text
Logout / clear Codex auth
```

Expected:

- ClawScribe invokes the supported Codex logout behavior.
- Only ClawScribe-owned isolated Codex auth/cache is cleared.
- The user's normal `~\.codex` profile is not deleted.

Confirm:

```powershell
dir "$env:APPDATA\ClawScribe\codex"
dir "$env:USERPROFILE\.codex"
```

Do not delete the user's global `~\.codex` unless explicitly testing existing
user session cleanup and you understand the consequence.

## Pass/Fail Statement

Use this exact wording until all steps pass:

```text
Codex provider implemented and fake-tested. Linux Codex smoke tested if
available. Windows login/runtime verification pending Alex after Windows build.
```

Use this only after all steps pass:

```text
Windows Codex login/runtime verification passed on <date> using ClawScribe
commit <sha> and Codex <version>.
```
