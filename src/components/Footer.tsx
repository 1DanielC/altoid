import { useUploadProgress } from '../contexts/AppContext';
import Dialog from './Dialog';

export default function Footer() {
  const { deviceId } = useUploadProgress();

  return (
    <div id="footer">
      <div id="footer-bar">
        <p>{deviceId}</p>
      </div>
      <Dialog />
    </div>
  );
}
