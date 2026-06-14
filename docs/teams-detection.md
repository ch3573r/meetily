# Teams Meeting Detection

This fork includes a read-only Teams meeting detection scaffold for Windows.
It is intended to make meeting state observable first, then feed a later
auto-recording workflow once start/stop rules have been proven.

## Tauri Commands

- `get_teams_detection_config` returns the default detector configuration.
- `get_teams_detection_status` returns a single detection snapshot. It accepts
  an optional config override and does not start or stop recording.

On non-Windows platforms the status response is cleanly unsupported:
`supported=false`, `enabled=false`, `detected=false`, and confidence `0.0`.

## Config

Default config:

```json
{
  "enabled": true,
  "confidenceThreshold": 0.65,
  "requireMeetingTitleSignal": true,
  "maxWindowTitleSamples": 25
}
```

`enabled` defaults to `true` only on Windows. `confidenceThreshold` is clamped
to `0.0..1.0`. `requireMeetingTitleSignal` prevents a background Teams process
from being enough by itself. `maxWindowTitleSamples` limits how many visible
window titles the detector samples per status call.

## Detector Signals

The current Windows detector uses lightweight local heuristics:

- Teams desktop process: `teams.exe`, `ms-teams.exe`, `msteams.exe`, legacy
  Teams updater names, or process names containing `Microsoft Teams`.
- Browser process: `msedge.exe`, `chrome.exe`, or `msedgewebview2.exe`.
- Visible window titles: titles containing Teams context plus meeting context
  such as meeting, call, joined, lobby, presenting, participants, mute/unmute,
  or leave.
- Browser meeting title: a Teams-like meeting title tied to an Edge/Chrome
  process.

The detector intentionally avoids Graph APIs, tenant permissions, Teams bots,
or cloud transcript APIs.

## Confidence Thresholds

Current weights:

- Teams desktop process: `0.35`
- Edge/Chrome browser process: `0.15`
- Teams meeting-like visible title: `0.45`
- Teams meeting-like browser title, when no desktop Teams process exists:
  `0.35`

The default threshold is `0.65`. With `requireMeetingTitleSignal=true`, process
signals alone cannot cross the threshold. Practical examples:

- Teams desktop process plus meeting title: `0.80`, detected.
- Edge/Chrome plus Teams meeting title: `0.95`, detected.
- Teams desktop process only: capped below threshold, not detected.
- Browser process only: `0.15`, not detected.

The status response includes the final `confidence`, `threshold`, each signal,
candidate processes/window titles, a human-readable reason, and
`nextRecommendedAction`. The current positive action is `PromptToRecord`, not
automatic recording.

## False-Positive Controls

The detector defaults are conservative:

- A visible meeting-like title is required.
- Background Teams, Edge, and Chrome processes do not trigger detection alone.
- Window title sampling is bounded to avoid expensive polling.
- The command is pull-based; callers choose polling cadence and can debounce in
  the UI or a future background worker.

Recommended UI behavior is to show an observable "possible Teams meeting"
state or prompt rather than starting capture immediately.

## WASAPI Future Detector

WASAPI/system-audio correlation is a good next detector, but it is intentionally
not part of this scaffold. A future implementation can raise confidence when:

- The default communications device or loopback stream is active.
- Teams/Edge/Chrome owns an active audio session.
- Microphone activity and system audio overlap for a sustained period.

This should be added through a dedicated audio-session detector so it can be
debounced and tested independently from process/window heuristics.

## Next Steps for Auto-Recording

1. Add a frontend or background polling loop for `get_teams_detection_status`.
2. Debounce positive detection for a sustained interval, for example 10-20
   seconds above threshold.
3. Debounce stop detection separately, for example 60-120 seconds below
   threshold, to avoid stopping during reconnects or Teams window changes.
4. Require user opt-in and expose the threshold in settings.
5. Wire the positive state to the existing recording commands only after the
   recorder can safely reject duplicate starts and preserve manual override.
6. Add WASAPI audio-session evidence before enabling unattended start/stop by
   default.
