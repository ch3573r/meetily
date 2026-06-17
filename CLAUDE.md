# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Current Handoff - ClawScribe Productization

Updated: 2026-06-17 Europe/Vienna.

### Start Here

Launch Claude CLI from this folder:

```bash
cd /path/to/clawscribe
claude
```

This is the active ClawScribe/Meetily fork. Work is on branch
`feat/clawscribe-productization-auth-theme-exports`, tracking
`fork/feat/clawscribe-productization-auth-theme-exports`.

Recent commits, newest first (2026-06-17 session):

```text
3a6c772 CI: opt-in 'directml' input to build Parakeet DirectML (experimental probe)
5a97080 Planner AI-polish: skip Codex cleanly (titles already model-authored)
f649fc8 DirectML Parakeet (Phase 1, dormant): cfg-gated EP + Beta toggle
425955d Persist Me/Participants speaker label for saved meetings
416d341 Settings: split Add-ons into Add-ons + Diagnostics tabs
b2951f4 Settings IA cleanup: tab scroll, naming, diagnostics, General order
1cdb2d3 Settings IA: single-column General, consolidate OpenClaw handoff
e207042 Route brand-energy UI through the accent token (no hardcoded cyan)
a6eb9fb Detect new-Teams in-call window via companion-window signal
```

### Product Context

This fork is being productized as **ClawScribe**, a Windows-oriented, bot-free
meeting recorder/summarizer based on Meetily Community Edition `0.4.0`.

Primary goal: a Windows app that records local mic + system audio, transcribes
locally, and can summarize through:

- Built-in local AI
- OpenAI API key
- OpenAI-compatible endpoint
- OpenClaw endpoint
- Bundled Codex app-server / ChatGPT sign-in path

Do not reintroduce the archived Python/FastAPI backend as a supported runtime.
Current app behavior belongs in `frontend/src-tauri/src` and the Tauri/Next UI.

### What's New Since Last Handoff

1. **`parse_meeting_output` trailing-character fix** (`533beaf`)
   - Codex/LLM providers sometimes return JSON with trailing characters.
   - Fixed with serde_json streaming deserializer.
   - File: `frontend/src-tauri/src/summary/codex_provider.rs`

2. **Live Microsoft Graph exports** (`7f1f18e`, `f36727a`)
   - Full Entra ID device-code OAuth 2.0 flow with OS keychain token storage.
   - OneNote page export and Planner task export via Microsoft Graph API.
   - Discovery commands for notebooks, sections, plans, and buckets.
   - IntegrationsSettings UI rewritten with sign-in panel and destination pickers.
   - App registration: client ID `4ab2ca8f-c2f1-45f3-b4ee-8bc9a511bcc8`,
     tenant `d0627577-cabb-4909-8ea1-c5d86abfd204`.
   - Key files:
     - `frontend/src-tauri/src/exports/` (auth, token_store, ms_auth_state,
       reqwest_transport, discovery, commands)
     - `frontend/src/services/microsoftExportService.ts`
     - `frontend/src/hooks/useMicrosoftExport.ts`
     - `frontend/src/components/IntegrationsSettings.tsx`

3. **Self-hosted GitHub Actions runner** (this session)
   - Proxmox VM `build-runner` (VMID 103) at `build-host.local` (node `pve`,
     `infra-host.local:8006`), cloned from the `WinSrv2025` template (VMID 10000).
   - Windows Server 2025, **6 cores**, 6GB RAM, 128GB disk.
   - Runner labels: `self-hosted, windows, x64, clawscribe`.
   - Workflow `runs-on: [self-hosted, windows, clawscribe]`.
   - Service `actions.runner.runner-service.build-runner` runs as a **local admin
     user `.\builder`** (NOT a service account — see gotchas below).
   - Toolchain installed via choco/rustup: VS Build Tools 2022 (C++ workload),
     **LLVM 20.1.8** (pinned), Rust 1.96.0, Node 24.16.0, pnpm 10.34.3,
     Git 2.54.0, CMake 4.3.3, PowerShell 7, NSIS 3.12, Vulkan SDK 1.4.350.

   **Non-obvious runner setup that MUST be preserved (cost real debugging):**
   - **LLVM must be 20.x, not 22+.** bindgen 0.69 (via whisper-rs-sys) mis-parses
     `whisper_full_params` with clang 21+, dropping 71 struct fields. The
     workflow pins `choco install llvm --version=20.1.8 --allow-downgrade`.
   - **Runner runs as a normal local user, not LocalSystem/NETWORK SERVICE.**
     WiX `light.exe` ICE validation needs admin privileges (NETWORK SERVICE
     fails). LocalSystem has privileges but its profile is under
     `C:\WINDOWS\system32\...`, and 32-bit `makensis.exe` then hits WoW64
     `system32`→`SysWOW64` redirection and can't load `zlib1.dll`
     (STATUS_DLL_NOT_FOUND / "os error 2"). A local admin user
     (`C:\Users\builder`) satisfies both.
   - **Windows Defender was uninstalled** (`Uninstall-WindowsFeature
     Windows-Defender`). It was content-flagging NSIS `makensis.exe` and
     silently truncating it to a 2560-byte stub. Do not reinstall.
   - **`git config --system --add safe.directory "*"`** is set so the build
     can run git in the cached workspace regardless of which account created it.
   - **Build cache is disk-local**: workflow uses `checkout clean:false` +
     machine-level `CARGO_HOME=C:\cargo`. No GitHub Actions cache is used
     (the `swatinem/rust-cache` step was removed), so nothing is written to the
     repo's Actions cache storage.

