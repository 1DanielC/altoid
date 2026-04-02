import { createContext, useContext, useState, useCallback, useRef, ReactNode } from 'react';

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface Notification {
  id: number;
  type: NotificationType;
  message: string;
}

interface NotificationContextValue {
  notification: Notification | null;
  notify: (type: NotificationType, message: string) => void;
  dismiss: () => void;
}

const NotificationContext = createContext<NotificationContextValue | null>(null);

const AUTO_DISMISS_MS = 5000;

export function NotificationProvider({ children }: { children: ReactNode }) {
  const [notification, setNotification] = useState<Notification | null>(null);
  const counterRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const dismiss = useCallback(() => {
    setNotification(null);
  }, []);

  const notify = useCallback((type: NotificationType, message: string) => {
    clearTimeout(timerRef.current);
    const id = ++counterRef.current;
    setNotification({ id, type, message });

    if (type !== 'error') {
      timerRef.current = setTimeout(() => {
        setNotification(prev => (prev?.id === id ? null : prev));
      }, AUTO_DISMISS_MS);
    }
  }, []);

  return (
    <NotificationContext.Provider value={{ notification, notify, dismiss }}>
      {children}
    </NotificationContext.Provider>
  );
}

export function useNotification() {
  const ctx = useContext(NotificationContext);
  if (!ctx) throw new Error('useNotification must be used within NotificationProvider');
  return ctx;
}
