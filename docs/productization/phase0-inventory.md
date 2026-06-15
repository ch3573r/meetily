# ClawScribe Productization Phase 0 Inventory

Date: 2026-06-15
Branch: `feat/clawscribe-productization-auth-theme-exports`
Baseline commit: `70628e0fc34dc8032bb42cc016180a312c28f20f`

## Version Baseline

- `frontend/package.json`: package name `clawscribe`, version `0.4.0`
- `frontend/src-tauri/tauri.conf.json`: product name `ClawScribe`, version `0.4.0`, identifier `net.rismondo.openclaw.clawscribe`
- `frontend/src-tauri/Cargo.toml`: crate name `clawscribe`, version `0.4.0`, repository `https://github.com/Zackriya-Solutions/meetily`
- About dialog: `frontend/src/components/About.tsx` defaults `currentVersion` to `0.4.0` and reads runtime Tauri version
- Sidebar footer: `frontend/src/components/Sidebar/index.tsx` displays `v0.4.0`
- Analytics UI/constants: `frontend/src/components/AnalyticsProvider.tsx`, `AnalyticsConsentSwitch.tsx`, and `AnalyticsDataModal.tsx` still contain `0.4.0`
- Windows installer metadata is partly ClawScribe-branded in Tauri config, with publisher/manufacturer `OpenClaw`

## App Name String Baseline

Current naming is mixed.

ClawScribe already appears in:
- Tauri product/window/installer metadata
- `frontend/package.json` package name
- `docs/windows-release.md`
- `docs/openclaw-handoff.md`
- OpenClaw provider source defaults
- Teams detection debug bridge names

Meetily remains in visible/product-facing paths including:
- README and multiple build/GPU docs
- About dialog text and external Zackriya links
- Logo/about dialog titles
- Onboarding copy
- Sidebar/logo display
- Notifications and tray tooltip
- Metadata title files
- Default recording folder names: `meetily-recordings`
- Template directory names and docs
- IndexedDB/recovery database names

Likely intentional/provenance uses include:
- Upstream license/copyright/provenance references
- Meetily-format artifact layout names such as `meetily-json-v1`
- Backward-compatible OpenClaw marker schemas such as `openclaw.meetily-submission.v1`
- Backward-compatible legacy import paths and previous-install migration text

## Meeting Detection Baseline

Current Teams detection is implemented as a read-only Windows detector in `frontend/src-tauri/src/teams_detection.rs` and frontend service `frontend/src/services/teamsDetectionService.ts`.

Observed implementation:
- Windows-only support path; non-Windows returns unsupported
- Process/window-title heuristics for Teams desktop and Teams-in-browser
- Confidence scoring with threshold default `0.65`
- Requires meeting-title signal by default
- Candidate signals include Teams process, browser process, meeting-like title, browser Teams title, and foreground title
- Output includes diagnostics, confidence, candidates, reason, and prompt-only recording safety
- It does not auto-start or auto-stop recording
- It recommends `promptToRecord` only; `recordingSafety.automaticRecordingAllowed=false`
- Existing docs: `docs/teams-detection.md`

Verification status:
- Code-level checks have existed previously, but no Windows runtime Teams verification has been performed from this Linux host.
- Do not claim runtime verification until Alex or a Windows runner/device executes the Windows checklist.

## Auth / Provider Baseline

Current OpenAI/provider implementation:
- Direct OpenAI API key provider exists.
- Custom OpenAI-compatible endpoint provider exists.
- Optional OpenClaw Gateway provider exists and must remain backward compatible.
- `frontend/src-tauri/src/openai/auth.rs` contains OAuth PKCE metadata scaffolding, but explicitly marks public OpenAI OAuth PKCE as unsupported for authenticating OpenAI API requests.
- Current UI tells users to use an OpenAI API key; there is no working "Sign in with OpenAI" UX.
- API keys and custom endpoint tokens are currently stored through the existing settings repository path, not proven OS credential storage.
- OpenClaw handoff config currently uses `openclaw.json` with bearer token preservation behavior; this is working against the OpenClaw ingest service and must not be broken.

Security baseline:
- Existing code avoids logging full OpenAI API keys in the normal OpenAI model fetch path.
- There are still old/native API paths that log token presence and one helper logs a truncated auth token; these need review before claiming secret hygiene.
- No OpenAI/ChatGPT cookies or web UI automation are present in the ClawScribe implementation.

## Microsoft Graph Export Baseline

No first-party Microsoft Graph export implementation was found in ClawScribe.

Observed state:
- No MSAL dependency in frontend package or Rust crate.
- No OneNote export implementation.
- No Planner export implementation.
- Existing Teams detection intentionally avoids Graph APIs, tenant permissions, Teams bots, and meeting APIs.

## OpenClaw Handoff Baseline

Working OpenClaw behavior must remain compatible:
- Config commands: `get_openclaw_config_status`, `save_openclaw_config`, `submit_meeting_folder_to_openclaw`, `get_openclaw_submission_status`
- Config file: `openclaw.json`
- Success/failure markers in recording folders: `.openclaw-pending.json`, `.openclaw-submitted.json`, `.openclaw-failed.json`
- Existing schema strings use `openclaw.meetily-submission*.v1`
- Current default recording folder is still `meetily-recordings`
- OpenClaw ingest has successfully processed a ClawScribe handoff from Windows on 2026-06-15, but the submitted transcript content was effectively empty

## Known Baseline Validation Gaps

- No Windows runtime verification is available from this Linux host.
- Full frontend `tsc --noEmit` has previously been blocked by existing missing `bun:test` typings in `tests/lib/blocknote-markdown.test.ts`.
- `pnpm lint` has previously been unusable because `next lint` prompts for ESLint setup.
- Full Linux Tauri validation may require staged sidecars and can consume substantial disk.

## Coordination Notes

- Do not trigger a Windows installer build for every change. Batch changes and build only at deliberate release/test checkpoints or when Alex asks.
- Do not leak OpenAI, Microsoft, OpenClaw, Graph, transcript, or bearer secrets into logs, screenshots, commits, docs, or chat.
- Workers should use disjoint write scopes and report exact files changed.
