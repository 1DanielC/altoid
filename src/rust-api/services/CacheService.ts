import { invoke } from "@tauri-apps/api/core";

export async function clearCache(): Promise<void> {
  await invoke("clear_cache");
}
