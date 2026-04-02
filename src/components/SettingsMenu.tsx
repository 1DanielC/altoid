import { useState, useEffect } from 'react';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useUserQuery, useLoginMutation, useLogoutMutation, useNotification } from '../contexts/AppContext';
import { getLogEntries } from '../services/log';
import { getHostOverride, setHostOverride } from '../contexts/services/ApiService';
import './SettingsMenu.css';

let activityLogWindow: WebviewWindow | null = null;

async function openActivityLog() {
  // Check if window already exists and is valid
  if (activityLogWindow) {
    try {
      await activityLogWindow.setFocus();
      return;
    } catch {
      // Window was closed, create a new one
      activityLogWindow = null;
    }
  }

  try {
    activityLogWindow = new WebviewWindow('activity-log', {
      url: '/?window=activity-log',
      title: 'Activity Log',
      width: 450,
      height: 400,
      minWidth: 300,
      minHeight: 200,
      resizable: true,
      center: true,
    });

    activityLogWindow.once('tauri://error', (e) => {
      console.error('Window creation error:', e);
    });

    activityLogWindow.once('tauri://destroyed', () => {
      activityLogWindow = null;
    });
  } catch (err) {
    console.error('Failed to create activity log window:', err);
  }
}

interface SettingsMenuProps {
  isOpen: boolean;
  setIsOpen: (open: boolean) => void;
}

const ENV_OPTIONS = [
  { label: 'Production', value: 'https://openspace.ai' },
  { label: 'Development', value: 'https://development.osdevenv.net' },
  { label: 'Local', value: 'http://localhost:8080' },
];

export default function SettingsMenu({ isOpen, setIsOpen }: SettingsMenuProps) {
  const [selectedHost, setSelectedHost] = useState<string>('https://openspace.ai');
  const [savedHost, setSavedHost] = useState<string>('https://openspace.ai');
  const [showConfirmModal, setShowConfirmModal] = useState(false);
  const [hostSaveStatus, setHostSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const { data: user } = useUserQuery();
  const { notify } = useNotification();
  const loginMutation = useLoginMutation();
  const logoutMutation = useLogoutMutation();

  // Load current host when menu opens
  useEffect(() => {
    if (isOpen) {
      getHostOverride().then(host => {
        const current = host || 'https://openspace.ai';
        setSelectedHost(current);
        setSavedHost(current);
      }).catch(() => {
        setSelectedHost('https://openspace.ai');
        setSavedHost('https://openspace.ai');
      });
    }
  }, [isOpen]);

  const handleHostSave = () => {
    if (selectedHost === savedHost) return;
    setShowConfirmModal(true);
  };

  const handleConfirmSwitch = async () => {
    setShowConfirmModal(false);
    setHostSaveStatus('saving');
    try {
      await setHostOverride(selectedHost);
      setSavedHost(selectedHost);
      logoutMutation.mutate();
      setHostSaveStatus('saved');
      notify('success', `Switched to ${ENV_OPTIONS.find(o => o.value === selectedHost)?.label ?? selectedHost}`);
      setTimeout(() => setHostSaveStatus('idle'), 2000);
    } catch {
      setHostSaveStatus('error');
      notify('error', 'Failed to switch environment');
      setTimeout(() => setHostSaveStatus('idle'), 2000);
    }
  };

  // Cmd+Shift+S (Mac) or Ctrl+Shift+S (Windows/Linux) hotkey
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 's') {
        e.preventDefault();
        setIsOpen(!isOpen);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Close on Escape
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        setIsOpen(false);
      }
    };

    window.addEventListener('keydown', handleEscape);
    return () => window.removeEventListener('keydown', handleEscape);
  }, [isOpen]);

  if (!isOpen) return null;

  const entries = getLogEntries();
  const errorCount = entries.filter(e => e.level === 'error').length;
  const warningCount = entries.filter(e => e.level === 'warning').length;

  return (
    <div className="settings-overlay" onClick={() => setIsOpen(false)}>
      <div className="settings-menu" onClick={e => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="settings-close" onClick={() => setIsOpen(false)}>×</button>
        </div>

        <div className="settings-content">
          {/* Profile Section */}
          <section className="settings-section">
            <h3>Profile</h3>
            <div className="profile-info">
              {user ? (
                <>
                  <div className="profile-row">
                    <span className="profile-label">Email</span>
                    <span className="profile-value">{user.email}</span>
                  </div>
                  {user.fullName && (
                    <div className="profile-row">
                      <span className="profile-label">Name</span>
                      <span className="profile-value">{user.fullName}</span>
                    </div>
                  )}
                  <button
                    className="auth-btn sign-out-btn"
                    onClick={() => logoutMutation.mutate()}
                    disabled={logoutMutation.isPending}
                  >
                    {logoutMutation.isPending ? 'Signing out...' : 'Sign Out'}
                  </button>
                </>
              ) : (
                <>
                  <div className="profile-empty">Not logged in</div>
                  <button
                    className="auth-btn sign-in-btn"
                    onClick={() => loginMutation.mutate({ clearAuth: false })}
                    disabled={loginMutation.isPending}
                  >
                    {loginMutation.isPending ? 'Signing in...' : 'Sign In'}
                  </button>
                </>
              )}
            </div>
          </section>

          {/* Environment Section */}
          <section className="settings-section">
            <h3>Environment</h3>
            <div className="env-group">
              <select
                id="env-select"
                className="env-select"
                value={selectedHost}
                onChange={(e) => setSelectedHost(e.target.value)}
              >
                {ENV_OPTIONS.map(opt => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
              <span className="env-host-hint">{selectedHost}</span>
              {selectedHost !== savedHost && (
                <button
                  className={`auth-btn confirm-switch-btn`}
                  onClick={handleHostSave}
                  disabled={hostSaveStatus === 'saving'}
                >
                  {hostSaveStatus === 'saving' ? 'Switching...' : 'Switch Environment'}
                </button>
              )}
            </div>
          </section>

          {/* Confirm Environment Switch Modal */}
          {showConfirmModal && (
            <div className="confirm-modal-overlay" onClick={() => setShowConfirmModal(false)}>
              <div className="confirm-modal" onClick={e => e.stopPropagation()}>
                <h3>Switch Environment?</h3>
                <p>
                  Switching to a different environment will log you out and all local data will be lost.
                  You will need to sign in again.
                </p>
                <div className="confirm-modal-actions">
                  <button className="auth-btn sign-out-btn" onClick={() => setShowConfirmModal(false)}>
                    Cancel
                  </button>
                  <button className="auth-btn confirm-switch-btn" onClick={handleConfirmSwitch}>
                    Switch Environment
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Activity Log Section */}
          <section className="settings-section">
            <h3>Activity Log</h3>
            <button className="activity-log-btn" onClick={openActivityLog}>
              <span>Open Activity Log</span>
              {(errorCount > 0 || warningCount > 0) && (
                <span className="log-badges">
                  {errorCount > 0 && <span className="log-badge badge-error">{errorCount}</span>}
                  {warningCount > 0 && <span className="log-badge badge-warning">{warningCount}</span>}
                </span>
              )}
            </button>
          </section>
        </div>

        <div className="settings-footer">
          <span className="hotkey-hint">Press <kbd>⌘</kbd><kbd>⇧</kbd><kbd>S</kbd> or <kbd>Esc</kbd> to close</span>
        </div>
      </div>
    </div>
  );
}
