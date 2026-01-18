export interface UploadStatus {
  filename: string;
  bytesUploaded: number;
  totalBytes: number;
  percentage: number;
  status: 'pending' | 'uploading' | 'completed' | 'skipped' | 'failed';
  error?: string;
}

export interface UploadProgress {
  uploads: Record<string, UploadStatus>;
  skippedCount: number;
}
