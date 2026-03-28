import {invoke} from "@tauri-apps/api/core";
import {UserInfo} from "../../rust-api/model/AuthResult.ts";
import {logError} from "../../services/log.ts";

const DEFAULT_TIMEOUT_MS = 30000; // 30 seconds

function withTimeout<T>(promise: Promise<T>, ms: number, errorMessage: string): Promise<T> {
  return Promise.race([
    promise,
    new Promise<T>((_, reject) =>
      setTimeout(() => reject(new Error(errorMessage)), ms)
    )
  ]);
}

export async function request(
    method: string,
    path: string,
    body: unknown,
    content_type?: string,
    timeoutMs: number = DEFAULT_TIMEOUT_MS
): Promise<any> {
  try {
    return await withTimeout(
      invoke<any>("req", {
        method,
        path,
        body,
        content_type
      }),
      timeoutMs,
      `Request timeout: ${method} ${path} took longer than ${timeoutMs}ms`
    );
  } catch (error) {
    logError(`Error making request: ${method} ${path} \n \n ${error}`);
    throw error;
  }
}

export async function getUser(timeoutMs: number = DEFAULT_TIMEOUT_MS): Promise<UserInfo> {
  try {
    return await withTimeout(
      invoke<UserInfo>("get_user"),
      timeoutMs,
      `Request timeout: get_user took longer than ${timeoutMs}ms`
    );
  } catch (error) {
    logError(`Error getting user`);
    throw error;
  }
}

export async function logout(timeoutMs: number = DEFAULT_TIMEOUT_MS): Promise<void> {
  try {
    return await withTimeout(
      invoke("clear_state"),
      timeoutMs,
      `Request timeout: logout took longer than ${timeoutMs}ms`
    );
  } catch (error) {
    logError(`Error logging out: ${error}`);
    throw error;
  }
}

export async function getHostOverride(): Promise<string | null> {
  try {
    return await invoke<string | null>("get_host");
  } catch (error) {
    logError(`Error getting host override: ${error}`);
    throw error;
  }
}

export async function setHostOverride(host: string | null): Promise<void> {
  try {
    return await invoke("set_host", { host });
  } catch (error) {
    logError(`Error setting host override: ${error}`);
    throw error;
  }
}