### Current Release/Test Status

Both CPU and Vulkan installers build green on the self-hosted runner:

- CPU run `27624474317` — success, artifact
  `clawscribe-windows-cpu-f6265aff...` (MSI + NSIS).
- Vulkan run `27625222659` — success, artifact
  `clawscribe-windows-vulkan-f6265aff...` (MSI + NSIS).
- GitHub Actions storage cleaned: old artifacts + stale `windows-latest`
  caches deleted; only the latest CPU/Vulkan artifacts retained.

Runtime Windows checks still pending (need actual install/run):

- Start Menu/taskbar/window/About icon visually updated and transparent
- No blank command windows during installed-app use
- Browser login opens correctly
- Device-code login opens verification page correctly
- Microsoft Graph sign-in and export to OneNote/Planner
- OpenAI-compatible/OpenClaw summary regeneration uses "Add context"

### Add-ons State

Settings -> Add-ons now exposes:

- Teams detection status
- OpenClaw handoff
- Microsoft sign-in (Entra ID device-code flow)
- OneNote export: live with notebook/section picker
- Planner task export: live with plan/bucket picker
- Advanced Codex app-server status

### Shipped this session (2026-06-17)

All committed/pushed on the feature branch; each built green on the self-hosted
runner (latest plain build artifact `clawscribe-windows-vulkan-<sha>`):

- Recording UX: restored floating pause/stop on the recording screen; sidebar
  "Recording" indicator stops; "Paused" labels; shortcuts "Reset" button.
- Accent (Direction B): all brand-energy UI driven by the `--primary` token
  (no hardcoded cyan); "Kontron" dropped from the light-theme label.
- Teams detection: fixed false positives (anchor on "Microsoft Teams" + section
  denylist) AND the missed real meeting (companion-window signal, not foreground).
  Add-ons detection panel auto-refreshes.
- Planner export: review/preview dialog (check/uncheck, edit title, per-task
  bucket defaulting to settings "Default Bucket") + optional AI title/notes
  polish (Settings → Add-ons toggle, default off; Codex skipped intentionally).
  OneNote "New notebook" + Planner "New bucket" creation (delegated only).
- Settings IA: split Add-ons (configure) vs new Diagnostics tab (status: Teams
  signals, OpenClaw handoff, Codex app-server status); single-column General;
  tab strip scrolls; widened to 2400px like Home.
- Transcription: Me/Participants speaker labels render in the transcript AND
  persist to the DB (the `speaker` column was unused). whisper `set_n_threads`
  applied; temperature fallback restored; VAD resampler swapped to rubato sinc.

### Open bug backlog (reported 2026-06-17, NOT yet fixed)

1. First-run model-download/setup dialog has dark-mode contrast issues (light
   mode untested).
2. **Source attribution broken**: everything labels as "Me" (a YouTube video via
   system audio should be "Participants"). And the speaker label is NOT included
   in the transcript text sent to the AI summary — so who-said-what never reaches
   the LLM. (See `audio/pipeline.rs` energy attribution + summary transcript
   assembly in `useSummaryGeneration.ts`.)
3. OpenClaw provider should NOT prefill a default IP/endpoint.
4. OpenAI-compatible Connection Test fails with LiteLLM `content_safety_violation`
   while the transcription test succeeds — the connection-test prompt trips a
   guardrail; use a benign test payload.
5. Teams detection shows "Prompt only" but no record prompt appears on detection
   — the prompt-to-record path isn't surfaced.
