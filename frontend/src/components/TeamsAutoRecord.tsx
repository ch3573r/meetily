"use client";

import { useEffect, useRef } from "react";
import { toast } from "sonner";
import { useRecordingState } from "@/contexts/RecordingStateContext";
import { teamsDetectionService } from "@/services/teamsDetectionService";
import { getTeamsDetectionMode } from "@/lib/autoRecord";

const POLL_INTERVAL_MS = 12_000;

/**
 * Background poller: when "auto-record on Teams detection" is enabled, starts a
 * recording once per detected meeting. Re-arms when the meeting ends, so a
 * manual stop mid-meeting isn't immediately overridden, and a new meeting later
 * still triggers. Mounted once (in the layout) inside the recording provider.
 */
export function TeamsAutoRecord() {
  const recordingState = useRecordingState();
  // Refs so the interval closure always sees fresh values.
  const isRecordingRef = useRef(recordingState.isRecording);
  const armedRef = useRef(true);

  useEffect(() => {
    isRecordingRef.current = recordingState.isRecording;
  }, [recordingState.isRecording]);

  useEffect(() => {
    let cancelled = false;

    const tick = async () => {
      const mode = getTeamsDetectionMode();
      if (mode === "off") return;
      try {
        const status = await teamsDetectionService.getStatus();
        if (cancelled) return;

        // Meeting ended (or none) → re-arm for the next one.
        if (!status.detected) {
          armedRef.current = true;
          return;
        }
        // Detected (once per meeting, and only if not already recording).
        if (status.detected && armedRef.current && !isRecordingRef.current) {
          armedRef.current = false;
          if (mode === "auto") {
            window.dispatchEvent(new CustomEvent("start-recording-from-sidebar"));
          } else {
            // Prompt-only: ask before recording.
            toast("Teams meeting detected", {
              description: "Start recording this meeting?",
              duration: 20_000,
              action: {
                label: "Record",
                onClick: () =>
                  window.dispatchEvent(new CustomEvent("start-recording-from-sidebar")),
              },
            });
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
