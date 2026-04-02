import { useState, useCallback, useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCamera, uploadFile, CameraResult } from "../contexts/services/CameraService.ts";
import { request } from "../contexts/services/ApiService.ts";
import { logError } from "../services/log.ts";
import { useNotification } from "../contexts/AppContext.tsx";
import UploadTable, { UploadEntry } from "./UploadTable.tsx";

interface FileProgress {
  filename: string;
  stage: "downloading" | "uploading";
  bytes: number;
  total: number;
}

export default function TestButton() {
  const [loading, setLoading] = useState<string | null>(null);
  const [cameraData, setCameraData] = useState<CameraResult | null>(null);
  const [uploads, setUploads] = useState<UploadEntry[]>([]);
  const { notify } = useNotification();

  const run = async (label: string, fn: () => Promise<unknown>) => {
    setLoading(label);
    try {
      const r = await fn();
      if (label === "camera") setCameraData(r as CameraResult | null);
      notify('success', `${label} completed successfully`);
    } catch (e: unknown) {
      const msg = typeof e === 'string' ? e
        : e instanceof Error ? e.message
        : JSON.stringify(e, null, 2);
      notify('error', `${label} failed: ${msg}`);
    } finally {
      setLoading(null);
    }
  };

  const updateUpload = useCallback((filename: string, update: Partial<UploadEntry>) => {
    setUploads(prev => prev.map(u =>
      u.filename === filename ? { ...u, ...update } : u
    ));
  }, []);

  // Listen for file progress events from Rust
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    listen<FileProgress>("file-progress", (event) => {
      const { filename, stage, bytes, total } = event.payload;
      updateUpload(filename, {
        status: stage === "downloading" ? "downloading" : "in_progress",
        bytes,
        ...(total > 0 ? { totalBytes: total } : {}),
      });
    }).then(fn => { unlisten = fn; });

    return () => { unlisten?.(); };
  }, [updateUpload]);

  const handleUploadFiles = async () => {
    setLoading("uploading");

    const mountPoint = cameraData?.mount_point ?? "";
    const deviceId = cameraData!.camera!.device_id;
    const files = (cameraData?.files ?? []).filter(f => {
      const ext = f.filename.split('.').pop()?.toLowerCase();
      return ext === 'insv' || ext === 'mp4';
    });

    // Populate the table with all files in "waiting" state
    setUploads(files.map(f => ({
      filename: f.filename,
      uploadId: null,
      status: 'waiting' as const,
      bytes: 0,
      totalBytes: f.size,
    })));

    // Process one at a time, top to bottom
    for (const file of files) {
      updateUpload(file.filename, { status: 'downloading', bytes: 0 });

      try {
        const r = await uploadFile(deviceId, file.path, file.filename, mountPoint, file.content_type);

        updateUpload(file.filename, {
          uploadId: r.uploadId ?? null,
          status: r.status === 'Completed' || r.status === 'Uploaded' ? 'uploaded' : 'waiting',
          bytes: file.size,
          totalBytes: file.size,
        });
      } catch (e: unknown) {
        const msg = typeof e === 'string' ? e
          : e instanceof Error ? e.message
          : JSON.stringify(e);
        updateUpload(file.filename, {
          status: 'error',
          error: msg,
          bytes: 0,
        });
        logError(`Upload failed for ${file.filename}: ${msg}`);
      }
    }

    setLoading(null);
    const failed = uploads.filter(u => u.status === 'error').length;
    if (failed > 0) {
      notify('warning', `Upload finished with ${failed} failed file(s)`);
    } else {
      notify('success', `All ${files.length} file(s) uploaded successfully`);
    }
  };

  const hasFiles = cameraData?.camera?.device_id && cameraData?.files && cameraData.files.length > 0;

  return (
    <div>
      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
        <button
          className="button"
          onClick={() => run("camera", getCamera)}
          disabled={loading !== null}
        >
          {loading === "camera" ? "Loading..." : "Test Camera"}
        </button>
        <button
          className="button"
          onClick={() => run("api", () => request("GET", "/api/self", null))}
          disabled={loading !== null}
        >
          {loading === "api" ? "Loading..." : "Test API"}
        </button>
        <button
          className="button"
          onClick={handleUploadFiles}
          disabled={loading !== null || !hasFiles}
        >
          {loading === "uploading" ? "Uploading..." : `Upload Files${hasFiles ? ` (${cameraData!.files!.length})` : ''}`}
        </button>
      </div>
      {loading && !uploads.length && (
        <div className="loading-indicator">
          <div className="spinner" />
          <span>{
            loading === "camera" ? "Scanning for cameras..." :
            loading === "api" ? "Contacting server..." :
            loading === "uploading" ? "Preparing uploads..." :
            "Working..."
          }</span>
        </div>
      )}
      <UploadTable uploads={uploads} />
    </div>
  );
}
