import {invoke} from "@tauri-apps/api/core";
import {UserInfo} from "../../rust-api/model/AuthResult.ts";
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

export async function getUser(): Promise<UserInfo> {
  return invoke<UserInfo>("get_user");
}

export async function logout(): Promise<void> {
  return invoke("clear_user_cache");
}