6. Import Audio modal: no drag-and-drop; long filenames break layout bounds.
   Wanted: post-import stats popup (elapsed time + RTF/segments) to use Import as
   the benchmark harness.

### Parked (see memory + roadmap notes)

- **DirectML Parakeet** — Phase 1 done & toolchain-validated (probe build green,
  `directml` cargo feature links on the runner); opt-in CI only, not in default
  build. Pending: benchmark int8-on-DirectML on the i5-1235u, then make default
  or do Phase 2 (fp32 model). Import on the 1235u "didn't use much GPU with the
  DirectML switch on" — expected for int8 (ops fall back to CPU); that's the
  Phase-2/fp32 signal.
- **ASR alternatives roadmap** — Nemotron 3.5 ASR Streaming 0.6B ONNX INT4 is the
  standout Parakeet alternative (ONNX/INT4/streaming/multilingual incl. German,
  DirectML-capable via the onnx-asr family); Moonshine v2 (English-only, ultra-low
  latency) secondary. See the roadmap memory.

For local Linux validation after changes, prefer focused tests first, then
`pnpm build`, then broader `cargo test` only when the change justifies the time
and disk cost.

## Project Overview

**Meetily** is a privacy-first AI meeting assistant that captures, transcribes, and summarizes meetings entirely on local infrastructure. The supported application is the Tauri desktop app with a Rust core.

1. **Frontend**: Tauri-based desktop application (Rust + Next.js + TypeScript)
2. **Rust Backend**: Tauri commands, audio capture, transcription, storage, and summarization orchestration
3. **Legacy Backend Archive**: the old Python/FastAPI, Docker, and standalone whisper-server backend under `backend/` is archived and unsupported

### Key Technology Stack
- **Desktop App**: Tauri 2.x (Rust) + Next.js 14 + React 18
- **Audio Processing**: Rust (cpal, whisper-rs, professional audio mixing)
- **Transcription**: Whisper.cpp / whisper-rs and Parakeet paths in the Tauri app
- **App API Surface**: Tauri commands and events, not a separate FastAPI service
- **LLM Integration**: Built-in/local AI, OpenAI API key, OpenAI-compatible endpoints, OpenClaw, and bundled Codex app-server

## Essential Development Commands

### Frontend Development (Tauri Desktop App)

**Location**: `/frontend`

```bash
# macOS Development
./clean_run.sh              # Clean build and run with info logging
./clean_run.sh debug        # Run with debug logging
./clean_build.sh            # Production build

# Windows Development
clean_run_windows.bat       # Clean build and run
clean_build_windows.bat     # Production build

# Manual Commands
pnpm install                # Install dependencies
pnpm run dev                # Next.js dev server (port 3118)
pnpm run tauri:dev          # Full Tauri development mode
pnpm run tauri:build        # Production build

# GPU-Specific Builds (for testing acceleration)
pnpm run tauri:dev:metal    # macOS Metal GPU
pnpm run tauri:dev:cuda     # NVIDIA CUDA
pnpm run tauri:dev:vulkan   # AMD/Intel Vulkan
pnpm run tauri:dev:cpu      # CPU-only (no GPU)
```

### Legacy Backend Archive

**Location**: `/backend`

The Python/FastAPI backend, Docker setup, and standalone whisper-server scripts are archived for historical reference and migration context only. Do not use them for current development, new installs, production deployments, or issue triage for the supported app.

The archived FastAPI service had unauthenticated, development-oriented CORS behavior. Treat that behavior as obsolete legacy context, not as a supported production API.

### Service Endpoints
- **Frontend Dev**: http://localhost:3118

## High-Level Architecture

### Tauri Desktop Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Frontend (Tauri Desktop App)                  │
│  ┌──────────────────┐  ┌─────────────────┐  ┌────────────────┐ │
│  │   Next.js UI     │  │  Rust Backend   │  │ Whisper Engine │ │
│  │  (React/TS)      │←→│  (Audio + IPC)  │←→│  (Local STT)   │ │
│  └──────────────────┘  └─────────────────┘  └────────────────┘ │
│         ↑ Tauri Events           ↑ Audio Pipeline               │
└─────────────────────────────────────────────────────────────────┘
```

The current app does not require a separate FastAPI tier. Meeting persistence, local transcription, and summary orchestration are handled through the Rust/Tauri core.

### Audio Processing Pipeline (Critical Understanding)

The audio system has **two parallel paths** with different purposes:

```
Raw Audio (Mic + System)
         ↓
