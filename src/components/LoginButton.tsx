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

  return (
      <button className="login-button" onClick={onClick}>
        {userInfo ? getInitials(userInfo.fullName) : (
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
              <polyline points="10 17 15 12 10 7" />
              <line x1="15" y1="12" x2="3" y2="12" />
            </svg>
        )}
      </button>
  );
}
