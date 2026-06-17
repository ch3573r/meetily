"use client";

import { invoke } from "@tauri-apps/api/core";

// Beta (experimental, default off): energy-based "Me" / "Participants" source
// attribution for live transcripts. The heuristic isn't reliable yet, so it's
// opt-in. Persisted locally and pushed to the backend, which applies it to
// segments transcribed after the change.

const KEY = "clawscribe.sourceAttribution";

export function getSourceAttribution(): boolean {
  if (typeof window === "undefined") return false;
  try {
    return window.localStorage.getItem(KEY) === "true";
  } catch {
    return false;
  }
}

export async function setSourceAttribution(enabled: boolean): Promise<void> {
  if (typeof window !== "undefined") {
    try {
      window.localStorage.setItem(KEY, String(enabled));
    } catch {
      // best-effort
    }
  }
  try {
    await invoke("set_source_attribution_enabled", { enabled });
  } catch {
    // Backend may not expose it (older build); ignore.
  }
}

/** Push the stored preference to the backend. Call once at app startup. */
export async function applySourceAttribution(): Promise<void> {
  try {
    await invoke("set_source_attribution_enabled", {
      enabled: getSourceAttribution(),
    });
  } catch {
    // ignore
  }
}
