# ClawScribe

ClawScribe is a local-first desktop meeting capture app for Windows-focused OpenClaw workflows. It records meetings from the user session, produces local transcripts and summaries, and can hand completed meeting artifacts to OpenClaw for post-processing and notification.

Current product version: `0.5.0-alpha.1`

ClawScribe is based on Meetily Community Edition `0.4.0`. The upstream Meetily project is copyright Zackriya Solutions and contributors and is distributed under the MIT License.

## What It Does

- Records microphone and system audio from the desktop user session.
- Transcribes meetings locally with bundled/on-device transcription models.
- Generates summaries with local models or configured AI providers.
- Supports optional OpenClaw handoff for completed recording folders.
- Avoids visible meeting bots and Microsoft tenant-admin meeting API dependencies.
- Keeps compatibility with existing Meetily-format artifacts where required.

## Status

This fork is in productization. Treat `0.5.0-alpha.1` as an alpha build for validation, not a finished release.

Known productization boundaries:

- Windows runtime validation should be performed on a Windows host before release.
- Default recording folder compatibility with existing `meetily-recordings` installs is intentionally preserved for now.
- OpenClaw handoff marker schemas currently retain `openclaw.meetily-submission*.v1` for compatibility.
- Some upstream build, model, and migration paths still contain Meetily names by design.

## Development

Frontend and Tauri app sources live under `frontend/`.

```bash
cd frontend
pnpm install
pnpm tauri:dev
```

Useful build scripts are also in `frontend/`, including `build-gpu.*`, `dev-gpu.*`, and `scripts/build-windows-release.ps1`.

Do not trigger a Windows installer build for every code change. Build installers at explicit release or validation checkpoints.

## OpenClaw Handoff

See [docs/openclaw-handoff.md](docs/openclaw-handoff.md) for the current OpenClaw configuration flow.

Compatibility note: `MEETILY_OPENCLAW_*` environment variables and Meetily-format recording layouts are currently retained so existing deployments and imported artifacts keep working.

## Upstream

ClawScribe is a fork of Meetily Community Edition. For upstream provenance and attribution, see [UPSTREAM.md](UPSTREAM.md) and [NOTICE.md](NOTICE.md).

Upstream project:

- https://github.com/Zackriya-Solutions/meeting-minutes
- https://github.com/Zackriya-Solutions/meetily

## License

This fork remains under the MIT License. See [LICENSE.md](LICENSE.md).

Copyright for the upstream Meetily code remains with Zackriya Solutions and contributors. ClawScribe changes are copyright OpenClaw contributors unless otherwise noted.
