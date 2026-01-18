import React, { createContext, useContext, useState, ReactNode, useEffect } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { UploadStatus, UploadProgress } from '../types/upload';

interface UploadProgressContextType extends UploadProgress {
  deviceId: string;
  resetProgress: () => void;
}

const UploadProgressContext = createContext<UploadProgressContextType | undefined>(undefined);

export const UploadProgressProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [deviceId] = useState('No camera connected');
  const [uploads, setUploads] = useState<Record<string, UploadStatus>>({});
  const [skippedCount, setSkippedCount] = useState(0);

  useEffect(() => {
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenComplete: UnlistenFn | undefined;
    let unlistenSkipped: UnlistenFn | undefined;
    let unlistenError: UnlistenFn | undefined;

    // Set up Tauri event listeners for real-time upload progress
    const setupListeners = async () => {
      // Listen for upload progress events
      unlistenProgress = await listen<UploadStatus>('upload-progress', (event) => {
        const status = event.payload;
        setUploads((prev) => ({
          ...prev,
          [status.filename]: status,
        }));
      });

      // Listen for upload completion
      unlistenComplete = await listen<{ filename: string }>('upload-complete', (event) => {
        const { filename } = event.payload;
        setUploads((prev) => ({
          ...prev,
          [filename]: { ...prev[filename], status: 'completed', percentage: 100 },
        }));
      });

      // Listen for skipped files
      unlistenSkipped = await listen('upload-skipped', () => {
        setSkippedCount((prev) => prev + 1);
      });

      // Listen for upload errors
      unlistenError = await listen<{ filename: string; error: string }>('upload-error', (event) => {
        const { filename, error } = event.payload;
        setUploads((prev) => ({
          ...prev,
          [filename]: { ...prev[filename], status: 'failed', error },
        }));
      });
    };

    setupListeners();

    // Cleanup listeners on unmount
    return () => {
      unlistenProgress?.();
      unlistenComplete?.();
      unlistenSkipped?.();
      unlistenError?.();
    };
  }, []);

  const resetProgress = () => {
    setUploads({});
    setSkippedCount(0);
  };

  return (
    <UploadProgressContext.Provider
      value={{
        deviceId,
        uploads,
        skippedCount,
        resetProgress,
      }}
    >
      {children}
    </UploadProgressContext.Provider>
  );
};

export const useUploadProgress = () => {
  const context = useContext(UploadProgressContext);
  if (!context) {
    throw new Error('useUploadProgress must be used within UploadProgressProvider');
  }
  return context;
};
