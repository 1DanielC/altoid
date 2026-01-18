import { useUpload } from '../contexts/AppContext';

export default function ClearCacheButton() {
  const { clearCache } = useUpload();

  return (
    <button
      className="button button-danger"
      onClick={clearCache}
    >
      Clear Cache
    </button>
  );
}
