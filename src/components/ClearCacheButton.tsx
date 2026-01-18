import { useDeleteDataMutation } from '../contexts/AppContext';

export default function ClearCacheButton() {
  const deleteDataMutation = useDeleteDataMutation();

  return (
      <button
          className="button button-danger"
          disabled={deleteDataMutation.isPending}
          onClick={() => deleteDataMutation.mutate()}
      >
        {deleteDataMutation.isPending ? 'Clearing...' : 'Clear Cache'}
      </button>
  );
}
