# ClawScribe Privacy Policy

*Last updated: 2026-06-19 — applies to ClawScribe 0.5.0-alpha.4 and later.*

ClawScribe is a local-first, open-source (MIT) meeting recorder and summarizer.
Your meeting data stays on your device unless **you** explicitly configure a
feature that sends it elsewhere. This policy describes exactly what stays local
and what leaves the device, and only when.

## Local-first by default

With no optional integrations configured, ClawScribe processes everything on your
device and sends nothing off it:

- **Audio recording** — captured and stored locally; never uploaded.
- **Transcription** — runs locally (Whisper / Parakeet / Nemotron models that you
  download and run on your own machine).
- **Storage** — meetings, transcripts, and summaries live in a local database and
  local files under your user profile.
- **You own your data** — export or delete it at any time; no account is required
  to record, transcribe, or summarize.

## What leaves your device — only when you turn it on

ClawScribe sends data off-device **only** for the optional features you configure.
Each is off until you set it up:

- **Cloud AI summaries.** If you choose a cloud summarization provider, your
  transcript (and any context you add) is sent to that provider to generate the
  summary. Providers you can choose include OpenAI, OpenAI-compatible endpoints,
  an OpenClaw endpoint, and the bundled Codex/ChatGPT path. Built-in local AI and
  local Ollama keep this on-device. Each third party handles your data under its
  own privacy policy.
- **OpenClaw handoff.** If enabled, completed meeting artifacts are sent to the
  OpenClaw endpoint you configure.
- **Microsoft 365 export (Microsoft Graph).** If you sign in and export, the
  selected summary/notes/tasks are sent to Microsoft (OneNote/Planner) under your
  own Microsoft account and Microsoft's privacy terms. ClawScribe uses a
  delegated, public-client OAuth flow; sign-in tokens are stored in your OS
  credential store, not transmitted to us.
- **Model downloads.** Transcription models are downloaded at runtime from their
  publishers (e.g. Hugging Face) the first time you select them.

We (the ClawScribe project) do not operate a backend that receives your meeting
content. There is no ClawScribe account, license server, or activation check.

## Usage analytics (optional, off by default)

If — and only if — you enable analytics in Settings, ClawScribe sends anonymized,
aggregate usage data via [PostHog](https://posthog.com):

**Collected when enabled:** feature-usage patterns, session frequency/duration,
performance metrics (e.g. transcription timings, error/crash counts), and app
version/platform — tied only to a generated random ID.

**Never collected:** meeting audio, transcripts, summaries, titles, file names,
participant names, LLM prompts/responses, API keys, or any meeting content.

Analytics is **disabled until you opt in**, can be turned off again at any time,
and the full implementation is open source for review.

## Your rights

- **Access / export / delete** all local data at any time.
- **Disable** every off-device feature; ClawScribe remains fully functional local-only.
- **Inspect** everything — the source is MIT-licensed and public.

## Security

- Local data is protected by your operating system's file permissions.
- Credentials (API keys, OAuth tokens, OpenClaw bearer tokens) are stored in the
  OS credential store where available.
- Network calls to the services above use TLS.

## Changes

Material changes are reflected in this document in the public repository and noted
in release notes.

## Contact

- **Issues / questions:** open an issue in the ClawScribe GitHub repository.

## Open source

ClawScribe is distributed under the MIT License (it is a fork of Meetily Community
Edition; see `LICENSE.md`, `NOTICE.md`, and `UPSTREAM.md`). You may review, modify,
self-host, and audit every part of its data handling.
