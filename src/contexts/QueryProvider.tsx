import React, { ReactNode } from 'react';
import { QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { queryClient } from '../config/queryClient';

export const QueryProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  return (
    <QueryClientProvider client={queryClient}>
      {children}
      {/* Only in development */}
      {import.meta.env.DEV && <ReactQueryDevtools initialIsOpen={false} />}
    </QueryClientProvider>
  );
};
