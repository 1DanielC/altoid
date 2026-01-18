import { useUpload } from '../contexts/AppContext';

export default function UploadButton() {
  const { isUploading, startUpload } = useUpload();

  return (
    <button
      className="button"
      disabled={isUploading}
      onClick={startUpload}
    >
      {isUploading ? 'Uploading...' : 'Upload Files'}
    </button>
  );
}
