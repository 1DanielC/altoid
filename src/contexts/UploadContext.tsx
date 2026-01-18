import React, { createContext, useContext, useState, ReactNode } from 'react';

// ============================================================================
// TYPES
// ============================================================================
interface UploadStatus {
  filename: string;
  bytesUploaded: number;
  totalBytes: number;
  percentage: number;
  status: 'pending' | 'uploading' | 'completed' | 'skipped' | 'failed';
  error?: string;
}

interface UploadContextType {
  deviceId: string;
  uploads: Record<string, UploadStatus>;
  skippedCount: number;
  isUploading: boolean;
  startUpload: () => Promise<void>;
}

const UploadContext = createContext<UploadContextType | undefined>(undefined);

// ============================================================================
// PROVIDER
// ============================================================================
export const UploadProvider: React.FC<{ children: ReactNode }> = ({ children }) => {

  const [deviceId] = useState('No camera connected');
  const [uploads, setUploads] = useState<Record<string, UploadStatus>>({});
  const [skippedCount, setSkippedCount] = useState(0);
  const [isUploading, setIsUploading] = useState(false);

  const startUpload = async () => {
    setIsUploading(true);
    setUploads({});
    setSkippedCount(0);

    try {
      const { uploadAllFiles } = await import('./services/CameraService.ts');
      // TODO upload files from camera
      await uploadAllFiles();
    } catch (error) {
      console.error('Upload failed:', error);
    } finally {
      setTimeout(() => setIsUploading(false), 1000);
    }
  };

  return (
    <UploadContext.Provider
      value={{
        deviceId,
        uploads,
        skippedCount,
        isUploading,
        startUpload,
      }}
    >
      {children}
    </UploadContext.Provider>
  );
};

// ============================================================================
// HOOK
// ============================================================================
export const useUpload = () => {
  const context = useContext(UploadContext);
  if (!context) {
    throw new Error('useUpload must be used within UploadProvider');
  }
  return context;
};
