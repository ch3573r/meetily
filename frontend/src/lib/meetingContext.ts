"use client";

// Per-meeting "Add context" text for AI summaries. Persisted in localStorage so
// it survives reopening a meeting and is applied on every generate/regenerate,
// keyed by meeting id. (UI preference, not meeting content — no DB column.)

const PREFIX = "clawscribe.meetingContext.";

export function getMeetingContext(meetingId: string): string {
  if (typeof window === "undefined" || !meetingId) return "";
  try {
    return window.localStorage.getItem(PREFIX + meetingId) ?? "";
  } catch {
    return "";
  }
}

export function setMeetingContext(meetingId: string, context: string): void {
  if (typeof window === "undefined" || !meetingId) return;
  try {
    if (context.trim()) {
      window.localStorage.setItem(PREFIX + meetingId, context);
    } else {
      window.localStorage.removeItem(PREFIX + meetingId);
    }
  } catch {
    // Persisting context is best-effort; generation still works without it.
  }
}
