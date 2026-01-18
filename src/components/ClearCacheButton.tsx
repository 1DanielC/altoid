import {useUser} from '../contexts/AppContext';

export default function ClearCacheButton() {
  const {deleteAllData} = useUser();

  return (
      <button
          className="button button-danger"
          onClick={deleteAllData}
      >
        Clear Cache
      </button>
  );
}
