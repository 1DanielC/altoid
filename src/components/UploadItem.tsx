interface UploadStatus {
  filename: string;
  bytesUploaded: number;
  totalBytes: number;
  percentage: number;
  status: 'pending' | 'uploading' | 'completed' | 'skipped' | 'failed';
  error?: string;
}

interface Props {
  upload: UploadStatus;
}

export default function UploadItem({ upload }: Props) {
  const statusClass = `status-${upload.status}`;
  const displayStatus = upload.status === 'failed' && upload.error
    ? `failed: ${upload.error}`
    : upload.status;

  return (
    <div className="upload-item">
      <p className="upload-filename">{upload.filename}</p>
      <p className={`upload-status ${statusClass}`}>
        Status: {displayStatus}
      </p>
      {upload.status === 'uploading' && (
        <>
          <p className="upload-progress-text">
            {upload.bytesUploaded} / {upload.totalBytes} bytes ({upload.percentage.toFixed(1)}%)
          </p>
          <div className="progress-bar-container">
            <div
              className="progress-bar-fill"
              style={{ width: `${upload.percentage}%` }}
            />
          </div>
        </>
      )}
    </div>
  );
}
