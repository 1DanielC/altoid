import { useUploadProgress } from '../contexts/AppContext';
import UploadItem from './UploadItem';

export default function UploadList() {
  const { uploads } = useUploadProgress();
  const uploadArray = Object.values(uploads);

  if (uploadArray.length === 0) {
    return null;
  }

  return (
    <div className="upload-list-container">
      {uploadArray.map(upload => (
        <UploadItem key={upload.filename} upload={upload} />
      ))}
    </div>
  );
}
