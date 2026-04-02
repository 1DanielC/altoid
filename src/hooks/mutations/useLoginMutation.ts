import { useMutation, useQueryClient } from '@tanstack/react-query';
import { getUser, logout } from '../../contexts/services/ApiService';
import { UserInfo } from '../../rust-api/model/AuthResult';
import { USER_QUERY_KEY } from '../queries/useUserQuery';
import { NotificationType } from '../../contexts/NotificationContext';
import { parseIpcError } from '../../services/ipcError';

interface LoginParams {
  clearAuth?: boolean;
}

export function useLoginMutation(notify?: (type: NotificationType, message: string) => void) {
  const queryClient = useQueryClient();

  return useMutation<UserInfo, Error, LoginParams>({
    mutationFn: async ({ clearAuth = false }: LoginParams) => {
      if (clearAuth) {
        await logout();
      }
      return await getUser();
    },
    onSuccess: (userData) => {
      queryClient.setQueryData(USER_QUERY_KEY, userData);
      notify?.('success', `Signed in as ${userData.email}`);
    },
    onError: (error) => {
      console.error('Login failed:', error);
      const parsed = parseIpcError(error);
      notify?.(parsed.type, parsed.message);
    },
  });
}
