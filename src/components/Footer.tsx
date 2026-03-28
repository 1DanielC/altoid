import { useCameraQuery } from '../contexts/AppContext';

export default function Footer() {
  const { data: camera } = useCameraQuery();

  const label = camera?.camera?.device_id
    ? `Camera: ${camera.camera.device_id}`
    : 'No camera connected';

  return (
    <div id="footer">
      <div id="footer-bar">
        <p>{label}</p>
      </div>
    </div>
  );
}
