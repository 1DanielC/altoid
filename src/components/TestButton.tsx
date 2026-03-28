import { useState, useCallback } from "react";
import { getCamera, uploadFile, CameraResult } from "../contexts/services/CameraService.ts";
import { request } from "../contexts/services/ApiService.ts";
import UploadTable, { UploadEntry } from "./UploadTable.tsx";

export default function TestButton() {
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [cameraData, setCameraData] = useState<CameraResult | null>(null);
  const [uploads, setUploads] = useState<UploadEntry[]>([]);

  const run = async (label: string, fn: () => Promise<unknown>) => {
    setLoading(label);
    setResult(null);
    try {
      const r = await fn();
      if (label === "camera") setCameraData(r as CameraResult | null);
      setResult(JSON.stringify(r, null, 2));
    } catch (e: unknown) {
      const msg = typeof e === 'string' ? e
        : e instanceof Error ? e.message
        : JSON.stringify(e, null, 2);
      setResult(`Error: ${msg}`);
    } finally {
      setLoading(null);
    }
  };

  const updateUpload = useCallback((filename: string, update: Partial<UploadEntry>) => {
    setUploads(prev => prev.map(u =>
      u.filename === filename ? { ...u, ...update } : u
    ));
  }, []);


  const handleUploadFiles = async () => {
    setLoading("uploading");
    setResult(null);

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
      progress: 0,
    })));

    // Process one at a time, top to bottom
    for (const file of files) {
      updateUpload(file.filename, { status: 'in_progress', progress: 50 });

      try {
        const r = await uploadFile(deviceId, file.path, file.filename, mountPoint, file.content_type);

        updateUpload(file.filename, {
          uploadId: r.uploadId ?? null,
          status: r.status === 'Completed' || r.status === 'Uploaded' ? 'uploaded' : 'new',
          progress: r.status === 'Completed' || r.status === 'Uploaded' ? 100 : 0,
        });
      } catch (e: unknown) {
        const msg = typeof e === 'string' ? e
          : e instanceof Error ? e.message
          : JSON.stringify(e);
        updateUpload(file.filename, { status: 'new', progress: 0 });
        console.error(`Upload failed for ${file.filename}: ${msg}`);
      }
    }

    setLoading(null);
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
            loading === "uploading" ? "Uploading files..." :
            "Working..."
          }</span>
        </div>
      )}
      <UploadTable uploads={uploads} />
      {result && !loading && <pre className="test-result">{result}</pre>}
    </div>
  );
}
