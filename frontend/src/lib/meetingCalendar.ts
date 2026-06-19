"use client";

// Links a recording to a Microsoft calendar event (title + invited attendees),
// for title prefill and attendee context in summaries. Persisted in
// localStorage (UI association, not meeting content — no DB column), mirroring
// meetingContext.ts.
//
// Two slots:
//  - a single "pending" selection chosen for the *next* recording, and
//  - a per-meeting binding keyed by meeting id (consumed from pending on the
//    first summary, since the final meeting id isn't known at record-start).

export interface MeetingCalendarLink {
  eventId: string;
  subject: string | null;
  attendees: { name: string | null; email: string | null }[];
  joinUrl: string | null;
}

const MEETING_PREFIX = "clawscribe.meetingCalendar.";
const PENDING_KEY = "clawscribe.pendingCalendar";

function read(key: string): MeetingCalendarLink | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = window.localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as MeetingCalendarLink) : null;
  } catch {
    return null;
  }
}

function write(key: string, link: MeetingCalendarLink | null): void {
  if (typeof window === "undefined") return;
  try {
    if (link) window.localStorage.setItem(key, JSON.stringify(link));
    else window.localStorage.removeItem(key);
  } catch {
    // Best-effort; the app works without the association.
  }
}

/** The calendar event chosen for the next recording (title prefill source). */
export function getPendingCalendar(): MeetingCalendarLink | null {
  return read(PENDING_KEY);
}
export function setPendingCalendar(link: MeetingCalendarLink | null): void {
  write(PENDING_KEY, link);
}

/** The calendar event bound to a specific meeting. */
export function getMeetingCalendar(meetingId: string): MeetingCalendarLink | null {
  if (!meetingId) return null;
  return read(MEETING_PREFIX + meetingId);
}
export function setMeetingCalendar(
  meetingId: string,
  link: MeetingCalendarLink | null,
): void {
  if (!meetingId) return;
  write(MEETING_PREFIX + meetingId, link);
}

/** Human-readable attendee list for prompts/UI, capped for length. */
export function attendeeNames(link: MeetingCalendarLink, max = 25): string[] {
  return link.attendees
    .map((a) => (a.name?.trim() || a.email?.trim() || "").trim())
    .filter((s) => s.length > 0)
    .slice(0, max);
}
