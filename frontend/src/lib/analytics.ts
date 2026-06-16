// Telemetry has been removed from ClawScribe.
//
// This module is intentionally a no-op shim. It preserves the historical
// `Analytics` API surface so the many existing call sites keep compiling, but
// every method does nothing and no data ever leaves the device. There are no
// network calls, no PostHog, and no persistent identifiers.
//
// Every method accepts variadic args so any existing call signature is
// accepted without a type error.

export interface AnalyticsProperties {
  [key: string]: string | number | boolean | null | undefined;
}

export interface DeviceInfo {
  platform?: string;
  os_version?: string;
  app_version?: string;
  architecture?: string;
  [key: string]: string | number | boolean | null | undefined;
}

export interface UserSession {
  sessionId?: string;
  startedAt?: number;
  [key: string]: unknown;
}

/** No-op analytics. All methods are inert. */
export class Analytics {
  // ── lifecycle ───────────────────────────────────────────────────────────
  static async init(..._args: unknown[]): Promise<void> {}
  static async doInit(..._args: unknown[]): Promise<void> {}
  static async cleanup(..._args: unknown[]): Promise<void> {}
  static async disable(..._args: unknown[]): Promise<void> {}
  static async reset(..._args: unknown[]): Promise<void> {}
  static async waitForInitialization(..._args: unknown[]): Promise<boolean> {
    return false;
  }
  static async isEnabled(..._args: unknown[]): Promise<boolean> {
    return false;
  }

  // ── identity / session ──────────────────────────────────────────────────
  static async identify(..._args: unknown[]): Promise<void> {}
  static getCurrentUserId(..._args: unknown[]): string | null {
    return null;
  }
  static async getPersistentUserId(..._args: unknown[]): Promise<string> {
    return "";
  }
  static async startSession(..._args: unknown[]): Promise<string | null> {
    return null;
  }
  static async endSession(..._args: unknown[]): Promise<void> {}
  static async isSessionActive(..._args: unknown[]): Promise<boolean> {
    return false;
  }

  // ── device info ─────────────────────────────────────────────────────────
  static async getPlatform(..._args: unknown[]): Promise<string> {
    return "";
  }
  static async getOSVersion(..._args: unknown[]): Promise<string> {
    return "";
  }
  static async getDeviceInfo(..._args: unknown[]): Promise<DeviceInfo> {
    return {};
  }

  // ── usage bookkeeping ───────────────────────────────────────────────────
  static async calculateDaysSince(..._args: unknown[]): Promise<number | null> {
    return null;
  }
  static async getMeetingsCountToday(..._args: unknown[]): Promise<number> {
    return 0;
  }
  static async updateMeetingCount(..._args: unknown[]): Promise<void> {}
  static async hasUsedFeatureBefore(..._args: unknown[]): Promise<boolean> {
    return false;
  }
  static async markFeatureUsed(..._args: unknown[]): Promise<void> {}
  static async checkAndTrackFirstLaunch(..._args: unknown[]): Promise<void> {}
  static async checkAndTrackDailyUsage(..._args: unknown[]): Promise<void> {}
  static async trackDailyActiveUser(..._args: unknown[]): Promise<void> {}
  static async trackUserFirstLaunch(..._args: unknown[]): Promise<void> {}

  // ── generic + named tracking (all inert) ────────────────────────────────
  static async track(..._args: unknown[]): Promise<void> {}
  static async trackEvent(..._args: unknown[]): Promise<void> {}
  static async trackButtonClick(..._args: unknown[]): Promise<void> {}
  static async trackPageView(..._args: unknown[]): Promise<void> {}
  static async trackFeatureUsed(..._args: unknown[]): Promise<void> {}
  static async trackFeatureUsedEnhanced(..._args: unknown[]): Promise<void> {}
  static async trackError(..._args: unknown[]): Promise<void> {}
  static async trackCopy(..._args: unknown[]): Promise<void> {}
  static async trackCustomPromptUsed(..._args: unknown[]): Promise<void> {}
  static async trackAppStarted(..._args: unknown[]): Promise<void> {}
  static async trackBackendConnection(..._args: unknown[]): Promise<void> {}
  static async trackModelChanged(..._args: unknown[]): Promise<void> {}
  static async trackSettingsChanged(..._args: unknown[]): Promise<void> {}
  static async trackMeetingStarted(..._args: unknown[]): Promise<void> {}
  static async trackMeetingCompleted(..._args: unknown[]): Promise<void> {}
  static async trackMeetingDeleted(..._args: unknown[]): Promise<void> {}
  static async trackRecordingStarted(..._args: unknown[]): Promise<void> {}
  static async trackRecordingStopped(..._args: unknown[]): Promise<void> {}
  static async trackTranscriptionSuccess(..._args: unknown[]): Promise<void> {}
  static async trackTranscriptionError(..._args: unknown[]): Promise<void> {}
  static async trackSummaryGenerationStarted(..._args: unknown[]): Promise<void> {}
  static async trackSummaryGenerationCompleted(..._args: unknown[]): Promise<void> {}
  static async trackSummaryRegenerated(..._args: unknown[]): Promise<void> {}
  static async trackSessionStarted(..._args: unknown[]): Promise<void> {}
  static async trackSessionEnded(..._args: unknown[]): Promise<void> {}
}

export default Analytics;
