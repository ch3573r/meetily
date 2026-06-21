"use client";

import Image from "next/image";
import { Minus, Square, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

const getAppWindow = () => {
  if (typeof window === "undefined") return null;
  return getCurrentWindow();
};

export function AppTitlebar() {
  const handleDrag = async (event: React.MouseEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;

    try {
      const appWindow = getAppWindow();
      if (!appWindow) return;

      if (event.detail === 2) {
        await appWindow.toggleMaximize();
      } else {
        await appWindow.startDragging();
      }
    } catch {
      // Browser previews do not have Tauri's native window API.
    }
  };

  const handleMinimize = async () => {
    try {
      const appWindow = getAppWindow();
      if (!appWindow) return;
      await appWindow.minimize();
    } catch {
      // Browser previews do not have Tauri's native window API.
    }
  };

  const handleMaximize = async () => {
    try {
      const appWindow = getAppWindow();
      if (!appWindow) return;
      await appWindow.toggleMaximize();
    } catch {
      // Browser previews do not have Tauri's native window API.
    }
  };

  const handleClose = async () => {
    try {
      const appWindow = getAppWindow();
      if (!appWindow) return;
      await appWindow.close();
    } catch {
      // Browser previews do not have Tauri's native window API.
    }
  };

  return (
    <header
      className="fixed inset-x-0 top-0 z-[100] flex h-[var(--titlebar-height)] select-none items-center border-b border-border bg-background/95 text-foreground backdrop-blur"
      onMouseDown={handleDrag}
      data-tauri-drag-region
    >
      <div className="flex min-w-0 flex-1 items-center gap-2 px-2" data-tauri-drag-region>
        <Image
          src="/brand/clawscribe-icon-64.png"
          alt=""
          width={18}
          height={18}
          className="pointer-events-none rounded-[3px]"
          priority
        />
        <span className="truncate text-xs font-medium text-muted-foreground" data-tauri-drag-region>
          ClawScribe
        </span>
      </div>

      <div className="no-drag flex h-full items-center" onMouseDown={(event) => event.stopPropagation()}>
        <button
          type="button"
          className="flex h-full w-11 items-center justify-center text-muted-foreground transition hover:bg-muted hover:text-foreground"
          onClick={handleMinimize}
          aria-label="Minimize"
          title="Minimize"
        >
          <Minus className="h-4 w-4" />
        </button>
        <button
          type="button"
          className="flex h-full w-11 items-center justify-center text-muted-foreground transition hover:bg-muted hover:text-foreground"
          onClick={handleMaximize}
          aria-label="Maximize or restore"
          title="Maximize or restore"
        >
          <Square className="h-3.5 w-3.5" />
        </button>
        <button
          type="button"
          className="flex h-full w-12 items-center justify-center text-muted-foreground transition hover:bg-destructive hover:text-destructive-foreground"
          onClick={handleClose}
          aria-label="Close"
          title="Close"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
    </header>
  );
}
