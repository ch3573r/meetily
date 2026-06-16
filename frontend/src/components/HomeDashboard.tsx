"use client";

import { useMemo } from "react";
import {
  ArrowRight,
  ChevronRight,
  Clock3,
  FileText,
  Link2,
  Settings2,
  ShieldCheck,
  Sparkles,
  Upload,
} from "lucide-react";
import { RecordingControls } from "@/components/RecordingControls";
import { useRouter } from "next/navigation";
import { useSidebar } from "@/components/Sidebar/SidebarProvider";
import { useConfig } from "@/contexts/ConfigContext";
import { useImportDialog } from "@/contexts/ImportDialogContext";
import { useRecordingState } from "@/contexts/RecordingStateContext";

interface HomeDashboardProps {
  canRecord: boolean;
  isRecording: boolean;
  isProcessingStop: boolean;
  isRecordingDisabled: boolean;
  barHeights: string[];
  meetingName?: string;
  onRecordingStart: () => void;
  onRecordingStop: (callApi?: boolean) => void;
  onStopInitiated: () => void;
  onTranscriptionError: (message: string) => void;
}

export function HomeDashboard({
  canRecord,
  isRecording,
  isProcessingStop,
  isRecordingDisabled,
  barHeights,
  meetingName,
  onRecordingStart,
  onRecordingStop,
  onStopInitiated,
  onTranscriptionError,
}: HomeDashboardProps) {
  const { meetings, serverAddress } = useSidebar();
  const { selectedDevices, betaFeatures } = useConfig();
  const { openImportDialog } = useImportDialog();
  const recordingState = useRecordingState();
  const router = useRouter();

  const recentMeetings = useMemo(() => meetings.slice(0, 5), [meetings]);

  const greeting = useMemo(() => {
    const h = new Date().getHours();
    if (h < 12) return "Good morning";
    if (h < 18) return "Good afternoon";
    return "Good evening";
  }, []);

  // Honest status: reflect actual recording / microphone state.
  const appStatus = isRecording
    ? { label: "Recording in progress", dot: "bg-red-400", glow: "shadow-[0_0_16px_rgba(248,113,113,0.7)]" }
    : canRecord
      ? { label: "Ready to record", dot: "bg-emerald-400", glow: "shadow-[0_0_16px_rgba(52,211,153,0.7)]" }
      : { label: "Microphone needed", dot: "bg-amber-400", glow: "shadow-[0_0_16px_rgba(251,191,36,0.7)]" };

  return (
    <div className="min-h-screen overflow-y-auto bg-[#081019] px-8 py-7 text-slate-100">
      <div className="mx-auto flex max-w-[1500px] flex-col gap-6">
        <header className="flex items-start justify-between gap-6">
          <div>
            <h1 className="text-4xl font-semibold tracking-tight text-white">
              {greeting}
            </h1>
            <p className="mt-2 text-base text-slate-400">
              Ready to capture and transcribe your meetings.
            </p>
          </div>

          <div className="hidden items-center gap-3 rounded-full border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-300 lg:flex">
            <span className={`flex h-2.5 w-2.5 rounded-full ${appStatus.dot} ${appStatus.glow}`} />
            {appStatus.label}
          </div>
        </header>

        <div className="grid gap-5 xl:grid-cols-[1.25fr_0.9fr]">
          <section className="relative overflow-hidden rounded-3xl border border-white/10 bg-gradient-to-br from-[#111b2a] via-[#0d1724] to-[#0a111c] p-7 shadow-2xl shadow-black/30">
            <div className="absolute right-[-120px] top-[-120px] h-72 w-72 rounded-full bg-cyan-400/10 blur-3xl" />
            <div className="absolute bottom-[-160px] left-[20%] h-72 w-72 rounded-full bg-blue-600/10 blur-3xl" />

            <div className="relative z-10 flex h-full min-h-[265px] flex-col justify-between gap-8">
              <div>
                <div className="inline-flex items-center gap-2 rounded-full border border-cyan-300/20 bg-cyan-300/10 px-3 py-1 text-xs font-medium text-cyan-200">
                  <Sparkles className="h-3.5 w-3.5" />
                  Live capture
                </div>
                <h2 className="mt-5 text-3xl font-semibold text-white">
                  Start a Recording
                </h2>
                <p className="mt-2 max-w-xl text-sm leading-6 text-slate-400">
                  Capture microphone and system audio, stream the transcript,
                  and save the finished meeting for summary and review.
                </p>
              </div>

              <div className="flex flex-col items-start gap-5 sm:flex-row sm:items-center sm:justify-between">
                {canRecord ? (
                  <RecordingControls
                    variant="dashboard"
                    isRecording={recordingState.isRecording}
                    onRecordingStop={onRecordingStop}
                    onRecordingStart={onRecordingStart}
                    onTranscriptReceived={() => {}}
                    onStopInitiated={onStopInitiated}
                    barHeights={barHeights}
                    onTranscriptionError={onTranscriptionError}
                    isRecordingDisabled={isRecordingDisabled}
                    isParentProcessing={isProcessingStop}
                    selectedDevices={selectedDevices}
                    meetingName={meetingName}
                  />
                ) : (
                  <div className="rounded-2xl border border-amber-300/20 bg-amber-300/10 px-4 py-3 text-sm text-amber-100">
                    Microphone access is required before recording.
                  </div>
                )}

                <div className="grid gap-2 text-sm text-slate-400 sm:text-right">
                  <div className="font-medium text-slate-200">
                    {isRecording ? "Recording in progress" : "Ready to start"}
                  </div>
                  <div>Use the recording controls to pause or stop.</div>
                </div>
              </div>
            </div>
          </section>

          <div className="grid gap-5 md:grid-cols-2 xl:grid-cols-1">
            <section className="rounded-3xl border border-white/10 bg-[#0e1723] p-6 shadow-xl shadow-black/20">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <div className="flex items-center gap-2 text-sm font-medium text-slate-300">
                    <ShieldCheck className="h-4 w-4 text-emerald-400" />
                    OpenClaw Handoff
                  </div>
                  <p className="mt-3 text-sm leading-6 text-slate-400">
                    Meeting notes can be submitted into your OpenClaw workspace
                    when processing completes.
                  </p>
                </div>
                <span className="rounded-full border border-slate-500/30 bg-white/[0.04] px-2.5 py-1 text-xs font-medium text-slate-300">
                  Settings
                </span>
              </div>
              <button
                onClick={() => router.push("/settings?tab=general")}
                className="mt-5 inline-flex items-center gap-2 text-sm font-medium text-cyan-300 hover:text-cyan-200"
              >
                View details <ArrowRight className="h-4 w-4" />
              </button>
            </section>

            <section className="rounded-3xl border border-white/10 bg-[#0e1723] p-6 shadow-xl shadow-black/20">
              <div className="flex items-center gap-2 text-sm font-medium text-slate-300">
                <Sparkles className="h-4 w-4 text-cyan-300" />
                AI Meeting Summary
              </div>
              <div className="mt-4 rounded-2xl border border-white/10 bg-white/[0.03] p-4 text-sm leading-6 text-slate-300">
                <p className="font-medium text-white">After recording:</p>
                <ul className="mt-2 space-y-1 text-slate-400">
                  <li>• Key decisions and blockers</li>
                  <li>• Action items with owners</li>
                  <li>• Export-ready meeting notes</li>
                </ul>
              </div>
              <button
                onClick={() => router.push("/settings?tab=summaryModels")}
                className="mt-4 inline-flex items-center gap-2 text-sm font-medium text-cyan-300 hover:text-cyan-200"
              >
                Configure summary <ArrowRight className="h-4 w-4" />
              </button>
            </section>
          </div>
        </div>

        <div className="grid gap-5 xl:grid-cols-[1.25fr_0.9fr]">
          <section className="rounded-3xl border border-white/10 bg-[#0e1723] shadow-xl shadow-black/20">
            <div className="flex items-center justify-between border-b border-white/10 px-6 py-5">
              <div>
                <h2 className="text-lg font-semibold text-white">
                  Recent Meetings
                </h2>
                <p className="mt-1 text-sm text-slate-400">
                  Continue where you left off.
                </p>
              </div>
              <span className="text-sm text-slate-500">
                {meetings.length} total
              </span>
            </div>

            <div className="divide-y divide-white/10">
              {recentMeetings.length > 0 ? (
                recentMeetings.map((meeting) => (
                  <button
                    key={meeting.id}
                    onClick={() =>
                      router.push(`/meeting-details?id=${meeting.id}`)
                    }
                    className="grid w-full grid-cols-[1fr_auto] items-center gap-4 px-6 py-4 text-left text-sm hover:bg-white/[0.03]"
                  >
                    <div className="flex min-w-0 items-center gap-3">
                      <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl border border-white/10 bg-white/[0.04] text-cyan-300">
                        <FileText className="h-4 w-4" />
                      </span>
                      <div className="min-w-0">
                        <div className="truncate font-medium text-slate-100">
                          {meeting.title}
                        </div>
                        <div className="mt-1 flex items-center gap-2 text-xs text-slate-500">
                          <Clock3 className="h-3.5 w-3.5" />
                          Saved meeting
                        </div>
                      </div>
                    </div>
                    <span className="hidden max-w-[260px] truncate text-slate-400 lg:block">
                      Open meeting details
                    </span>
                  </button>
                ))
              ) : (
                <div className="px-6 py-14 text-center text-sm text-slate-500">
                  No meetings yet. Start your first recording from the card
                  above.
                </div>
              )}
            </div>
          </section>

          <section className="rounded-3xl border border-white/10 bg-[#0e1723] p-6 shadow-xl shadow-black/20">
            <h2 className="text-lg font-semibold text-white">Quick Actions</h2>
            <p className="mt-1 text-sm text-slate-400">
              Shortcuts for common setup and capture work.
            </p>

            <div className="mt-5 space-y-3">
              {[
                {
                  icon: Upload,
                  label: "Import Audio/Video",
                  enabled: betaFeatures.importAndRetranscribe,
                  onClick: () => openImportDialog(),
                },
                {
                  icon: Settings2,
                  label: "Transcription Settings",
                  enabled: true,
                  onClick: () =>
                    router.push("/settings?tab=Transcriptionmodels"),
                },
                {
                  icon: Link2,
                  label: "Add-ons",
                  enabled: true,
                  onClick: () => router.push("/settings?tab=integrations"),
                },
              ].map((item) => (
                <button
                  key={item.label}
                  onClick={item.onClick}
                  disabled={!item.enabled}
                  className="flex w-full items-center justify-between rounded-2xl border border-white/10 bg-white/[0.03] px-4 py-3 text-left text-sm text-slate-200 transition hover:border-cyan-300/30 hover:bg-cyan-300/10 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  <span className="flex items-center gap-3">
                    <item.icon className="h-4 w-4 text-cyan-300" />
                    {item.label}
                  </span>
                  <ChevronRight className="h-4 w-4 text-slate-500" />
                </button>
              ))}
            </div>
          </section>
        </div>

        <footer className="grid gap-4 rounded-3xl border border-white/10 bg-[#0e1723] px-6 py-4 text-sm text-slate-400 shadow-xl shadow-black/20 lg:grid-cols-[1fr_auto] lg:items-center">
          <div className="flex items-center gap-3 font-medium text-slate-200">
            <span className={`h-2.5 w-2.5 rounded-full ${appStatus.dot}`} />
            {appStatus.label}
          </div>
          <div className="flex items-center gap-2">
            Backend endpoint {serverAddress || "http://localhost:5167"}
          </div>
        </footer>
      </div>
    </div>
  );
}
