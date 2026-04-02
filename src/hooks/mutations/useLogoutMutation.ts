import { useMutation, useQueryClient } from '@tanstack/react-query';
import { logout } from '../../contexts/services/ApiService';
import { USER_QUERY_KEY } from '../queries/useUserQuery';
import { NotificationType } from '../../contexts/NotificationContext';
import { parseIpcError } from '../../services/ipcError';

export function useLogoutMutation(notify?: (type: NotificationType, message: string) => void) {
  const queryClient = useQueryClient();

  return useMutation<void, Error, void>({
    mutationFn: async () => {
      await logout();
    },
    onSuccess: () => {
      queryClient.setQueryData(USER_QUERY_KEY, null);
      notify?.('info', 'Signed out successfully');
    },
    onError: (error) => {
      const parsed = parseIpcError(error);
      notify?.(parsed.type, parsed.message);
    },
  });
}
