import { useQuery } from '@tanstack/react-query';
import { getUser } from '../../contexts/services/ApiService';
import { UserInfo } from '../../rust-api/model/AuthResult';

export const USER_QUERY_KEY = ['user'] as const;

export function useUserQuery() {
  return useQuery<UserInfo | null, Error>({
    queryKey: USER_QUERY_KEY,
    queryFn: async () => {
      try {
        return await getUser();
      } catch (error) {
        // If user not authenticated, return null instead of throwing
        console.log('User not authenticated:', error);
        return null;
      }
    },
    // Automatically fetch on mount (replaces UserContext useEffect)
    staleTime: 5 * 60 * 1000,
  });
}
