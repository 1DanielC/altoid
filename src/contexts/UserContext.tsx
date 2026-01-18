import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { OpenSpaceAPIClient } from '../api/client';
import { loginAndGetAuthToken } from '../rust-api/services/AuthService';
import { checkAuth } from '../rust-api/services/SystemService';
import { AuthResult } from '../rust-api/model/AuthResult.ts';

// ============================================================================
// TYPES
// ============================================================================
interface UserInfo {
  email: string;
  fullName?: string;
}

interface AuthContextType {
  userInfo: UserInfo | null;
  isLoggingIn: boolean;
  apiClient: OpenSpaceAPIClient | null;
  login: () => Promise<void>;
}

const UserContext = createContext<AuthContextType | undefined>(undefined);

// ============================================================================
// HELPERS
// ============================================================================
function initializeApiClient(authData: AuthResult): OpenSpaceAPIClient {
  sessionStorage.setItem('os_token', authData.accessToken);
  sessionStorage.setItem('os_token_type', authData.tokenType);
  sessionStorage.setItem('os_api_host', authData.apiHost);

  return new OpenSpaceAPIClient(
    authData.apiHost,
    authData.accessToken,
    authData.tokenType
  );
}

function restoreApiClientFromSession(): OpenSpaceAPIClient | null {
  const storedToken = sessionStorage.getItem('os_token');
  const tokenType = sessionStorage.getItem('os_token_type');
  const apiHost = sessionStorage.getItem('os_api_host');

  console.log("Restoring API client from session:", { storedToken, tokenType, apiHost });
  if (storedToken && tokenType && apiHost) {
    return new OpenSpaceAPIClient(apiHost, storedToken, tokenType);
  }

  return null;
}

// ============================================================================
// PROVIDER
// ============================================================================
export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [isLoggingIn, setIsLoggingIn] = useState(false);
  const [apiClient, setApiClient] = useState<OpenSpaceAPIClient | null>(null);

  // Restore session on mount
  useEffect(() => {
    const client = restoreApiClientFromSession();

    if (client) {
      setApiClient(client);

      client.getSelf()
        .then(setUserInfo)
        .catch(() => {
          sessionStorage.clear();
          setApiClient(null);
          setUserInfo(null);
        });
    } else {
      checkAuth()
        .then(setUserInfo)
        .catch(() => setUserInfo(null));
    }
  }, []);

  const login = async () => {
    setIsLoggingIn(true);
    try {
      const authData = await loginAndGetAuthToken();
      setUserInfo(authData.userInfo);

      const client = initializeApiClient(authData);
      setApiClient(client);
    } catch (error) {
      console.error('Login failed:', error);
    } finally {
      setIsLoggingIn(false);
    }
  };

  return (
    <UserContext.Provider value={{ userInfo, isLoggingIn, apiClient, login }}>
      {children}
    </UserContext.Provider>
  );
};

// ============================================================================
// HOOK
// ============================================================================
export const useAuth = () => {
  const context = useContext(UserContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
};
