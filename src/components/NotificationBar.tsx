import { useNotification } from '../contexts/NotificationContext';
import './NotificationBar.css';

const ICONS: Record<string, string> = {
  success: '\u2713',
  error: '\u2717',
  warning: '!',
  info: 'i',
};

export default function NotificationBar() {
  const { notification, dismiss } = useNotification();

  if (!notification) return null;

  return (
    <div className={`notification-bar notification-${notification.type}`}>
      <span className="notification-icon">{ICONS[notification.type]}</span>
      <span className="notification-message">{notification.message}</span>
      <button className="notification-dismiss" onClick={dismiss}>&times;</button>
    </div>
  );
}
