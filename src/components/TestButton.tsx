import { useState } from "react";
import { getCamera } from "../contexts/services/CameraService.ts";
import { request } from "../contexts/services/ApiService.ts";

export default function TestButton() {
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);

  const run = async (label: string, fn: () => Promise<unknown>) => {
    setLoading(label);
    setResult(null);
    try {
      const r = await fn();
      setResult(JSON.stringify(r, null, 2));
    } catch (e) {
      setResult(`Error: ${e}`);
    } finally {
      setLoading(null);
    }
  };

  return (
    <div>
      <div style={{ display: "flex", gap: 8 }}>
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
      </div>
      {result && <pre className="test-result">{result}</pre>}
    </div>
  );
}
