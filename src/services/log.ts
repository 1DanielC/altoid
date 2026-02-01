export type LogLevel = 'info' | 'warning' | 'error';

export interface LogEntry {
  id: number;
  level: LogLevel;
  message: string;
  timestamp: Date;
}

interface StoredLogEntry {
  id: number;
  level: LogLevel;
  message: string;
  timestamp: string;
}

const STORAGE_KEY = 'activity-log';

// Global log state and listeners
let logEntries: LogEntry[] = [];
let nextId = 1;
let listeners: Set<() => void> = new Set();

// Load initial state from localStorage
function loadFromStorage(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed: StoredLogEntry[] = JSON.parse(stored);
      logEntries = parsed.map(e => ({ ...e, timestamp: new Date(e.timestamp) }));
      nextId = logEntries.length > 0 ? Math.max(...logEntries.map(e => e.id)) + 1 : 1;
    }
  } catch {
    // Ignore parse errors
  }
}

// Save to localStorage
function saveToStorage(): void {
  try {
    const toStore: StoredLogEntry[] = logEntries.map(e => ({
      ...e,
      timestamp: e.timestamp.toISOString()
    }));
    localStorage.setItem(STORAGE_KEY, JSON.stringify(toStore));
  } catch {
    // Ignore storage errors
  }
}

// Listen for changes from other windows
function setupStorageListener(): void {
  window.addEventListener('storage', (event) => {
    if (event.key === STORAGE_KEY) {
      loadFromStorage();
      notifyListeners();
    }
  });
}

// Initialize
loadFromStorage();
setupStorageListener();

function notifyListeners() {
  listeners.forEach(listener => listener());
}

function addEntry(level: LogLevel, message: string): void {
  logEntries = [...logEntries, { id: nextId++, level, message, timestamp: new Date() }];
  saveToStorage();
  notifyListeners();
}

// Public API to add log entries from anywhere in the app
export function logInfo(message: string) {
  addEntry('info', message);
}

export function logWarning(message: string) {
  addEntry('warning', message);
}

export function logError(message: string) {
  addEntry('error', message);
}

export function clearLog() {
  logEntries = [];
  saveToStorage();
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
