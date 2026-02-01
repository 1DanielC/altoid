export type LogLevel = 'info' | 'warning' | 'error';

export interface LogEntry {
  id: number;
  level: LogLevel;
  message: string;
  timestamp: Date;
}

// Global log state and listeners
let logEntries: LogEntry[] = [];
let nextId = 1;
let listeners: Set<() => void> = new Set();

function notifyListeners() {
  listeners.forEach(listener => listener());
}

// Public API to add log entries from anywhere in the app
export function logInfo(message: string) {
  logEntries = [...logEntries, { id: nextId++, level: 'info', message, timestamp: new Date() }];
  notifyListeners();
}

export function logWarning(message: string) {
  logEntries = [...logEntries, { id: nextId++, level: 'warning', message, timestamp: new Date() }];
  notifyListeners();
}

export function logError(message: string) {
  logEntries = [...logEntries, { id: nextId++, level: 'error', message, timestamp: new Date() }];
  notifyListeners();
}

export function clearLog() {
  logEntries = [];
  notifyListeners();
}

// For subscribing to log changes (used by Dialog component)
export function getLogEntries(): LogEntry[] {
  return logEntries;
}

export function subscribeToLog(listener: () => void): () => void {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}
