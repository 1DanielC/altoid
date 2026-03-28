import {invoke} from "@tauri-apps/api/core";

export async function getCamera(): Promise<string | null> {
  return await Promise.race([
    invoke<string | null>("get_camera"),
    new Promise<string | null>((_, reject) =>
      setTimeout(() => reject(new Error("Camera detection timed out")), 10000)
    ),
  ]);
}
export async function uploadAllFiles(): Promise<void> {
  await invoke("get_camera_files");
}