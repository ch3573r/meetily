# ClawScribe Theme Tokens

Date: 2026-06-15

## Source Palette

| Name | Hex | Usage |
| --- | --- | --- |
| Kontron Blue | `#005083` | Light primary actions, active tabs, focus identity |
| Kontron Green | `#3fb498` | Dark primary accent, active or positive states |
| Black Blue | `#113350` | Strong light-mode text |
| Blue | `#006bac` | Secondary light blue accent |
| Mid Blue | `#4a86b5` | Secondary dark blue accent and charts |
| Light Blue | `#a1bbd0` | Quiet light blue surfaces and dividers |
| Black | `#000000` | Base brand black, shadow source only |
| Dark Grey | `#58585a` | Light-mode muted text |
| Grey | `#808080` | Disabled or nonessential UI |
| Light Grey | `#f2f2f2` | Light app background |
| Magenta | `#e50076` | Tiny highlights only |
| Cyan | `#00ffff` | Tiny highlights only |

## Semantic Tokens

| Token | Light | Dark | Tailwind utility |
| --- | --- | --- | --- |
| App background | `#f2f2f2` | `#0b1117` | `bg-background` |
| Surface/card | `#ffffff` | `#111a22` | `bg-card` |
| Elevated/popover | `#ffffff` | `#172330` | `bg-popover` |
| Strong text | `#113350` | `#f4f7f9` | `text-foreground` |
| Secondary text | `#58585a` | `#b8c4cc` | `text-secondary-foreground` |
| Muted text | `#58585a` | `#82909a` | `text-muted-foreground` |
| Border/input | `#a1bbd0` softened | `#2a3948` | `border-border`, `border-input` |
| Primary action | `#005083` | `#3fb498` | `bg-primary`, `text-primary` |
| Primary action text | `#ffffff` | `#0b1117` | `text-primary-foreground` |
| Secondary surface | light blue tint | `#172330` | `bg-secondary` |
| Accent/positive | `#3fb498` | `#3fb498` | `bg-accent`, green utilities |
| Focus ring | `#005083` | `#3fb498` | `ring-ring`, `focus:ring-ring` |
| Sidebar surface | `#ffffff` | near `#0d1821` | `--sidebar` |
| Sidebar active item | light blue tint | deep blue tint | `--sidebar-active` |
| Sidebar border | softened light blue | stronger blue-grey | `--sidebar-border` |

The native Tauri window theme must follow the user-selected app theme. The
window config should not force `Light`; the app syncs the native title bar via
the `set_native_theme` command when Light, Dark, or System is selected.

## Legacy Utility Mapping

Older screens still contain Tailwind utility classes like `bg-white`, `bg-gray-50`, `text-gray-600`, and `bg-blue-100`. The theme layer keeps these readable by remapping the generated Tailwind palette to Kontron colors and adding dark-mode utility overrides for common hardcoded surfaces, text, borders, blue highlights, and focus rings.

New code should prefer semantic utilities:

- `bg-background` for app canvases.
- `bg-card` for panels and transcript/summary surfaces.
- `bg-secondary` for selected rows, subtle callouts, and active quiet controls.
- `text-foreground`, `text-muted-foreground`, and `text-primary`.
- `border-border`, `border-input`, and `ring-ring`.

## Contrast Notes

Measured contrast ratios for major text/control pairs:

| Pair | Ratio | AA status |
| --- | ---: | --- |
| `#113350` on `#f2f2f2` | 11.63:1 | Pass |
| `#113350` on `#ffffff` | 13.02:1 | Pass |
| `#58585a` on `#f2f2f2` | 6.34:1 | Pass |
| `#ffffff` on `#005083` | 8.48:1 | Pass |
| `#113350` on light secondary blue | 10.45:1 | Pass |
| `#f4f7f9` on `#0b1117` | 17.64:1 | Pass |
| `#f4f7f9` on `#111a22` | 16.34:1 | Pass |
| `#b8c4cc` on `#111a22` | 9.88:1 | Pass |
| `#82909a` on `#111a22` | 5.36:1 | Pass |
| `#0b1117` on `#3fb498` | 7.40:1 | Pass |

The dark border pair `#2a3948` on `#111a22` is about 1.49:1. That is intentional for low-emphasis separators; do not use border color as text.

## Visual QA Notes

Local Linux browser screenshots were captured under `docs/brand/screenshots/`.
The non-Tauri browser path can verify the light app canvas, sidebar, settings
shell, and icon swap, but it cannot complete the full desktop runtime flows
that depend on Tauri commands. Headless browser attempts to force and capture
dark mode through CDP timed out in this environment, so the dark-mode runtime
screenshot pass remains a Windows retest item.

The code-level theme checks are still valid: dark tokens are neutral surfaces
with green and blue accents, light tokens are Kontron-branded, and the major
token contrast pairs above pass WCAG AA for normal text.

## Remaining Hardcoded Color Classification

- Intentional status colors: recording, warning, error, and success states use
  red, amber, yellow, and green semantic feedback colors.
- Intentional overlays: modal overlays such as `bg-black/80` stay literal
  because they express transparency rather than brand color.
- Brand constants: Kontron palette hex values are documented above and should
  remain centralized in the theme layer.
- Deferred legacy surfaces: older model-management, onboarding, progress, and
  AI summary components still contain some literal Tailwind color utilities and
  SVG stroke values. The global palette mapping and dark overrides keep common
  cases readable, but these components should be directly tokenized when they
  are next touched.