┌────────────────────────────────────────────────────────────┐
│              Audio Pipeline Manager                         │
│  (frontend/src-tauri/src/audio/pipeline.rs)                │
└─────────────┬──────────────────────────┬───────────────────┘
              ↓                          ↓
    ┌─────────────────┐        ┌─────────────────────┐
    │ Recording Path  │        │ Transcription Path  │
    │ (Pre-mixed)     │        │ (VAD-filtered)      │
    └─────────────────┘        └─────────────────────┘
              ↓                          ↓
    RecordingSaver.save()      WhisperEngine.transcribe()
```

**Key Insight**: The pipeline performs **professional audio mixing** (RMS-based ducking, clipping prevention) for recording, while simultaneously applying **Voice Activity Detection (VAD)** to send only speech segments to Whisper for transcription.

### Audio Device Modularization (Recently Completed)

**Context**: The audio system was refactored from a monolithic 1028-line `core.rs` file into focused modules. See [AUDIO_MODULARIZATION_PLAN.md](AUDIO_MODULARIZATION_PLAN.md) for details.

```
audio/
├── devices/                    # Device discovery and configuration
│   ├── discovery.rs           # list_audio_devices, trigger_audio_permission
│   ├── microphone.rs          # default_input_device
│   ├── speakers.rs            # default_output_device
│   ├── configuration.rs       # AudioDevice types, parsing
│   └── platform/              # Platform-specific implementations
│       ├── windows.rs         # WASAPI logic (~200 lines)
│       ├── macos.rs           # ScreenCaptureKit logic
│       └── linux.rs           # ALSA/PulseAudio logic
├── capture/                   # Audio stream capture
│   ├── microphone.rs          # Microphone capture stream
│   ├── system.rs              # System audio capture stream
│   └── core_audio.rs          # macOS ScreenCaptureKit integration
├── pipeline.rs                # Audio mixing and VAD processing
├── recording_manager.rs       # High-level recording coordination
├── recording_commands.rs      # Tauri command interface
└── recording_saver.rs         # Audio file writing
```

**When working on audio features**:
- Device detection issues → `devices/discovery.rs` or `devices/platform/{windows,macos,linux}.rs`
- Microphone/speaker problems → `devices/microphone.rs` or `devices/speakers.rs`
- Audio capture issues → `capture/microphone.rs` or `capture/system.rs`
- Mixing/processing problems → `pipeline.rs`
- Recording workflow → `recording_manager.rs`

### Rust ↔ Frontend Communication (Tauri Architecture)

**Command Pattern** (Frontend → Rust):
```typescript
// Frontend: src/app/page.tsx
await invoke('start_recording', {
  mic_device_name: "Built-in Microphone",
  system_device_name: "BlackHole 2ch",
  meeting_name: "Team Standup"
});
```

```rust
// Rust: src/lib.rs
#[tauri::command]
async fn start_recording<R: Runtime>(
    app: AppHandle<R>,
    mic_device_name: Option<String>,
    system_device_name: Option<String>,
    meeting_name: Option<String>
) -> Result<(), String> {
    // Implementation delegates to audio::recording_commands
}
```

**Event Pattern** (Rust → Frontend):
```rust
// Rust: Emit transcript updates
app.emit("transcript-update", TranscriptUpdate {
    text: "Hello world".to_string(),
    timestamp: chrono::Utc::now(),
    // ...
})?;
```

```typescript
// Frontend: Listen for events
await listen<TranscriptUpdate>('transcript-update', (event) => {
  setTranscripts(prev => [...prev, event.payload]);
});
```

### Whisper Model Management

**Model Storage Locations**:
- **Development**: `frontend/models/`
- **Production (macOS)**: `~/Library/Application Support/Meetily/models/`
- **Production (Windows)**: `%APPDATA%\Meetily\models\`

**Model Loading** (frontend/src-tauri/src/whisper_engine/whisper_engine.rs):
```rust
pub async fn load_model(&self, model_name: &str) -> Result<()> {
    // Automatically detects GPU capabilities (Metal/CUDA/Vulkan)
    // Falls back to CPU if GPU unavailable
}
```

**GPU Acceleration**:
- **macOS**: Metal + CoreML (automatically enabled)
- **Windows/Linux**: CUDA (NVIDIA), Vulkan (AMD/Intel), or CPU
- Configure via Cargo features: `--features cuda`, `--features vulkan`

## Critical Development Patterns

### 1. Audio Buffer Management

**Ring Buffer Mixing** (pipeline.rs):
- Mic and system audio arrive asynchronously at different rates
- Ring buffer accumulates samples until both streams have aligned windows (50ms)
- Professional mixing applies RMS-based ducking to prevent system audio from drowning out microphone
- Uses `VecDeque` for efficient windowed processing

### 2. Thread Safety and Async Boundaries

**Recording State** (recording_state.rs):
```rust
pub struct RecordingState {
    is_recording: Arc<AtomicBool>,
    audio_sender: Arc<RwLock<Option<mpsc::UnboundedSender<AudioChunk>>>>,
    // ...
}
```

**Key Pattern**: Use `Arc<RwLock<T>>` for shared state across async tasks, `Arc<AtomicBool>` for simple flags.

### 3. Error Handling and Logging

**Performance-Aware Logging** (lib.rs):
```rust
#[cfg(debug_assertions)]
macro_rules! perf_debug {
    ($($arg:tt)*) => { log::debug!($($arg)*) };
}

