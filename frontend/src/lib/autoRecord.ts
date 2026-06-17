"use client";

// Persisted toggle: auto-start recording when a Teams meeting is detected.
// Kept in localStorage so the settings toggle and the background poller share
// it without prop drilling.

export const AUTO_RECORD_STORAGE_KEY = "clawscribe.autoRecordOnTeams";

export function getAutoRecordEnabled(): boolean {
  if (typeof window === "undefined") return false;
  try {
    return window.localStorage.getItem(AUTO_RECORD_STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

export function setAutoRecordEnabled(enabled: boolean): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(AUTO_RECORD_STORAGE_KEY, enabled ? "true" : "false");
  } catch {
    // ignore storage failures
  }
  // Notify same-tab listeners (storage event only fires cross-tab).
  window.dispatchEvent(new CustomEvent("clawscribe-autorecord-changed"));
}
