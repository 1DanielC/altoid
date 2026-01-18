import React, {createContext, useContext, useState, ReactNode, useEffect} from 'react';
import {getUser, logout} from "./services/ApiService.ts";
import {UserInfo} from "../rust-api/model/AuthResult.ts";
import {deleteData} from "./services/SystemService.ts";

interface AuthContextType {
  userInfo: UserInfo | null;
  isLoggingIn: boolean;
  doLogin: (clearAuth: Boolean) => Promise<void>;
  doLogout: () => Promise<void>;
  deleteAllData: () => Promise<void>;
}

const UserContext = createContext<AuthContextType | undefined>(undefined);

export const UserProvider: React.FC<{ children: ReactNode }> = ({children}) => {
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [isLoggingIn, setIsLoggingIn] = useState(false);

  useEffect(() => {
    doLogin().then(() => {});
  }, []);

  const doLogin = async (clearAuth: Boolean = false) => {
    setIsLoggingIn(true);
    try {
      if (clearAuth) {
        await doLogout()
      }

      const authData: UserInfo = await getUser();
      console.log("User Acquired", authData);
      setUserInfo(authData);
    } catch (error) {
      console.error('Login failed:', error);
    } finally {
      setIsLoggingIn(false);
    }
  }

  const doLogout = async () => {
    await logout();
    setUserInfo(null);
  }

  const deleteAllData = async () => {
    await deleteData()
    setUserInfo(null);
  }

  return (
      <UserContext.Provider value={{userInfo, isLoggingIn, doLogin, doLogout, deleteAllData}}>
        {children}
      </UserContext.Provider>
  );
};

export const useUser = () => {
  const context = useContext(UserContext);
  if (!context) {
    throw new Error('useUser must be used within UserProvider');
  }
  return context;
};
