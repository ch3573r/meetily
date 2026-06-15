# ClawScribe Theme QA

Date: 2026-06-15
Branch: `feat/clawscribe-productization-auth-theme-exports`

## Scope

- Added Kontron-compliant light theme tokens for ClawScribe.
- Added dark mode tokens and a persisted Light / Dark / System preference.
- Kept auth, export, About, and Model Settings copy out of scope.
- Preserved OpenClaw handoff UI behavior; only presentation classes around the status panel were tokenized.

## Token Mapping

Kontron source colors are centralized in `frontend/src/app/globals.css` as CSS variables:

- Primary `#005083`
- Accent/signal `#3fb498`
- Black Blue `#113350`
- Blue `#006bac`
- Mid Blue `#4a86b5`
- Light Blue `#a1bbd0`
- Black `#000000`
- Dark Grey `#58585a`
- Grey `#808080`
- Light Grey `#f2f2f2`
- Magenta `#e50076` and Cyan `#00ffff` are retained as highlight-only tokens.

Tailwind semantic colors now resolve through CSS variables in `frontend/tailwind.config.ts`, including `background`, `foreground`, `card`, `primary`, `secondary`, `muted`, `accent`, `destructive`, `border`, `input`, and `ring`.

## Manual Visual QA Steps

1. Start the frontend:

   ```bash
   cd frontend
   pnpm dev
   ```

2. Open ClawScribe and navigate to Settings > General.
3. Verify the Appearance section shows Light, Dark, and System options.
4. Select Light.
   - The app applies the Kontron light palette immediately.
   - The primary blue should align with `#005083`.
   - Cards should remain white on the light grey `#f2f2f2` app background.
5. Select Dark.
   - The root `html` element should receive the `dark` class.
   - Settings cards, tabs, OpenClaw handoff status, and storage controls should change without a page reload.
6. Select System.
   - Change the OS light/dark preference.
   - ClawScribe should follow the OS preference while keeping `localStorage["clawscribe.theme"]` set to `system`.
7. Reload the app.
   - The last selected preference should persist.
8. Confirm OpenClaw handoff status still loads and the refresh button still invokes the status check.

## Screenshot Notes

No screenshots are attached from this Linux-side pass. Capture screenshots during Windows/Tauri visual QA after launching the packaged app or `pnpm tauri:dev` on a machine with the full runtime.

Recommended screenshot set:

- Settings > General, Light
- Settings > General, Dark
- Main recording view, Light
- Main recording view, Dark
- OpenClaw handoff panel with configured and unconfigured states, if available

## Remaining Integration Notes

- The settings control is mounted through `PreferenceSettings`, which appears in both the settings page and the legacy preferences modal.
- Full app-wide dark mode quality still depends on progressively replacing older hard-coded one-off classes in unrelated screens with semantic tokens.
