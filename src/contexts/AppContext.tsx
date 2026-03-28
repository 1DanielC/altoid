import React, { ReactNode } from 'react';
import { QueryProvider } from './QueryProvider';

export const AppProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  return (
    <QueryProvider>
      {children}
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
} from '../hooks';