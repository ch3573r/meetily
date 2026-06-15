# ClawScribe Teams Detection Verification

Date: 2026-06-15
Branch: `feat/clawscribe-productization-auth-theme-exports`
Status: code-inspected, pending Windows Teams runtime verification

This guide verifies the current ClawScribe Teams detector without enabling any
automatic recording behavior. It is written for Alex's Windows test laptop and
must be run from a logged-in desktop session with Microsoft Teams available.

Do not claim Windows runtime verification until this matrix has been executed
on Windows with real Teams desktop and browser Teams sessions.

## Code Inspection Findings

- Implementation files:
  - `frontend/src-tauri/src/teams_detection.rs`
  - `frontend/src/services/teamsDetectionService.ts`
  - `frontend/src/app/layout.tsx` installs the dev-only debug bridge.
- Platform support is Windows-only for live process and window enumeration.
  Non-Windows returns `supported=false`, `enabled=false`,
  `status="unsupported"`, `detected=false`, and confidence `0.0`.
- Desktop Teams support is heuristic-based. It recognizes `teams.exe`,
  `ms-teams.exe`, `msteams.exe`, process names containing
  `Microsoft Teams`, and legacy `update.exe` only when the executable path or
  command line contains Teams context.
- Browser Teams support covers `msedge.exe`, `chrome.exe`, and
  `msedgewebview2.exe`, with a positive browser meeting title tied to an
  Edge/Chrome/WebView2 process.
- Signal sources are local only: process list, relevant Win32 visible window
  titles, foreground window state, minimized state, and returned diagnostics.
  There are no Graph APIs, tenant permissions, Teams bots, cloud transcript
  APIs, browser cookies, or Teams service calls in this detector.
- Meeting title matching requires both Teams context and meeting context.
  Teams context is `teams`, `microsoft teams`, or `teams.microsoft.com`.
  Meeting context is one of `meeting`, `call`, `joined`, `lobby`,
  `screen sharing`, `presenting`, `participants`, `mute`, `unmute`, or `leave`.
- Default config is:

  ```json
  {
    "enabled": true,
    "confidenceThreshold": 0.65,
    "requireMeetingTitleSignal": true,
    "maxWindowTitleSamples": 100
  }
  ```

- Confidence scoring is additive and capped at `1.0`:
  - Teams desktop process: `0.30`
  - Edge/Chrome/WebView2 process: `0.10`
  - Teams meeting-like visible title: `0.50`
  - Teams meeting-like browser title, only when no Teams desktop process exists:
    `0.35`
  - Foreground Teams meeting-like title: `0.10`
- With `requireMeetingTitleSignal=true`, process-only evidence cannot detect a
  meeting. Confidence is capped below threshold and
  `diagnostics.confidenceCappedByTitleRequirement=true`.
- Positive detection only returns `nextRecommendedAction="promptToRecord"`.
  It does not start, stop, schedule, or silently trigger recording.
- Required safety invariant on every status response:

  ```json
  {
    "recordingSafety": {
      "mode": "prompt-only",
      "automaticRecordingAllowed": false,
      "promptRequired": true
    }
  }
  ```

## Runtime Setup

1. On the Windows test laptop, start ClawScribe from the repo:

   ```powershell
   cd frontend
   pnpm tauri:dev:cpu
   ```

2. Open ClawScribe DevTools and confirm the debug bridge exists:

   ```js
   window.__clawscribeTeamsDetection
   ```

3. Use this helper for every matrix row:

   ```js
   const status = await window.__clawscribeTeamsDetection.printStatus()
   copy(JSON.stringify(status, null, 2))
   status
   ```

4. To verify disabled behavior:

   ```js
   const config = await window.__clawscribeTeamsDetection.getConfig()
   const disabled = await window.__clawscribeTeamsDetection.printStatus({
     ...config,
     enabled: false
   })
   copy(JSON.stringify(disabled, null, 2))
   disabled
   ```

Do not click **Start Recording** during this detector smoke unless a separate
manual recording test is being performed. If ClawScribe starts or stops
recording as a side effect of any command above, stop testing and treat it as a
blocker.

## Alex's Verification Matrix

