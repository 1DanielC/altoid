export interface AuthResult {
  userInfo: UserInfo;
  accessToken: string;
  tokenType: string;
  apiHost: string;
}

export interface UserInfo {
  email: string;
  fullName?: string;
}