import React, { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { motion, AnimatePresence } from 'framer-motion';
import { toast } from 'sonner';
import { Loader2, CheckCircle2, Download, X } from 'lucide-react';

type ModelStatus =
  | 'Available'
  | 'Missing'
  | { Downloading: { progress: number } }
  | { Error: string };

interface RawModelInfo {
  name: string;
  size_mb: number;
  status: ModelStatus;
  description?: string;
}

interface NemotronModelManagerProps {
  selectedModel?: string;
  onModelSelect?: (modelName: string) => void;
  autoSave?: boolean;
  className?: string;
}

/**
 * Settings model manager for the Nemotron streaming engine (Beta). One model for now (fp16, ~1.3 GB); downloads/selects mirror the Parakeet manager but the
 * UI is token-colored for dark mode and self-contained (no lib/nemotron layer).
 */
export function NemotronModelManager({
  selectedModel,
  onModelSelect,
  autoSave = false,
  className = '',
}: NemotronModelManagerProps) {
  const [model, setModel] = useState<RawModelInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const onSelectRef = useRef(onModelSelect);
  const autoSaveRef = useRef(autoSave);
  useEffect(() => {
    onSelectRef.current = onModelSelect;
    autoSaveRef.current = autoSave;
  }, [onModelSelect, autoSave]);

  const saveSelection = async (name: string) => {
    try {
      await invoke('api_save_transcript_config', { provider: 'nemotron', model: name, apiKey: null });
    } catch (e) {
      console.error('Failed to save Nemotron selection:', e);
    }
  };

  useEffect(() => {
    let active = true;
    (async () => {
      try {
        setLoading(true);
        await invoke('nemotron_init');
        const list = await invoke<RawModelInfo[]>('nemotron_get_available_models');
        if (active) setModel(list[0] ?? null);
      } catch (e) {
        if (active) setError(e instanceof Error ? e.message : 'Failed to load model');
      } finally {
        if (active) setLoading(false);
      }
    })();
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    (async () => {
      unlisteners.push(
        await listen<{ modelName: string; progress: number }>(
          'nemotron-model-download-progress',
          (e) =>
            setModel((m) =>
              m && m.name === e.payload.modelName
                ? { ...m, status: { Downloading: { progress: e.payload.progress } } }
                : m,
            ),
        ),
      );
      unlisteners.push(
        await listen<{ modelName: string }>('nemotron-model-download-complete', (e) => {
          setModel((m) => (m && m.name === e.payload.modelName ? { ...m, status: 'Available' } : m));
          toast.success('🌊 Nemotron ready!', { description: 'Model downloaded and ready to use' });
          if (onSelectRef.current) {
            onSelectRef.current(e.payload.modelName);
            if (autoSaveRef.current) saveSelection(e.payload.modelName);
          }
        }),
      );
      unlisteners.push(
        await listen<{ modelName: string; error: string }>('nemotron-model-download-error', (e) => {
          setModel((m) =>
            m && m.name === e.payload.modelName ? { ...m, status: { Error: e.payload.error } } : m,
          );
          toast.error('Failed to download Nemotron', { description: e.payload.error });
        }),
      );
    })();
    return () => unlisteners.forEach((u) => u());
  }, []);

  const download = async () => {
    if (!model) return;
    setModel({ ...model, status: { Downloading: { progress: 0 } } });
    toast.info('Downloading Nemotron…', { description: 'About 1.3 GB — this may take a few minutes' });
    try {
      await invoke('nemotron_download_model', { modelName: model.name });
    } catch (e) {
      setModel((m) => (m ? { ...m, status: { Error: e instanceof Error ? e.message : 'Download failed' } } : m));
    }
  };

  const cancel = async () => {
    if (!model) return;
    try {
      await invoke('nemotron_cancel_download', { modelName: model.name });
      setModel({ ...model, status: 'Missing' });
      toast.info('Download cancelled');
    } catch (e) {
      console.error('Cancel failed:', e);
    }
  };

  const select = async () => {
    if (!model) return;
    onModelSelect?.(model.name);
    if (autoSave) await saveSelection(model.name);
    toast.success('Switched to Nemotron');
  };

  if (loading) {
    return (
      <div className={`space-y-3 ${className}`}>
        <div className="h-24 animate-pulse rounded-lg bg-muted" />
      </div>
    );
  }
  if (error || !model) {
    return (
      <div className={`rounded-lg border border-destructive/30 bg-destructive/10 p-4 ${className}`}>
        <p className="text-sm text-destructive">Failed to load Nemotron model</p>
        {error && <p className="mt-1 text-xs text-destructive/80">{error}</p>}
      </div>
    );
  }

  const isAvailable = model.status === 'Available';
  const isMissing = model.status === 'Missing';
  const isError = typeof model.status === 'object' && 'Error' in model.status;
  const progress =
    typeof model.status === 'object' && 'Downloading' in model.status
      ? model.status.Downloading.progress
      : null;
  const isSelected = selectedModel === model.name;

  return (
    <div className={`space-y-3 ${className}`}>
      <div
        className={`relative rounded-lg border-2 p-4 transition-all ${
          isSelected && isAvailable ? 'border-primary bg-primary/5' : 'border-border bg-card'
        } ${isAvailable ? 'cursor-pointer' : ''}`}
        onClick={() => isAvailable && select()}
      >
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1">
            <div className="mb-1 flex items-center gap-2">
              <span className="text-2xl">🌊</span>
              <h3 className="font-semibold text-foreground">Nemotron 3.5 ASR</h3>
              <span className="rounded-full bg-yellow-100 px-2 py-0.5 text-xs font-medium text-yellow-800">
                BETA
              </span>
              {isSelected && isAvailable && (
                <span className="flex items-center gap-1 rounded-full bg-primary px-2 py-0.5 text-xs font-medium text-primary-foreground">
                  <CheckCircle2 className="h-3 w-3" />
                </span>
              )}
            </div>
            <p className="ml-9 text-sm text-muted-foreground">
              Streaming, multilingual (incl. German). FP16 — tries GPU (DirectML) with CPU fallback. ~1.3 GB.
            </p>
          </div>

          <div className="flex items-center gap-2">
            {isAvailable && (
              <div className="flex items-center gap-1.5 text-emerald-500">
                <div className="h-2 w-2 rounded-full bg-emerald-500" />
                <span className="text-xs font-medium">Ready</span>
              </div>
            )}
            {isMissing && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  download();
                }}
                className="flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
              >
                <Download className="h-4 w-4" />
                Download
              </button>
            )}
            {progress === null && isError && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  download();
                }}
                className="rounded-md bg-destructive px-3 py-1.5 text-sm font-medium text-destructive-foreground transition-colors hover:bg-destructive/90"
              >
                Retry
              </button>
            )}
          </div>
        </div>

        <AnimatePresence>
          {progress !== null && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              className="mt-3 border-t border-border pt-3"
            >
              <div className="mb-2 flex items-center justify-between">
                <div className="flex items-center gap-2 text-primary">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  <span className="text-sm font-medium">Downloading…</span>
                  <span className="text-sm font-semibold">{Math.round(progress)}%</span>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    cancel();
                  }}
                  className="flex items-center gap-1 rounded px-2 py-1 text-xs font-medium text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
                >
                  <X className="h-3 w-3" />
                  Cancel
                </button>
              </div>
              <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                <motion.div
                  className="h-full rounded-full bg-primary"
                  initial={{ width: 0 }}
                  animate={{ width: `${progress}%` }}
                  transition={{ duration: 0.3, ease: 'easeOut' }}
                />
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {isSelected && (
        <p className="pt-1 text-center text-xs text-muted-foreground">
          Using Nemotron for transcription
        </p>
      )}
    </div>
  );
}
