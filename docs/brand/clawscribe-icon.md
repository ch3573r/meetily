# ClawScribe Icon

The ClawScribe icon is generated from `frontend/src-tauri/icons/clawscribe-icon.svg`.

## Direction

- Product meaning: meeting capture, transcripts/notes, and the OpenClaw claw identity.
- Shape: Windows-friendly rounded square with a simple central mark that remains legible at 16x16 and 32x32.
- Mark: three claw strokes arranged as a pen nib, with transcript lines below.
- Colors:
  - Primary blue: `#005083`
  - Green accent: `#3fb498`
  - Dark navy: `#113350`

## Generated Assets

Generated with local tooling:

```bash
cd /path/to/clawscribe

for size in 16 24 32 48 64 128 256 512 1024; do
  rsvg-convert -w "$size" -h "$size" \
    frontend/src-tauri/icons/clawscribe-icon.svg \
    -o "/tmp/clawscribe-icons/icon_${size}.png"
done

# PNG outputs are copied into frontend/src-tauri/icons and frontend/public.
# Python 3 stdlib packaging scripts write icon.ico, app_icon.ico,
# icon.icns, app_icon.icns, and frontend/src/app/favicon.ico from those PNGs.
```

Required app, installer, tray, taskbar, Tauri, About, and public logo assets are regenerated from the same source mark. The tray icon uses Tauri's default window icon, so it inherits the bundled icon configured in `frontend/src-tauri/tauri.conf.json`.

Dependencies used: `rsvg-convert` from librsvg and Python 3 standard library. ImageMagick and Pillow were not installed or required.
