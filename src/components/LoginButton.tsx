import { useAuth } from '../contexts/AppContext';

function getInitials(fullName?: string): string {
  if (!fullName || !fullName.trim()) {
    return 'OS';
  }

  const parts = fullName.trim().split(/\s+/);

  if (parts.length === 0) {
    return 'OS';
  } else if (parts.length === 1) {
    // Single name, take first two characters
    return parts[0].substring(0, 2).toUpperCase();
  } else {
    // Multiple names, take first char of first and last name
    const first = parts[0].charAt(0);
    const last = parts[parts.length - 1].charAt(0);
    return (first + last).toUpperCase();
  }
}

export default function LoginButton() {
  const { userInfo, isLoggingIn, login } = useAuth();

  const initials = userInfo
    ? getInitials(userInfo.fullName)
    : '🔑';

  return (
    <button
      className="login-button"
      disabled={isLoggingIn || !!userInfo}
      onClick={login}
    >
      {initials}
    </button>
  );
}
