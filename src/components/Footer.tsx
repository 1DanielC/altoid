import { useUploadProgress } from '../contexts/AppContext';

export default function Footer() {
  const { deviceId, skippedCount } = useUploadProgress();

  return (
    <div id="footer">
      <div id="footer-bar">
        <p>{deviceId}</p>
      </div>
      <div id="footer-bar">
        <p>Le App is Updated</p>
      </div>
      {skippedCount > 0 && (
        <div id="footer-bar">
          <p className="skipped-count">Total skipped files: {skippedCount}</p>
        </div>
      )}
    </div>
  );
}
