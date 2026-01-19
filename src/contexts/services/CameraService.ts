import {invoke} from "@tauri-apps/api/core";


export async function getCamera(): Promise<string> {
  return await invoke<string>("get_camera");
}
export async function uploadAllFiles(): Promise<void> {
  await invoke("get_camera_files");
}