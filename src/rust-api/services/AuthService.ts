import { invoke } from "@tauri-apps/api/core";
import {AuthResult} from "../model/AuthResult.ts";

export async function loginAndGetAuthToken(): Promise<AuthResult> {
  let res = invoke<AuthResult>("login");
  console.log("BEEF")
  console.log(res)
  return res
}