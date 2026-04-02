import React, { ReactNode } from 'react';
import { QueryProvider } from './QueryProvider';
import { NotificationProvider } from './NotificationContext';

export const AppProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  return (
    <QueryProvider>
      <NotificationProvider>
        {children}
      </NotificationProvider>
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

export { useNotification } from './NotificationContext';
