# ClawScribe Icon

The ClawScribe icon is generated from the current product mark. The source mark
is `docs/brand/clawscribe-logo.png`: an abstract claw, waveform, and scribe
mark. App-icon exports fit that mark into a rounded-square mask and keep
transparent corners where the target format supports alpha.

## Generated Assets

Regenerate from the source mark with Python 3 and Pillow, producing the assets
below.

Covered surfaces:

- Tauri app/window/default/tray icon: `frontend/src-tauri/icons/icon.png`
- Windows installer/uninstaller icon: `frontend/src-tauri/icons/app_icon.ico`
- Windows `.ico`: `frontend/src-tauri/icons/icon.ico`, `app_icon.ico`
- macOS `.icns`: `frontend/src-tauri/icons/icon.icns`, `app_icon.icns`
- Next/web favicon: `frontend/src/app/favicon.ico`
- About/public icon: `frontend/public/icon_128x128.png`, `icon_32x32@2x.png`
- Sidebar collapsed logo: `frontend/public/logo-collapsed.png`
- Public horizontal logo: `frontend/public/logo.png`
- Public brand sizes: `frontend/public/brand/clawscribe-icon-*`
- Microsoft Store/Windows logo aliases under `frontend/src-tauri/icons/`

The expanded sidebar currently renders the text label `ClawScribe`; the
collapsed sidebar and About dialog use the regenerated product icon.

Dependencies used: Python 3 and Pillow.

Run `pnpm verify:icons` from `frontend/` before cutting Windows installers. It
checks the Tauri bundle wiring, NSIS installer/uninstaller icons, app/exe ICO
sizes, macOS ICNS files, favicon, Windows logo aliases, and public brand assets.