#[cfg(not(debug_assertions))]
macro_rules! perf_debug {
    ($($arg:tt)*) => {};  // Zero overhead in release builds
}
```

**Usage**: Use `perf_debug!()` and `perf_trace!()` for hot-path logging that should be eliminated in production.

### 4. Frontend State Management

**Sidebar Context** (components/Sidebar/SidebarProvider.tsx):
- Global state for meetings list, current meeting, recording status
- Communicates with the Rust/Tauri core through Tauri commands and events
- Keeps React state synchronized with native recording, meeting, transcript, and summary state

**Pattern**: Tauri commands update Rust state → Emit events → Frontend listeners update React state → Context propagates to components

## Common Development Tasks

### Adding a New Audio Device Platform

1. Create platform file: `audio/devices/platform/{platform_name}.rs`
2. Implement device enumeration for the platform
3. Add platform-specific configuration in `audio/devices/configuration.rs`
4. Update `audio/devices/platform/mod.rs` to export new platform functions
5. Test with `cargo check` and platform-specific device tests

### Adding a New Tauri Command

1. Define command in `src/lib.rs`:
   ```rust
   #[tauri::command]
   async fn my_command(arg: String) -> Result<String, String> { /* ... */ }
   ```
2. Register in `tauri::Builder`:
   ```rust
   .invoke_handler(tauri::generate_handler![
       start_recording,
       my_command,  // Add here
   ])
   ```
3. Call from frontend:
   ```typescript
   const result = await invoke<string>('my_command', { arg: 'value' });
   ```

### Modifying Audio Pipeline Behavior

**Location**: `frontend/src-tauri/src/audio/pipeline.rs`

Key components:
- `AudioMixerRingBuffer`: Manages mic + system audio synchronization
- `ProfessionalAudioMixer`: RMS-based ducking and mixing
- `AudioPipelineManager`: Orchestrates VAD, mixing, and distribution

**Testing Audio Changes**:
```bash
# Enable verbose audio logging
RUST_LOG=app_lib::audio=debug ./clean_run.sh

# Monitor audio metrics in real-time
# Check Developer Console in the app (Cmd+Shift+I on macOS)
```

### Tauri Backend Development

Current app behavior should be implemented in the Rust/Tauri core, not in the archived Python backend. Add new frontend-facing behavior through Tauri commands/events and existing Rust services under `frontend/src-tauri/src`.

Do not add new endpoints to `backend/app/main.py`; that FastAPI code is legacy archive material only.

## Testing and Debugging

### Frontend Debugging

**Enable Rust Logging**:
```bash
# macOS
RUST_LOG=debug ./clean_run.sh

