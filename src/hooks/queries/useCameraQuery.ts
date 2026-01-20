import {useQuery} from "@tanstack/react-query";
import {getCamera} from "../../contexts/services/CameraService.ts";

export const CAMERA_QUERY_KEY = ['camera'] as const;

export function useCameraQuery() {
  return useQuery<string | null, Error>({
    queryKey: CAMERA_QUERY_KEY,
    queryFn: async () => {
      try {
        return await getCamera();
      } catch (error) {
        return null;
      }
    },
    staleTime: 5 * 60 * 1000,
  });
}
