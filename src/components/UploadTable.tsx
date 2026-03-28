import './UploadTable.css';

export type UploadStatus = 'waiting' | 'downloading' | 'in_progress' | 'uploaded' | 'error';

export interface UploadEntry {
  filename: string;
  uploadId: string | null;
  status: UploadStatus;
  bytes: number;
  totalBytes: number;
  error?: string;
}

interface UploadTableProps {
  uploads: UploadEntry[];
}

function statusLabel(status: UploadStatus): string {
  switch (status) {
    case 'waiting': return 'Waiting';
    case 'downloading': return 'Downloading';
    case 'in_progress': return 'Uploading';
    case 'uploaded': return 'Uploaded';
    case 'error': return 'Error';
  }
}

function statusTooltip(status: UploadStatus): string {
  switch (status) {
    case 'waiting': return 'Queued — waiting for other files to finish';
    case 'downloading': return 'Copying file from camera to local disk';
    case 'in_progress': return 'Streaming file to the server';
    case 'uploaded': return 'File has been uploaded successfully';
    case 'error': return 'An error occurred during upload';
  }
}

function statusClass(status: UploadStatus): string {
  switch (status) {
    case 'waiting': return 'status-waiting';
    case 'downloading': return 'status-downloading';
    case 'in_progress': return 'status-progress';
    case 'uploaded': return 'status-uploaded';
    case 'error': return 'status-error';
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

function progressPercent(entry: UploadEntry): number {
  if (entry.totalBytes <= 0) return 0;
  return Math.min(100, (entry.bytes / entry.totalBytes) * 100);
}

export default function UploadTable({ uploads }: UploadTableProps) {
  if (uploads.length === 0) return null;

  return (
    <div className="upload-table-container">
      <table className="upload-table">
        <thead>
          <tr>
            <th style={{ width: '30%' }}>File</th>
            <th style={{ width: '28%' }}>Status</th>
            <th style={{ width: '42%' }}>Progress</th>
          </tr>
        </thead>
        <tbody>
          {uploads.map((entry) => (
            <tr key={entry.filename} className={entry.status === 'error' ? 'row-error' : ''}>
              <td className="upload-filename" title={entry.filename}>
                {entry.filename}
              </td>
              <td>
                <span className={`upload-status ${statusClass(entry.status)}`} title={statusTooltip(entry.status)}>
                  {statusLabel(entry.status)}
                </span>
              </td>
              <td className="progress-cell">
                {(entry.status === 'downloading' || entry.status === 'in_progress') ? (
                  <>
                    {entry.bytes === 0 && entry.status === 'downloading' ? (
                      <div className="progress-bar">
                        <div className="progress-fill progress-downloading progress-indeterminate" />
                      </div>
                    ) : (
                      <div className="progress-bar">
                        <div
                          className={`progress-fill progress-${entry.status}`}
                          style={{ width: `${progressPercent(entry)}%` }}
                        />
                      </div>
                    )}
                    <span className="progress-text">
                      {entry.bytes > 0 ? formatBytes(entry.bytes) : `Downloading ${formatBytes(entry.totalBytes)}...`}{entry.bytes > 0 && entry.totalBytes > 0 ? ` / ${formatBytes(entry.totalBytes)}` : ''}
                    </span>
                  </>
                ) : entry.status === 'uploaded' ? (
                  <div className="progress-bar">
                    <div className="progress-fill progress-uploaded" style={{ width: '100%' }} />
                  </div>
                ) : entry.status === 'error' ? (
                  <span className="error-text">See Activity Log for details</span>
                ) : null}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
