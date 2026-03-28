import {invoke} from "@tauri-apps/api/core";

export interface CameraFile {
  path: string;
  filename: string;
  size: number;
  content_type: string;
}

export interface CameraResult {
  message?: string;
  device_id?: string;
  camera?: {
    info: { vendor: string; vendor_id: number; device: string };
    serial_number: string | null;
    device_id: string;
  };
  mount_point?: string | null;
  files?: CameraFile[];
  access_error?: string | null;
}

export async function getCamera(): Promise<CameraResult | null> {
  return await Promise.race([
    invoke<CameraResult | null>("get_camera"),
    new Promise<CameraResult | null>((_, reject) =>
      setTimeout(() => reject(new Error("Camera detection timed out")), 60000)
    ),
  ]);
}

export interface CreateUploadsResult {
  total: number;
  results: Array<{
    filename: string;
    response?: { uploadId: string | null; status?: string };
    error?: string;
  }>;
}

export async function createUploads(deviceId: string, files: CameraFile[]): Promise<CreateUploadsResult> {
  return await invoke<CreateUploadsResult>("create_uploads", { deviceId, files });
}
export interface UploadFileResult {
  filename: string;
  status: string;
  uploadId?: string | null;
}

export async function uploadFile(
  deviceId: string,
  filePath: string,
  filename: string,
  mountPoint: string,
  contentType: string,
): Promise<UploadFileResult> {
  return await invoke<UploadFileResult>("upload_file", { deviceId, filePath, filename, mountPoint, contentType });
}