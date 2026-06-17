"use client";

// Persisted toggle: auto-start recording when a Teams meeting is detected.
// Kept in localStorage so the settings toggle and the background poller share
// it without prop drilling.

export const AUTO_RECORD_STORAGE_KEY = "clawscribe.autoRecordOnTeams";
const MODE_KEY = "clawscribe.teamsDetectionMode";

// How the app reacts when a Teams meeting is detected:
//   off    — do nothing (no polling-driven action)
//   prompt — show a "start recording?" prompt once per meeting
//   auto   — silently auto-start recording once per meeting
export type TeamsDetectionMode = "off" | "prompt" | "auto";

export function getTeamsDetectionMode(): TeamsDetectionMode {
  if (typeof window === "undefined") return "off";
  try {
    const v = window.localStorage.getItem(MODE_KEY);
    if (v === "off" || v === "prompt" || v === "auto") return v;
    // Migrate the legacy boolean auto-record flag.
    if (window.localStorage.getItem(AUTO_RECORD_STORAGE_KEY) === "true") return "auto";
    return "off";
  } catch {
    return "off";
  }
}

export function setTeamsDetectionMode(mode: TeamsDetectionMode): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(MODE_KEY, mode);
    // Keep the legacy flag in sync for any other reader.
    window.localStorage.setItem(AUTO_RECORD_STORAGE_KEY, mode === "auto" ? "true" : "false");
  } catch {
    // ignore storage failures
  }
  window.dispatchEvent(new CustomEvent("clawscribe-autorecord-changed"));
}

export function getAutoRecordEnabled(): boolean {
  return getTeamsDetectionMode() === "auto";
}
