import { useUserQuery } from '../contexts/AppContext';

function getInitials(fullName?: string): string {
  if (!fullName || !fullName.trim()) {
    return 'OS';
  }

  const parts = fullName.trim().split(/\s+/);

  if (parts.length === 0) {
    return 'OS';
  } else if (parts.length === 1) {
    return parts[0].substring(0, 2).toUpperCase();
  } else {
    const first = parts[0].charAt(0);
    const last = parts[parts.length - 1].charAt(0);
    return (first + last).toUpperCase();
  }
}

export default function LoginButton({ onClick }: { onClick: () => void }) {
  const { data: userInfo } = useUserQuery();

  const initials = userInfo
      ? getInitials(userInfo.fullName)
      : 'OS';

  return (
      <button className="login-button" onClick={onClick}>
        {initials}
      </button>
  );
}
