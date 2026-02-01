import { useUploadProgress } from '../contexts/AppContext';

export default function Footer() {
  const { deviceId } = useUploadProgress();

  return (
    <div id="footer">
      <div id="footer-bar">
        <p>{deviceId}</p>
      </div>
    </div>
  );
}