# Windows (PowerShell)
$env:RUST_LOG="debug"; ./clean_run_windows.bat
```

**Developer Tools**:
- Open DevTools: `Cmd+Shift+I` (macOS) or `Ctrl+Shift+I` (Windows)
- Console Toggle: Built into app UI (console icon)
- View Rust logs: Check terminal output

### Audio Pipeline Debugging

**Key Metrics** (emitted by pipeline):
- Buffer sizes (mic/system)
- Mixing window count
- VAD detection rate
- Dropped chunk warnings

**Monitor via Developer Console**: The app includes real-time metrics display when recording.

## Platform-Specific Notes

### macOS
- **Audio Capture**: Uses ScreenCaptureKit for system audio (macOS 13+)
- **GPU**: Metal + CoreML automatically enabled
- **Permissions**: Requires microphone + screen recording permissions
- **System Audio**: Requires virtual audio device (BlackHole) for system capture

### Windows
- **Audio Capture**: Uses WASAPI (Windows Audio Session API)
- **GPU**: CUDA (NVIDIA) or Vulkan (AMD/Intel) via Cargo features
- **Build Tools**: Requires Visual Studio Build Tools with C++ workload
- **System Audio**: Uses WASAPI loopback for system capture

### Linux
- **Audio Capture**: ALSA/PulseAudio
- **GPU**: CUDA (NVIDIA) or Vulkan via Cargo features
- **Dependencies**: Requires cmake, llvm, libomp

## Performance Optimization Guidelines

### Audio Processing
- Use `perf_debug!()` / `perf_trace!()` for hot-path logging (zero cost in release)
- Batch audio metrics using `AudioMetricsBatcher` (pipeline.rs)
- Pre-allocate buffers with `AudioBufferPool` (buffer_pool.rs)
- VAD filtering reduces Whisper load by ~70% (only processes speech)

### Whisper Transcription
- **Model Selection**: Balance accuracy vs speed
  - Development: `base` or `small` (fast iteration)
  - Production: `medium` or `large-v3` (best quality)
- **GPU Acceleration**: 5-10x faster than CPU
- **Parallel Processing**: Available in `whisper_engine/parallel_processor.rs` for batch workloads

### Frontend Performance
- React state updates batched via Sidebar context
- Transcript rendering virtualized for large meetings
- Audio level monitoring throttled to 60fps

## Important Constraints and Gotchas

1. **Audio Chunk Size**: Pipeline expects consistent 48kHz sample rate. Resampling happens at capture time.

2. **Platform Audio Quirks**:
   - macOS: ScreenCaptureKit requires macOS 13+, needs screen recording permission
   - Windows: WASAPI exclusive mode can conflict with other apps
   - System audio requires virtual device (BlackHole on macOS, WASAPI loopback on Windows)

3. **Whisper Model Loading**: Models are loaded once and cached. Changing models requires app restart or manual unload/reload.

4. **No Separate Backend Dependency**: Meeting persistence, transcription, and LLM features are handled by the Tauri app. Do not reintroduce the archived FastAPI backend as a supported requirement.

5. **Legacy FastAPI Security Context**: The archived FastAPI/CORS behavior is unsupported legacy code and must not be treated as a supported production API.

6. **File Paths**: Use Tauri's path APIs (`downloadDir`, etc.) for cross-platform compatibility. Never hardcode paths.

7. **Audio Permissions**: Request permissions early. macOS requires both microphone AND screen recording for system audio.

## Repository-Specific Conventions

- **Logging Format**: Rust logs should include enough module context to diagnose app behavior
- **Error Handling**: Rust uses `anyhow::Result`, frontend uses try-catch with user-friendly messages
- **Naming**: Audio devices use "microphone" and "system" consistently (not "input"/"output")
- **Git Branches**:
  - `main`: Stable releases
  - `fix/*`: Bug fixes
  - `enhance/*`: Feature enhancements
  - Current: `feat/clawscribe-productization-auth-theme-exports` (ClawScribe productization, auth, settings, exports)

## Key Files Reference

**Core Coordination**:
- [frontend/src-tauri/src/lib.rs](frontend/src-tauri/src/lib.rs) - Main Tauri entry point, command registration
- [frontend/src-tauri/src/audio/mod.rs](frontend/src-tauri/src/audio/mod.rs) - Audio module exports
- [frontend/src-tauri/src/database/mod.rs](frontend/src-tauri/src/database/mod.rs) - Local database module

**Audio System**:
- [frontend/src-tauri/src/audio/recording_manager.rs](frontend/src-tauri/src/audio/recording_manager.rs) - Recording orchestration
- [frontend/src-tauri/src/audio/pipeline.rs](frontend/src-tauri/src/audio/pipeline.rs) - Audio mixing and VAD
- [frontend/src-tauri/src/audio/recording_saver.rs](frontend/src-tauri/src/audio/recording_saver.rs) - Audio file writing

**UI Components**:
- [frontend/src/app/page.tsx](frontend/src/app/page.tsx) - Main recording interface
- [frontend/src/components/Sidebar/SidebarProvider.tsx](frontend/src/components/Sidebar/SidebarProvider.tsx) - Global state management

**Whisper Integration**:
- [frontend/src-tauri/src/whisper_engine/whisper_engine.rs](frontend/src-tauri/src/whisper_engine/whisper_engine.rs) - Whisper model management and transcription
