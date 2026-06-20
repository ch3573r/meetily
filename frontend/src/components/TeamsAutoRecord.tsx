"use client";

import { useEffect, useRef } from "react";
import { toast } from "sonner";
import { useRecordingState } from "@/contexts/RecordingStateContext";
import { teamsDetectionService } from "@/services/teamsDetectionService";
import { getTeamsDetectionMode } from "@/lib/autoRecord";

const POLL_INTERVAL_MS = 12_000;
// Re-arm only after the meeting has been gone for several consecutive polls.
// A single missed window-title read (detection flickers to not-detected for one
// tick) must not re-arm: otherwise a user who manually stopped mid-meeting gets
// auto-started again on the very next detected tick. ~3 misses ≈ 36s.
const REARM_MISS_THRESHOLD = 3;

/**
 * Background poller: when "auto-record on Teams detection" is enabled, starts a
 * recording once per detected meeting. Re-arms only after detection has been
 * absent for several consecutive polls, so a manual stop mid-meeting isn't
 * overridden by a transient detection flicker, while a genuinely new meeting
 * later still triggers. Mounted once (in the layout) inside the recording
 * provider.
 */
export function TeamsAutoRecord() {
  const recordingState = useRecordingState();
  // Refs so the interval closure always sees fresh values.
  const isRecordingRef = useRef(recordingState.isRecording);
  const armedRef = useRef(true);
  // Consecutive not-detected polls, for debounced re-arming.
  const missCountRef = useRef(0);

  useEffect(() => {
    isRecordingRef.current = recordingState.isRecording;
  }, [recordingState.isRecording]);

  useEffect(() => {
    let cancelled = false;

    const promptToRecord = () =>
      toast("Teams meeting detected", {
        description: "Start recording this meeting?",
        duration: 20_000,
        action: {
          label: "Record",
          onClick: () =>
            window.dispatchEvent(new CustomEvent("start-recording-from-sidebar")),
        },
      });

    const tick = async () => {
      const mode = getTeamsDetectionMode();
      if (mode === "off") return;
      try {
        const status = await teamsDetectionService.getStatus();
        if (cancelled) return;

        // Meeting ended (or none). Re-arm only after enough consecutive misses
        // that this isn't just a one-tick detection flicker.
        if (!status.detected) {
          missCountRef.current += 1;
          if (missCountRef.current >= REARM_MISS_THRESHOLD) {
            armedRef.current = true;
          }
          return;
        }
        // Detected → reset the miss streak.
        missCountRef.current = 0;

        // Once per meeting, and only if not already recording.
        if (armedRef.current && !isRecordingRef.current) {
          armedRef.current = false;
          // The backend detector is read-only and only ever recommends a prompt
          // (recording_safety.automaticRecordingAllowed is always false). "auto"
          // mode is the user's explicit opt-in to override that default, so we
          // honor it — but never silently: surface a visible toast so an
          // auto-started recording always has a user-facing signal.
          if (mode === "auto" && status.recordingSafety?.automaticRecordingAllowed === false) {
            window.dispatchEvent(new CustomEvent("start-recording-from-sidebar"));
            toast("Recording started", {
              description: "Auto-started for a detected Teams meeting.",
              duration: 8_000,
            });
          } else if (mode === "auto") {
            window.dispatchEvent(new CustomEvent("start-recording-from-sidebar"));
          } else {
            // Prompt-only: ask before recording.
            promptToRecord();
          }
        }
      } catch {
        // Detection unavailable this tick — try again next interval.
      }
    };

    const id = window.setInterval(tick, POLL_INTERVAL_MS);
    void tick();
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, []);

  return null;
}
