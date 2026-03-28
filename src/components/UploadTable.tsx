import './UploadTable.css';

export type UploadStatus = 'waiting' | 'new' | 'in_progress' | 'uploaded';

export interface UploadEntry {
  filename: string;
  uploadId: string | null;
  status: UploadStatus;
  progress: number; // 0-100
}

interface UploadTableProps {
  uploads: UploadEntry[];
}

function statusLabel(status: UploadStatus): string {
  switch (status) {
    case 'waiting': return 'Waiting';
    case 'new': return 'New';
    case 'in_progress': return 'In Progress';
    case 'uploaded': return 'Uploaded';
  }
}

function statusClass(status: UploadStatus): string {
  switch (status) {
    case 'waiting': return 'status-waiting';
    case 'new': return 'status-new';
    case 'in_progress': return 'status-progress';
    case 'uploaded': return 'status-uploaded';
  }
}

export default function UploadTable({ uploads }: UploadTableProps) {
  if (uploads.length === 0) return null;

  return (
    <div className="upload-table-container">
      <table className="upload-table">
        <thead>
          <tr>
            <th>File</th>
            <th>Status</th>
            <th>Progress</th>
          </tr>
        </thead>
        <tbody>
          {uploads.map((entry) => (
            <tr key={entry.filename}>
              <td className="upload-filename" title={entry.filename}>
                {entry.filename}
              </td>
              <td>
                <span className={`upload-status ${statusClass(entry.status)}`}>
                  {statusLabel(entry.status)}
                </span>
              </td>
              <td>
                {entry.status === 'in_progress' ? (
                  <div className="progress-bar">
                    <div
                      className="progress-fill"
                      style={{ width: `${entry.progress}%` }}
                    />
                  </div>
                ) : entry.status === 'uploaded' ? (
                  <div className="progress-bar">
                    <div className="progress-fill progress-complete" style={{ width: '100%' }} />
                  </div>
                ) : null}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
