# ClawScribe QA Baseline Results

Date: 2026-06-15
Worker: QA/release subagent F
Branch: `feat/clawscribe-productization-auth-theme-exports`
Host: Linux, not Windows

These results are a safe baseline only. They do not verify Windows launch,
installer behavior, WebView2, Teams desktop/browser runtime detection, WASAPI
recording, OneNote, Planner, or Windows OpenClaw handoff.

## Repo State

Command:

```bash
git status --short --branch
```

Result:

```text
## feat/clawscribe-productization-auth-theme-exports
?? docs/productization/
```

Later during the pass, other worker docs also appeared under `docs/auth/`,
`docs/integrations/`, `docs/verification/`, and `docs/productization/`. Those
files were treated as collaborator work and not modified by QA.

Command:

```bash
git rev-parse HEAD && git rev-parse --abbrev-ref HEAD
```

Result:

```text
70628e0fc34dc8032bb42cc016180a312c28f20f
feat/clawscribe-productization-auth-theme-exports
```

## Disk And Sidecars

Command:

```bash
df -h . frontend/src-tauri/target 2>/dev/null || df -h .
```

Result:

```text
Filesystem           Size  Used Avail Use% Mounted on
/dev/mapper/rl-root   51G   42G  8.4G  84% /
Filesystem           Size  Used Avail Use% Mounted on
/dev/mapper/rl-root   51G   42G  8.4G  84% /
```

Command:

```bash
find frontend/src-tauri/binaries -maxdepth 1 -type f -printf '%f %s bytes\n' | sort
```

Result:

```text
ffmpeg-x86_64-unknown-linux-gnu 79826272 bytes
llama-helper-x86_64-unknown-linux-gnu 24921424 bytes
```

## Frontend Build

Command:

```bash
cd frontend
pnpm build
```

Result: passed.

Important output from the integrated coordinator rerun:

```text
> clawscribe@0.5.0-alpha.1 build /path/to/clawscribe/frontend
> next build

Compiled successfully
Linting and checking validity of types ...
Generating static pages (11/11)
```

Routes generated:

```text
static /
static /_not-found
static /meeting-details
ssg /notes/[id]
static /settings
```

Note: the QA worker first observed the package before all branding/version edits
had landed in the shared worktree. The coordinator rerun above is the current
integrated result.

## Explicit Typecheck

Command:

```bash
cd frontend
pnpm exec tsc --noEmit --pretty false
```

Result: failed with the known baseline blocker.

Output:

```text
tests/lib/blocknote-markdown.test.ts(1,57): error TS2307: Cannot find module 'bun:test' or its corresponding type declarations.
```

## Lint

Command:

```bash
cd frontend
CI=1 pnpm lint
```

Result: failed because `next lint` still prompts for ESLint setup.

Output:

```text
> clawscribe@0.5.0-alpha.1 lint /path/to/clawscribe/frontend
> next lint

? How would you like to configure ESLint? https://nextjs.org/docs/basic-features/eslint
>  Strict (recommended)
   Base
   Cancel
ELIFECYCLE Command failed with exit code 1.
```

## Linux Tauri Check

Command:

```bash
cargo check --manifest-path frontend/src-tauri/Cargo.toml
```

Result: passed.

