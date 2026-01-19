// TODO get files from The camera and upload them!
import {invoke} from "@tauri-apps/api/core";

export async function uploadAllFiles(): Promise<void> {
  await invoke("get_camera_files");
}