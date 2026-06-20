"use client";

export const AUTO_UPDATE_CHECK_STORAGE_KEY = "clawscribe.autoUpdateCheck";
export const AUTO_UPDATE_CHECK_CHANGED_EVENT = "clawscribe:auto-update-check-changed";

export function getAutoUpdateCheckEnabled(): boolean {
  if (typeof window === "undefined") return true;

  try {
    return window.localStorage.getItem(AUTO_UPDATE_CHECK_STORAGE_KEY) !== "false";
  } catch {
    return true;
  }
}

export function setAutoUpdateCheckEnabled(enabled: boolean): void {
  if (typeof window === "undefined") return;

  try {
    window.localStorage.setItem(AUTO_UPDATE_CHECK_STORAGE_KEY, String(enabled));
  } catch {
    // Keep the in-session event working even when storage is unavailable.
  }

  window.dispatchEvent(
    new CustomEvent<boolean>(AUTO_UPDATE_CHECK_CHANGED_EVENT, {
      detail: enabled,
    }),
  );
}
