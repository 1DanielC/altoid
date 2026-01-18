import { useUserQuery, useLoginMutation } from '../contexts/AppContext';

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
  // Query for current user state
  const { data: userInfo, isLoading: isLoadingUser } = useUserQuery();

  // Mutation for login action
  const loginMutation = useLoginMutation();

  const initials = userInfo
      ? getInitials(userInfo.fullName)
      : '🔑';

  // Derive loading state from query and mutation
  const isLoggingIn = isLoadingUser || loginMutation.isPending;
  const disabled = isLoggingIn || !!userInfo;

  return (
      <button
          className="login-button"
          disabled={disabled}
          onClick={() => loginMutation.mutate({ clearAuth: !!userInfo })}
      >
        {initials}
      </button>
  );
}
