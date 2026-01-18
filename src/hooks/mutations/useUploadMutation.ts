import { useMutation } from '@tanstack/react-query';
import { uploadAllFiles } from '../../contexts/services/CameraService';

export function useUploadMutation() {
  return useMutation<void, Error, void>({
    mutationFn: async () => {
      await uploadAllFiles();
    },
    onError: (error) => {
      console.error('Upload failed:', error);
    },
  });
}
