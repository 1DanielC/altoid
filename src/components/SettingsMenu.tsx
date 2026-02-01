import { useState, useEffect } from 'react';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useUserQuery } from '../contexts/AppContext';
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

export default function SettingsMenu() {
  const [isOpen, setIsOpen] = useState(false);
  const [hostOverride, setHostOverrideState] = useState<string>('');
  const [hostSaveStatus, setHostSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const { data: user } = useUserQuery();

  // Load host override when menu opens
  useEffect(() => {
    if (isOpen) {
      getHostOverride().then(host => {
        setHostOverrideState(host || '');
      }).catch(() => {
        setHostOverrideState('');
      });
    }
  }, [isOpen]);

  const handleHostSave = async () => {
    setHostSaveStatus('saving');
    try {
      await setHostOverride(hostOverride.trim() || null);
      setHostSaveStatus('saved');
      setTimeout(() => setHostSaveStatus('idle'), 2000);
    } catch {
      setHostSaveStatus('error');
      setTimeout(() => setHostSaveStatus('idle'), 2000);
    }
  };

  // Cmd+Shift+S (Mac) or Ctrl+Shift+S (Windows/Linux) hotkey
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 's') {
        e.preventDefault();
        setIsOpen(prev => !prev);
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
                </>
              ) : (
                <div className="profile-empty">Not logged in</div>
              )}
            </div>
          </section>

          {/* Host Override Section */}
          <section className="settings-section">
            <h3>API Configuration</h3>
            <div className="host-override-group">
              <label className="host-override-label" htmlFor="host-override">
                Host Override
              </label>
              <div className="host-override-input-row">
                <input
                  id="host-override"
                  type="text"
                  className="host-override-input"
                  placeholder="e.g., https://custom.openspace.ai"
                  value={hostOverride}
                  onChange={(e) => setHostOverrideState(e.target.value)}
                />
                <button
                  className={`host-save-btn ${hostSaveStatus}`}
                  onClick={handleHostSave}
                  disabled={hostSaveStatus === 'saving'}
                >
                  {hostSaveStatus === 'saving' ? 'Saving...' :
                   hostSaveStatus === 'saved' ? 'Saved' :
                   hostSaveStatus === 'error' ? 'Error' : 'Save'}
                </button>
              </div>
              <p className="host-override-hint">
                Leave empty to use the default host. Requires restart to take effect.
              </p>
            </div>
          </section>

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
