'use client';

import React, { ReactNode, createContext } from 'react';

// Telemetry has been removed from ClawScribe. This provider is now a
// pass-through that keeps the context shape for any remaining consumers, but it
// never initializes analytics and is always reported as opted-out.

interface AnalyticsProviderProps {
  children: ReactNode;
}

interface AnalyticsContextType {
  isAnalyticsOptedIn: boolean;
  setIsAnalyticsOptedIn: (optedIn: boolean) => void;
}

export const AnalyticsContext = createContext<AnalyticsContextType>({
  isAnalyticsOptedIn: false,
  setIsAnalyticsOptedIn: () => { },
});

export default function AnalyticsProvider({ children }: AnalyticsProviderProps) {
  return (
    <AnalyticsContext.Provider
      value={{ isAnalyticsOptedIn: false, setIsAnalyticsOptedIn: () => { } }}
    >
      {children}
    </AnalyticsContext.Provider>
  );
}
