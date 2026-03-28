import { useState, useCallback } from "react";
import { getCamera, createUploads, uploadFile, CameraResult } from "../contexts/services/CameraService.ts";
import { request } from "../contexts/services/ApiService.ts";
import UploadTable, { UploadEntry } from "./UploadTable.tsx";

const BATCH_SIZE = 3;

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

  const handleCreateUploads = async () => {
    setLoading("uploads");
    setResult(null);
    try {
      const r = await createUploads(cameraData!.camera!.device_id, cameraData!.files!);
      const entries: UploadEntry[] = r.results
        .filter((item) => !item.error)
        .map((item) => ({
          filename: item.filename,
          uploadId: item.response?.uploadId ?? null,
          status: item.response?.uploadId === null ? 'uploaded' : 'new',
          progress: item.response?.uploadId === null ? 100 : 0,
        }));
      setUploads(entries);
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
    const pending = uploads.filter(u => u.status === 'new' && u.uploadId);

    // Find the matching camera file for each pending upload
    const fileMap = new Map(
      (cameraData?.files ?? []).map(f => [f.filename, f])
    );

    // Process in batches of BATCH_SIZE
    for (let i = 0; i < pending.length; i += BATCH_SIZE) {
      const batch = pending.slice(i, i + BATCH_SIZE);

      await Promise.all(batch.map(async (entry) => {
        const file = fileMap.get(entry.filename);
        if (!file || !entry.uploadId) return;

        const fullPath = mountPoint ? `${mountPoint}/${file.path}` : file.path;

        updateUpload(entry.filename, { status: 'in_progress', progress: 50 });

        try {
          await uploadFile(entry.uploadId, fullPath, file.content_type);
          updateUpload(entry.filename, { status: 'uploaded', progress: 100 });
        } catch (e) {
          const msg = typeof e === 'string' ? e
            : e instanceof Error ? e.message
            : JSON.stringify(e);
          updateUpload(entry.filename, { status: 'new', progress: 0 });
          console.error(`Upload failed for ${entry.filename}: ${msg}`);
        }
      }));
    }

    setLoading(null);
  };

  const hasFiles = cameraData?.camera?.device_id && cameraData?.files && cameraData.files.length > 0;
  const pendingUploads = uploads.filter(u => u.status === 'new' && u.uploadId);

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
          onClick={handleCreateUploads}
          disabled={loading !== null || !hasFiles}
        >
          {loading === "uploads" ? "Creating..." : `Create Uploads${hasFiles ? ` (${cameraData!.files!.length})` : ''}`}
        </button>
        <button
          className="button"
          onClick={handleUploadFiles}
          disabled={loading !== null || pendingUploads.length === 0}
        >
          {loading === "uploading" ? "Uploading..." : `Upload Files${pendingUploads.length > 0 ? ` (${pendingUploads.length})` : ''}`}
        </button>
      </div>
      <UploadTable uploads={uploads} />
      {result && <pre className="test-result">{result}</pre>}
    </div>
  );
}
