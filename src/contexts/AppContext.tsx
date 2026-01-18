import React, { ReactNode } from 'react';
import { AuthProvider } from './UserContext.tsx';
import { UploadProvider } from './UploadContext';

/**
 * AppProvider composes all feature-based contexts.
 *
 * Context hierarchy:
 * - AuthContext: User authentication, API client
 * - UploadContext: File uploads, device info (depends on AuthContext)
 */
export const AppProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  return (
    <AuthProvider>
      <UploadProvider>
        {children}
      </UploadProvider>
    </AuthProvider>
  );
};

// Re-export hooks for convenience
export { useAuth } from './UserContext.tsx';
export { useUpload } from './UploadContext';