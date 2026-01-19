import React, { ReactNode } from 'react';
import { QueryProvider } from './QueryProvider';
import { UploadProgressProvider } from './UploadProgressContext';

/**
 * AppProvider composes React Query and upload progress tracking.
 *
 * Context hierarchy:
 * - QueryProvider: React Query client for all server state
 * - UploadProgressProvider: Real-time upload progress from Tauri events
 */
export const AppProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  return (
    <QueryProvider>
      <UploadProgressProvider>
        {children}
      </UploadProgressProvider>
    </QueryProvider>
  );
};

// Re-export hooks for convenience
export {
  useUserQuery,
  useCameraQuery,
  useLoginMutation,
  useLogoutMutation,
  useDeleteDataMutation,
  useUploadMutation,
} from '../hooks';

export { useUploadProgress } from './UploadProgressContext';