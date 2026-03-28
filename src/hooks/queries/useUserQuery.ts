import {useQuery} from '@tanstack/react-query';
import {UserInfo} from '../../rust-api/model/AuthResult';
import {getUser} from "../../contexts/services/ApiService.ts";

export const USER_QUERY_KEY = ['user'] as const;

export function useUserQuery() {
  return useQuery<UserInfo | null, Error>({
    queryKey: USER_QUERY_KEY,
    queryFn: async () => {
      try {
        return getUser();
      } catch (error) {
        // If user not authenticated, return null instead of throwing
        console.log('User not authenticated:', error);
        return null;
      }
    },
    staleTime: 5 * 60 * 1000,
    // Do not automatically fetch on mount - user must sign in explicitly
    enabled: false,
  });
}
