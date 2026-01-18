import { useMutation, useQueryClient } from '@tanstack/react-query';
import { logout } from '../../contexts/services/ApiService';
import { USER_QUERY_KEY } from '../queries/useUserQuery';

export function useLogoutMutation() {
  const queryClient = useQueryClient();

  return useMutation<void, Error, void>({
    mutationFn: async () => {
      await logout();
    },
    onSuccess: () => {
      // Clear user data from cache
      queryClient.setQueryData(USER_QUERY_KEY, null);
    },
  });
}
