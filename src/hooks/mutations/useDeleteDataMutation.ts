import { useMutation, useQueryClient } from '@tanstack/react-query';
import { deleteData } from '../../contexts/services/SystemService';
import { USER_QUERY_KEY } from '../queries/useUserQuery';

export function useDeleteDataMutation() {
  const queryClient = useQueryClient();

  return useMutation<void, Error, void>({
    mutationFn: async () => {
      await deleteData();
    },
    onSuccess: () => {
      // Clear all caches when data is deleted
      queryClient.setQueryData(USER_QUERY_KEY, null);
      // Could also invalidate or clear other queries if needed
      queryClient.invalidateQueries();
    },
  });
}
