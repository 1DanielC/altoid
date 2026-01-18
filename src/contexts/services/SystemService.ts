import { invoke } from "@tauri-apps/api/core";
import {UserInfo} from "../../rust-api/model/AuthResult.ts";

export async function checkAuth(): Promise<UserInfo> {
  return await invoke<UserInfo>("check_auth");
}

export async function getInitials(fullName?: string): Promise<string> {
  return await invoke<string>("get_initials", { fullName });
}

export async function deleteData(): Promise<void> {
  await invoke("clear_cache");
}