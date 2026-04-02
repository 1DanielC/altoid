import { useState, useEffect, useRef } from 'react';
import { LogEntry, LogLevel, getLogEntries, subscribeToLog, clearLog } from '../services/log';
import { exportActivityLog } from '../contexts/services/ApiService';
import './ActivityLogWindow.css';

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

export default function ActivityLogWindow() {
  const entries = useLogEntries();
  const logEndRef = useRef<HTMLDivElement>(null);
  const [exporting, setExporting] = useState(false);
  const [exportStatus, setExportStatus] = useState<'idle' | 'success' | 'error'>('idle');

  // Auto-scroll to bottom when new entries are added
  useEffect(() => {
    if (logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [entries]);

  const handleExport = async () => {
    setExporting(true);
    setExportStatus('idle');
    try {
      await exportActivityLog(
        entries.map(e => ({
          level: e.level,
          message: e.message,
          timestamp: e.timestamp.toISOString(),
        })),
      );
      setExportStatus('success');
      setTimeout(() => setExportStatus('idle'), 3000);
    } catch {
      setExportStatus('error');
      setTimeout(() => setExportStatus('idle'), 3000);
    } finally {
      setExporting(false);
    }
  };

  const errorCount = entries.filter(e => e.level === 'error').length;
  const warningCount = entries.filter(e => e.level === 'warning').length;

  function exportButtonLabel(): string {
    if (exporting) return 'Exporting...';
    if (exportStatus === 'success') return 'Exported';
    if (exportStatus === 'error') return 'Export Failed';
    return 'Export to OpenSpace';
  }

  return (
    <div className="activity-log-window">
      <div className="activity-log-header" data-tauri-drag-region>
        <div className="activity-log-title">
          <span>Activity Log</span>
          {entries.length > 0 && (
            <span className="log-badges">
              {errorCount > 0 && <span className="log-badge badge-error">{errorCount}</span>}
              {warningCount > 0 && <span className="log-badge badge-warning">{warningCount}</span>}
            </span>
          )}
        </div>
        <div className="log-header-actions">
          {entries.length > 0 && (
            <>
              <button
                className="log-export-btn"
                onClick={handleExport}
                disabled={exporting}
              >
                {exportButtonLabel()}
              </button>
              <button className="log-clear-btn" onClick={clearLog}>Clear</button>
            </>
          )}
        </div>
      </div>

      <div className="activity-log-content">
        {entries.length === 0 ? (
          <div className="log-empty">No activity yet</div>
        ) : (
          <>
            {entries.map(entry => (
              <div key={entry.id} className={`log-entry entry-${entry.level}`}>
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
    </div>
  );
}