| ID | Scenario | Setup | Expected status | Required diagnostics and safety |
| --- | --- | --- | --- | --- |
| T0 | Non-Windows sanity, if run from Linux/macOS | Call `printStatus()` outside Windows | `supported=false`, `enabled=false`, `status="unsupported"`, `detected=false`, confidence `0.0`, `nextRecommendedAction="unsupported"` | `recordingSafety.automaticRecordingAllowed=false`; no recording side effects |
| T1 | Windows baseline, Teams fully closed | Close Teams desktop, Edge/Chrome Teams tabs, and Teams WebView windows; call `printStatus()` | `supported=true`, `enabled=true`, `detected=false`, usually `status="notDetected"`, `nextRecommendedAction="idle"` | `teamsProcessCount=0`, `meetingTitleCount=0`, `titleSignalSatisfied=false`; safety false |
| T2 | Teams desktop idle, no meeting | Open Teams desktop to chat/calendar, do not join a meeting; call `printStatus()` | `detected=false`, usually `status="possible"` if a Teams process is found, `nextRecommendedAction="idle"` | `teamsProcessCount>0`, `meetingTitleCount=0`, `titleSignalSatisfied=false`, `confidenceCappedByTitleRequirement=true`; safety false |
| T3 | Teams desktop active meeting | Join a test Teams meeting in desktop Teams; keep the meeting window visible; call `printStatus()` | `detected=true`, `status="detected"`, confidence at least `0.65`, `nextRecommendedAction="promptToRecord"` | `teamsProcessCount>0`, `meetingTitleCount>0`, `titleSignalSatisfied=true`; safety false |
| T4 | Teams desktop foreground boost | While still in the desktop meeting, make the meeting window active and call `printStatus()` | `detected=true`; expected confidence includes the foreground boost, normally around `0.90` for desktop process + title + foreground | `foregroundMeetingTitleCount>0`, at least one signal named `foreground-meeting-window` is matched; safety false |
| T5 | Teams desktop minimized/background | Minimize the meeting window or put another app in front; call `printStatus()` | Acceptable result is `detected=true` if a visible meeting-like title is still enumerated, or `possible/notDetected` if Windows no longer exposes a visible meeting window | `isMinimized` and `isForeground` candidate fields explain the result; no automatic start/stop; safety false |
| T6 | Leave meeting, keep Teams open | Leave the desktop meeting but keep Teams running; call `printStatus()` after the meeting window closes | `detected=false`, `nextRecommendedAction="idle"` | `meetingTitleCount` should return to `0` unless Teams leaves a meeting-like window title visible; no recording side effects |
| T7 | Browser Teams in Edge | Join a Teams meeting from `teams.microsoft.com` in Edge; call `printStatus()` | `detected=true`, `status="detected"`, confidence at least `0.65`, `nextRecommendedAction="promptToRecord"` | `browserProcessCount>0`, `browserMeetingTitleCount>0`, `meetingTitleCount>0`; safety false |
| T8 | Browser Teams in Chrome | Join a Teams meeting from `teams.microsoft.com` in Chrome; call `printStatus()` | `detected=true`, `status="detected"`, confidence at least `0.65`, `nextRecommendedAction="promptToRecord"` | `browserProcessCount>0`, `browserMeetingTitleCount>0`, `meetingTitleCount>0`; safety false |
| T9 | Browser idle/non-Teams | Open Edge/Chrome with ordinary non-Teams tabs; call `printStatus()` | `detected=false`, usually `status="possible"` if only browser process evidence is present, `nextRecommendedAction="idle"` | `browserProcessCount>0`, `meetingTitleCount=0`, `titleSignalSatisfied=false`, safety false |
| T10 | Disabled config | Run the disabled config snippet above | `status="disabled"`, `enabled=false`, `detected=false`, confidence `0.0`, `nextRecommendedAction="disabled"` | Safety false; no process/window side effects |
| T11 | Recording safety invariant | Repeat during any positive detection case | No recording starts unless the user explicitly clicks the normal recording control | `recordingSafety.mode="prompt-only"`, `automaticRecordingAllowed=false`, `promptRequired=true`, `nextRecommendedAction="promptToRecord"` only |

Pass criteria:

- All expected status and diagnostics fields match for the scenario.
- Every row preserves `recordingSafety.automaticRecordingAllowed=false`.
- Positive detection rows recommend only `promptToRecord`.
- No row starts, stops, pauses, resumes, schedules, or changes recording state.
- Candidate window titles do not expose sensitive meeting content in committed
  artifacts. Keep raw JSON evidence local or redact meeting names before
  sharing.

## Result Logging Template

Use this template for Alex's run notes:

```markdown
## Teams Detection Runtime Results

Date:
Machine:
Windows version:
ClawScribe commit:
Teams desktop version:
Edge version:
Chrome version:

| ID | Pass/Fail | Status | Confidence | Key diagnostics | Notes |
| --- | --- | --- | --- | --- | --- |
| T1 | | | | | |
| T2 | | | | | |
| T3 | | | | | |
| T4 | | | | | |
| T5 | | | | | |
| T6 | | | | | |
| T7 | | | | | |
| T8 | | | | | |
| T9 | | | | | |
| T10 | | | | | |
| T11 | | | | | |

Recording safety observed:
Blockers:
Follow-up tuning needed:
```

For each row, preserve at minimum:

- `status`, `detected`, `confidence`, `threshold`, and `reason`
- `nextRecommendedAction`
- `recordingSafety`
- `diagnostics`
- candidate count and candidate `source`/`processName` values

Redact or omit full `windowTitle` values if they contain real meeting names,
customer names, or participant details.

## Known Gaps

- No Windows runtime Teams verification has been performed from this Linux host.
- No WASAPI/audio-session correlation is implemented yet.
- No debounced polling loop or permanent UI status panel is implemented; the
  current bridge is dev-only and pull-based.
- Detection relies on English Teams/browser window title text. Localized Teams
  builds or different title formats may require keyword tuning.
- Browser support is limited to Edge, Chrome, and WebView2 process names.
- The detector does not inspect browser tabs directly; it relies on visible
  window titles and process ownership.
- Minimized, hidden, or multi-window Teams behavior depends on what Win32
  exposes through visible top-level windows.
- Confidence weights are static and have not yet been calibrated against a live
  Windows corpus.
