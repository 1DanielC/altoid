import {UserInfo} from "../rust-api/model/AuthResult.ts";

export class OpenSpaceAPIClient {
  private token: string;
  private tokenType: string;
  private apiHost: string;

  constructor(apiHost: string, token: string, tokenType: string) {
    this.apiHost = apiHost;
    this.token = token;
    this.tokenType = tokenType;
  }

  private async request<T>(
    endpoint: string,
    options?: RequestInit
  ): Promise<T> {
    const response = await fetch(`${this.apiHost}${endpoint}`, {
      ...options,
      headers: {
        ...options?.headers,
        'Authorization': `${this.tokenType} ${this.token}`,
        'Content-Type': 'application/json',
      },
    });

    if (response.status === 401) {
      // Token expired - clear session storage
      sessionStorage.clear();
      throw new Error('Token expired - please login again');
    }

    if (!response.ok) {
      throw new Error(`API error: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  async getSelf(): Promise<UserInfo> {
    return this.request<UserInfo>('/api/self');
  }

  // TODO return upload
  async createUpload(): Promise<String> {
    return "TODO"
  }

  async uploadChunk(
    uploadId: string,
    chunk: Uint8Array,
    range: string
  ): Promise<void> {
    const response = await fetch(
      `${this.apiHost}/api/tictac/uploads/${uploadId}`,
      {
        method: 'PUT',
        headers: {
          'Authorization': `${this.tokenType} ${this.token}`,
          'Content-Type': 'application/octet-stream',
          'Content-Range': range,
        },
        body: chunk,
      }
    );

    if (response.status === 401) {
      sessionStorage.clear();
      throw new Error('Token expired - please login again');
    }

    if (!response.ok) {
      throw new Error(`Upload chunk failed: ${response.status}`);
    }
  }
}