Duration:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 43s
```

Notable warnings:

```text
warning: patch for the non root package will be ignored, specify patch at the workspace root
warning: profiles for the non root package will be ignored, specify profiles at the workspace root
warning: unused import: `std::io::Read`
warning: `clawscribe` (lib) generated 10 warnings
```

Build-script output confirmed Linux CPU-only mode and cached FFmpeg:

```text
Building ClawScribe for: linux
Linux: Using CPU-only mode (no GPU or BLAS acceleration)
Found cached FFmpeg binary: ffmpeg-x86_64-unknown-linux-gnu
FFmpeg verification passed
```

This does not verify Windows runtime behavior.

## OpenAI Auth Unit Tests

Command:

```bash
cargo test --manifest-path frontend/src-tauri/Cargo.toml openai::auth --lib
```

Result: passed.

The command waited on a Cargo target lock while other worker test commands were
running, then completed successfully.

Output:

```text
running 9 tests
test openai::auth::tests::disabled_mode_overrides_legacy_key_in_status ... ok
test openai::auth::tests::legacy_api_key_reports_api_key_mode_without_stored_config ... ok
test openai::auth::tests::oauth_pkce_requires_https_for_non_localhost_endpoints ... ok
test openai::auth::tests::oauth_pkce_normalization_trims_public_metadata ... ok
test openai::auth::tests::openclaw_codex_managed_normalization_trims_endpoint_metadata ... ok
test openai::auth::tests::openclaw_codex_managed_auth_is_request_ready_without_storing_chatgpt_tokens ... ok
test openai::auth::tests::openclaw_codex_managed_requires_https_for_non_localhost_endpoint ... ok
test openai::auth::tests::public_oauth_pkce_metadata_is_not_reported_as_request_ready ... ok
test openai::auth::tests::pkce_authorization_request_uses_s256_without_claiming_token_exchange ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 197 filtered out; finished in 0.00s
```

Warnings:

```text
warning: `clawscribe` (lib test) generated 13 warnings
```

## Secret Search

Command:

```bash
rg -n --hidden --pcre2 -S "(sk-proj-[A-Za-z0-9_-]{20,}|sk-[A-Za-z0-9_-]{32,}|sk-ant-[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{20,}|AIza[0-9A-Za-z_-]{30,})" --glob '!**/node_modules/**' --glob '!**/target/**' --glob '!frontend/out/**' --glob '!frontend/vs_buildtools.exe' --glob '!**/*.png' --glob '!**/*.jpg' --glob '!**/*.gif' .
```

Result: no matches. `rg` exited `1`, which means no matches for this command.

Command:

```bash
rg -n --hidden --pcre2 -S "(?i)(api[_-]?key|bearer|token|secret|password|client[_-]?secret|authorization)" --glob '!**/node_modules/**' --glob '!**/target/**' --glob '!frontend/out/**' --glob '!frontend/vs_buildtools.exe' --glob '!**/*.png' --glob '!**/*.jpg' --glob '!**/*.gif' .
```

Result: matches found. Reviewed at baseline level only.

Expected categories included:

- GitHub Actions `${{ secrets.* }}` references.
- settings repository fields such as `openaiApiKey`, `customOpenAIConfig`, and
  provider API-key columns.
- OpenClaw bearer-token config/status code paths.
- docs describing placeholders or secret-handling requirements.
- token-count terminology in LLM code.

Release risks to keep visible:

- `frontend/src-tauri/src/api/api.rs` includes a helper that logs a truncated
  auth token.
- `frontend/src-tauri/src/analytics/commands.rs` contains a hardcoded PostHog
  project key. This may be a public analytics key, but it should be explicitly
  accepted or moved/configured before stronger secret-hygiene claims.

## Branding Search

Command:

```bash
rg -n --hidden -S "Meetily|meetily|MEETILY|Zackriya" --glob '!**/node_modules/**' --glob '!**/target/**' --glob '!frontend/out/**' --glob '!frontend/vs_buildtools.exe' --glob '!**/*.png' --glob '!**/*.jpg' --glob '!**/*.gif' .
```

Result: matches found. Current classifications are documented in
`docs/productization/branding-search-report.md`.

Observed categories:

- upstream attribution and legal notices;
- compatibility names such as `meetily-recordings`,
  `openclaw.meetily-submission*.v1`, and `MEETILY_OPENCLAW_*`;
- previous-install migration text;
- upstream/developer docs;
- archived/legacy code;
- workflow artifact names and older GitHub workflow references.

Command:

```bash
rg -n --hidden -S "ClawScribe|clawscribe|OpenClaw" --glob '!**/node_modules/**' --glob '!**/target/**' --glob '!frontend/out/**' --glob '!frontend/vs_buildtools.exe' --glob '!**/*.png' --glob '!**/*.jpg' --glob '!**/*.gif' .
```

Result: matches found across current product docs, package metadata, UI strings,
Tauri config, OpenClaw handoff, and productization docs.

## Checks Not Run

Not run in this Linux QA pass:

- Windows installer build.
- GitHub Actions Windows release workflow.
- Windows `build-windows-release.ps1 -CheckOnly`.
- Windows app launch/install/uninstall.
- Teams desktop/browser runtime detection.
- WASAPI microphone/system-audio recording.
- Windows OpenClaw handoff from the recorder.
- Microsoft login.
- OneNote export.
- Planner export.

## Coordinator Should Rerun

After integrations land, rerun:

```bash
git status --short --branch
cd frontend && pnpm build
cd frontend && pnpm exec tsc --noEmit --pretty false
cd frontend && CI=1 pnpm lint
cargo check --manifest-path frontend/src-tauri/Cargo.toml
cargo test --manifest-path frontend/src-tauri/Cargo.toml openai::auth --lib
cargo test --manifest-path frontend/src-tauri/Cargo.toml teams_detection --lib
```

Then rerun the release-gate secret and branding searches from
`docs/productization/release-gate.md`.

At the release checkpoint, run `docs/verification/windows-final-checklist.md` on
the Windows recorder before making any Windows runtime claims.
