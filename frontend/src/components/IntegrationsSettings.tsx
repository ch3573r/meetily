"use client";

import { useEffect, useMemo, useState } from "react";
import type { ElementType, ReactNode } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  Cloud,
  FileCheck2,
  ListTodo,
  NotebookTabs,
  RefreshCw,
  ShieldCheck,
  Video,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { teamsDetectionService, TeamsDetectionStatus } from "@/services/teamsDetectionService";

type AddonState = "ready" | "prompt" | "planned" | "provider" | "advanced";

function stateBadge(state: AddonState) {
  switch (state) {
    case "ready":
      return "Ready";
    case "prompt":
      return "Prompt only";
    case "provider":
      return "Provider";
    case "advanced":
      return "Advanced";
    case "planned":
    default:
      return "Not implemented";
  }
}

function stateClasses(state: AddonState) {
  switch (state) {
    case "ready":
      return "border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-200";
    case "prompt":
      return "border-blue-200 bg-blue-50 text-blue-800 dark:border-blue-900 dark:bg-blue-950/40 dark:text-blue-200";
    case "provider":
      return "border-primary/20 bg-primary/10 text-primary";
    case "advanced":
      return "border-amber-200 bg-amber-50 text-amber-800 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-200";
    case "planned":
    default:
      return "border-muted bg-muted text-muted-foreground";
  }
}

interface AddonPanelProps {
  icon: ElementType;
  title: string;
  state: AddonState;
  detail: string;
  children?: ReactNode;
}

function AddonPanel({ icon: Icon, title, state, detail, children }: AddonPanelProps) {
  return (
    <section className="rounded-lg border border-border bg-card p-5 shadow-sm">
      <div className="flex items-start justify-between gap-4">
        <div className="flex min-w-0 items-start gap-3">
          <div className="rounded-md border border-border bg-background p-2">
            <Icon className="h-5 w-5 text-primary" />
          </div>
          <div className="min-w-0">
            <h3 className="text-base font-semibold text-card-foreground">{title}</h3>
            <p className="mt-1 text-sm text-muted-foreground">{detail}</p>
          </div>
        </div>
        <span className={`shrink-0 rounded-full border px-2.5 py-1 text-xs font-medium ${stateClasses(state)}`}>
          {stateBadge(state)}
        </span>
      </div>
      {children && <div className="mt-4">{children}</div>}
    </section>
  );
}

function DetectionSummary({ status }: { status: TeamsDetectionStatus | null }) {
  if (!status) {
    return (
      <p className="text-sm text-muted-foreground">
        Status has not been checked in this session.
      </p>
    );
  }

  return (
    <div className="grid gap-3 text-sm sm:grid-cols-2 lg:grid-cols-4">
      <div className="rounded-md border border-border bg-background p-3">
        <p className="text-xs text-muted-foreground">Platform</p>
        <p className="font-medium text-foreground">{status.platform}</p>
      </div>
      <div className="rounded-md border border-border bg-background p-3">
        <p className="text-xs text-muted-foreground">Status</p>
        <p className="font-medium text-foreground">{status.status}</p>
      </div>
      <div className="rounded-md border border-border bg-background p-3">
        <p className="text-xs text-muted-foreground">Confidence</p>
        <p className="font-medium text-foreground">{Math.round(status.confidence * 100)}%</p>
      </div>
      <div className="rounded-md border border-border bg-background p-3">
        <p className="text-xs text-muted-foreground">Action</p>
        <p className="font-medium text-foreground">{status.nextRecommendedAction}</p>
      </div>
    </div>
  );
}

export function IntegrationsSettings() {
  const [teamsStatus, setTeamsStatus] = useState<TeamsDetectionStatus | null>(null);
  const [isCheckingTeams, setIsCheckingTeams] = useState(false);
  const [teamsError, setTeamsError] = useState<string | null>(null);

  const teamsState: AddonState = useMemo(() => {
    if (!teamsStatus) return "prompt";
    if (!teamsStatus.supported) return "planned";
    return teamsStatus.recordingSafety.automaticRecordingAllowed ? "ready" : "prompt";
  }, [teamsStatus]);

  const checkTeamsDetection = async () => {
    setIsCheckingTeams(true);
    setTeamsError(null);
    try {
      const status = await teamsDetectionService.getStatus();
      setTeamsStatus(status);
    } catch (error) {
      setTeamsError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsCheckingTeams(false);
    }
  };

  useEffect(() => {
    void checkTeamsDetection();
  }, []);

  return (
    <div className="space-y-5">
      <div className="rounded-lg border border-border bg-card p-5">
        <div className="flex items-start gap-3">
          <ShieldCheck className="mt-0.5 h-5 w-5 text-primary" />
          <div>
            <h2 className="text-lg font-semibold text-card-foreground">Add-ons and integrations</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Current status for meeting detection, handoff, exports, and advanced providers.
            </p>
          </div>
        </div>
      </div>

      <AddonPanel
        icon={Video}
        title="Teams meeting detection"
        state={teamsState}
        detail="Windows detector for Teams desktop and browser meetings. Current safety mode recommends recording; it does not auto-start recording yet."
      >
        <div className="space-y-3">
          <DetectionSummary status={teamsStatus} />
          {teamsStatus?.reason && (
            <p className="rounded-md border border-border bg-background p-3 text-sm text-muted-foreground">
              {teamsStatus.reason}
            </p>
          )}
          {teamsError && (
            <div className="flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" />
              <span>{teamsError}</span>
            </div>
          )}
          <Button type="button" variant="outline" onClick={checkTeamsDetection} disabled={isCheckingTeams}>
            <RefreshCw className={`mr-2 h-4 w-4 ${isCheckingTeams ? "animate-spin" : ""}`} />
            Check Teams detection
          </Button>
        </div>
      </AddonPanel>

      <AddonPanel
        icon={Cloud}
        title="OpenClaw handoff"
        state="provider"
        detail="Configured from Summary → OpenClaw provider. It can receive meeting.completed payloads and return the same notes contract as the other providers."
      >
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <CheckCircle2 className="h-4 w-4 text-emerald-600" />
          <span>Available as a summary provider and handoff target.</span>
        </div>
      </AddonPanel>

      <AddonPanel
        icon={NotebookTabs}
        title="OneNote export"
        state="planned"
        detail="Microsoft Graph design exists, but live Microsoft sign-in and page creation are not implemented in the app yet."
      />

      <AddonPanel
        icon={ListTodo}
        title="Planner task export"
        state="planned"
        detail="Planner mapping and test plan exist, but live task creation is not implemented in the app yet."
      />

      <AddonPanel
        icon={FileCheck2}
        title="Advanced: Codex app-server"
        state="advanced"
        detail="Configured from Summary → Advanced: Codex app-server. This is a bundled runtime provider, not a global Codex CLI integration."
      />
    </div>
  );
}
