import { useMutation, useQueryClient } from '@tanstack/react-query';
import { getUser, logout } from '../../contexts/services/ApiService';
import { UserInfo } from '../../rust-api/model/AuthResult';
import { USER_QUERY_KEY } from '../queries/useUserQuery';

interface LoginParams {
  clearAuth?: boolean;
}

export function useLoginMutation() {
  const queryClient = useQueryClient();

  return useMutation<UserInfo, Error, LoginParams>({
    mutationFn: async ({ clearAuth = false }: LoginParams) => {
      if (clearAuth) {
        await logout();
      }
      return await getUser();
    },
    onSuccess: (userData) => {
      // Update the user query cache with the new data
      queryClient.setQueryData(USER_QUERY_KEY, userData);
    },
    onError: (error) => {
      console.error('Login failed:', error);
    },
  });
}
