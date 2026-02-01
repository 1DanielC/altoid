import { useState, useEffect, useRef } from 'react';
import { LogEntry, LogLevel, getLogEntries, subscribeToLog, clearLog } from '../services/log';
import './Dialog.css';

// Hook to subscribe to log changes
function useLogEntries(): LogEntry[] {
  const [, forceUpdate] = useState({});

  useEffect(() => {
    return subscribeToLog(() => forceUpdate({}));
  }, []);

  return getLogEntries();
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  });
}

function getLevelIcon(level: LogLevel): string {
  switch (level) {
    case 'info': return 'i';
    case 'warning': return '!';
    case 'error': return 'X';
  }
}

export default function Dialog() {
  const entries = useLogEntries();
  const logEndRef = useRef<HTMLDivElement>(null);
  const [isCollapsed, setIsCollapsed] = useState(false);

  // Auto-scroll to bottom when new entries are added
  useEffect(() => {
    if (!isCollapsed && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [entries, isCollapsed]);

  const errorCount = entries.filter(e => e.level === 'error').length;
  const warningCount = entries.filter(e => e.level === 'warning').length;

  return (
    <div className={`dialog-container ${isCollapsed ? 'collapsed' : ''}`}>
      <div className="dialog-header">
        <div className="dialog-title">
          <span className="dialog-title-text">Activity Log</span>
          {entries.length > 0 && (
            <span className="dialog-badge-container">
              {errorCount > 0 && (
                <span className="dialog-badge badge-error">{errorCount}</span>
              )}
              {warningCount > 0 && (
                <span className="dialog-badge badge-warning">{warningCount}</span>
              )}
            </span>
          )}
        </div>
        <div className="dialog-actions">
          {entries.length > 0 && (
            <button
              className="dialog-clear-btn"
              onClick={clearLog}
              title="Clear log"
            >
              Clear
            </button>
          )}
          <button
            className="dialog-toggle-btn"
            onClick={() => setIsCollapsed(!isCollapsed)}
            title={isCollapsed ? 'Expand' : 'Collapse'}
          >
            {isCollapsed ? '+' : '-'}
          </button>
        </div>
      </div>

      {!isCollapsed && (
        <div className="dialog-content">
          {entries.length === 0 ? (
            <div className="dialog-empty">No activity yet</div>
          ) : (
            <>
              {entries.map(entry => (
                <div key={entry.id} className={`dialog-entry entry-${entry.level}`}>
                  {entry.level === 'info' ? (
                    <span className="entry-icon-spacer" />
                  ) : (
                    <span className={`entry-icon icon-${entry.level}`}>
                      {getLevelIcon(entry.level)}
                    </span>
                  )}
                  <span className="entry-time">{formatTime(entry.timestamp)}</span>
                  <span className="entry-message">{entry.message}</span>
                </div>
              ))}
              <div ref={logEndRef} />
            </>
          )}
        </div>
      )}
    </div>
  );
}
