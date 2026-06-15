# ClawScribe Productization Coordinator Plan

Branch: `feat/clawscribe-productization-auth-theme-exports`
Baseline: `70628e0fc34dc8032bb42cc016180a312c28f20f`

## Worker Scopes

- Auth: OpenAI login research and docs, with backend-only patches if a supported reusable flow exists.
- UI/theme: CSS/theme tokens, light/dark/system preference, and visual QA notes.
- Teams detection: implementation status, structured verification guide, and small detector/logging improvements only.
- Microsoft Graph export: delegated-auth feasibility, OneNote and Planner export docs/design, implementation only if self-contained and mockable.
- Branding/version/license: ClawScribe rebrand, `0.5.0-alpha.1` versioning, About/legal notices, and remaining-string report.
- QA/release: release gate, verification checklists, safe baseline checks, and no-overclaiming final report structure.

## Conflict Rules

- `frontend/src/components/ModelSettingsModal.tsx` is integration-controlled because Auth, Microsoft, and theme may all want settings UI.
- `frontend/src/components/About.tsx` is owned by branding/version unless the coordinator explicitly integrates theme classes later.
- `frontend/src/app/globals.css` and `frontend/tailwind.config.ts` are owned by UI/theme.
- `frontend/src-tauri/src/openclaw.rs` is protected for backward compatibility; changes require coordinator review.
- `docs/productization/phase0-inventory.md` is coordinator-owned.

## Integration Order

1. Merge low-risk docs: auth findings, Microsoft feasibility, Teams verification, QA gate.
2. Merge branding/version/legal after checking remaining `Meetily` strings are classified.
3. Merge theme tokens and theme preference UI.
4. Integrate settings UI changes manually after deciding how OpenAI login/API key and Microsoft login appear together.
5. Run focused checks after each integration wave.
6. Run broader frontend/Rust checks once code changes settle.

## No-Overclaiming Rules

- Windows Teams runtime verification is pending until run on a Windows machine with Teams.
- Windows packaging is pending until an explicit release checkpoint build.
- Microsoft Graph export cannot be called verified without delegated Microsoft login and a test notebook/plan.
- OpenAI OAuth cannot be called implemented unless it uses a supported OpenClaw/Hermes-style flow and passes auth verification without private endpoints or browser scraping.

## Protected Behaviors

- OpenClaw handoff remains optional and backward compatible.
- Existing `openclaw.json` config and `.openclaw-*` recording-folder markers remain supported.
- Existing Meetily-format recording folders and artifact layout names remain readable.
- Secrets must not appear in logs, docs examples, screenshots, commits, or chat.
