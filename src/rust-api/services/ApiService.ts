import {invoke} from "@tauri-apps/api/core";

export async function request(
    method: string,
    path: string,
    body: unknown,
    content_type?: string
): Promise<any> {
  return invoke<any>("req", {
    method,
    path,
    body,
    content_type
  })
}
