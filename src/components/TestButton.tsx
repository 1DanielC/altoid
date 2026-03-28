import { useState } from "react";
import { getCamera, createUploads, CameraResult } from "../contexts/services/CameraService.ts";
import { request } from "../contexts/services/ApiService.ts";

export default function TestButton() {
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [cameraData, setCameraData] = useState<CameraResult | null>(null);

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
          onClick={() => run("uploads", () =>
            createUploads(cameraData!.camera!.device_id, cameraData!.files!)
          )}
          disabled={loading !== null || !hasFiles}
        >
          {loading === "uploads" ? "Creating..." : `Create Uploads${hasFiles ? ` (${cameraData!.files!.length})` : ''}`}
        </button>
      </div>
      {result && <pre className="test-result">{result}</pre>}
    </div>
  );
}
