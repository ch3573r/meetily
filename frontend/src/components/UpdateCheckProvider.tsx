'use client'

import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { useUpdateCheck } from '@/hooks/useUpdateCheck';
import { UpdateInfo } from '@/services/updateService';
import { UpdateDialog } from './UpdateDialog';
import { setUpdateDialogCallback, showUpdateNotification } from './UpdateNotification';
import {
  AUTO_UPDATE_CHECK_CHANGED_EVENT,
  getAutoUpdateCheckEnabled,
} from '@/lib/updatePreferences';

interface UpdateCheckContextType {
  updateInfo: UpdateInfo | null;
  isChecking: boolean;
  checkForUpdates: (force?: boolean) => Promise<void>;
  showUpdateDialog: () => void;
}

const UpdateCheckContext = createContext<UpdateCheckContextType | undefined>(undefined);

export function UpdateCheckProvider({ children }: { children: React.ReactNode }) {
  const [showDialog, setShowDialog] = useState(false);
  const [autoUpdateCheckEnabled, setAutoUpdateCheckEnabled] = useState<boolean | null>(null);

  const handleShowDialog = useCallback(() => {
    setShowDialog(true);
  }, []);

  useEffect(() => {
    const syncAutoUpdatePreference = (event?: Event) => {
      if (event instanceof CustomEvent && typeof event.detail === 'boolean') {
        setAutoUpdateCheckEnabled(event.detail);
        return;
      }

      setAutoUpdateCheckEnabled(getAutoUpdateCheckEnabled());
    };

    syncAutoUpdatePreference();
    window.addEventListener('storage', syncAutoUpdatePreference);
    window.addEventListener(AUTO_UPDATE_CHECK_CHANGED_EVENT, syncAutoUpdatePreference);

    return () => {
      window.removeEventListener('storage', syncAutoUpdatePreference);
      window.removeEventListener(AUTO_UPDATE_CHECK_CHANGED_EVENT, syncAutoUpdatePreference);
    };
  }, []);

  const handleUpdateAvailable = useCallback((info: UpdateInfo) => {
    showUpdateNotification(info, handleShowDialog);
  }, [handleShowDialog]);

  const { updateInfo, isChecking, checkForUpdates } = useUpdateCheck({
    checkOnMount: autoUpdateCheckEnabled === true,
    showNotification: true,
    onUpdateAvailable: handleUpdateAvailable,
  });

  useEffect(() => {
    // Register the callback so UpdateNotification can trigger the dialog
    setUpdateDialogCallback(handleShowDialog);
    return () => {
      setUpdateDialogCallback(() => {});
    };
  }, [handleShowDialog]);

  // Listen for tray menu events
  useEffect(() => {
    const handleTrayCheck = () => {
      checkForUpdates(true); // Force check from tray
      setShowDialog(true);
    };

    window.addEventListener('check-updates-from-tray', handleTrayCheck);
    return () => window.removeEventListener('check-updates-from-tray', handleTrayCheck);
  }, [checkForUpdates]);

  return (
    <UpdateCheckContext.Provider
      value={{
        updateInfo,
        isChecking,
        checkForUpdates,
        showUpdateDialog: handleShowDialog,
      }}
    >
      {children}
      <UpdateDialog
        open={showDialog}
        onOpenChange={setShowDialog}
        updateInfo={updateInfo}
      />
    </UpdateCheckContext.Provider>
  );
}

export function useUpdateCheckContext() {
  const context = useContext(UpdateCheckContext);
  if (context === undefined) {
    throw new Error('useUpdateCheckContext must be used within UpdateCheckProvider');
  }
  return context;
}
