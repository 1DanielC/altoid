import { useUploadMutation, useUploadProgress } from '../contexts/AppContext';

export default function UploadButton() {
  const uploadMutation = useUploadMutation();
  const { resetProgress } = useUploadProgress();

  const handleUpload = () => {
    // Reset progress tracking before starting new upload
    resetProgress();
    // Trigger upload mutation
    uploadMutation.mutate();
  };

  return (
    <button
      className="button"
      disabled={uploadMutation.isPending}
      onClick={handleUpload}
    >
      {uploadMutation.isPending ? 'Uploading...' : 'Upload Files'}
    </button>
  );
}
