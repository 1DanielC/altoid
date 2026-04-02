import { NotificationType } from '../contexts/NotificationContext';

interface IpcErrorPayload {
  status: string;
  body: { message: string };
}

const USER_FRIENDLY_MESSAGES: Record<string, string> = {
  NotAuthenticated: 'Please sign in to continue.',
  NotAuthorized: 'You don\'t have permission to do that.',
  NotFound: 'The requested resource was not found.',
  Unavailable: 'The server is unavailable. Check your connection and try again.',
  Conflict: 'A conflict occurred. The resource may have been modified.',
  InvalidArgument: 'The request was invalid.',
  InternalError: 'Something went wrong. Please try again or contact support.',
  ImATeapot: '\u{1FAD6}',
};

const STATUS_TO_NOTIFICATION_TYPE: Record<string, NotificationType> = {
  NotAuthenticated: 'warning',
  NotAuthorized: 'warning',
  NotFound: 'warning',
  Unavailable: 'error',
  Conflict: 'warning',
  InvalidArgument: 'warning',
  InternalError: 'error',
  ImATeapot: 'info',
};

function isIpcError(error: unknown): error is IpcErrorPayload {
  if (typeof error !== 'object' || error === null) return false;
  const obj = error as Record<string, unknown>;
  return typeof obj.status === 'string'
    && typeof obj.body === 'object'
    && obj.body !== null
    && typeof (obj.body as Record<string, unknown>).message === 'string';
}

function tryParseIpcError(error: unknown): IpcErrorPayload | null {
  if (isIpcError(error)) return error;

  if (typeof error === 'string') {
    try {
      const parsed = JSON.parse(error);
      if (isIpcError(parsed)) return parsed;
    } catch {
      // not JSON
    }
  }

  return null;
}

export interface ParsedError {
  type: NotificationType;
  message: string;
}

export function parseIpcError(error: unknown): ParsedError {
  const ipc = tryParseIpcError(error);

  if (ipc) {
    const friendly = USER_FRIENDLY_MESSAGES[ipc.status];
    const type = STATUS_TO_NOTIFICATION_TYPE[ipc.status] ?? 'error';
    return { type, message: friendly ?? ipc.body.message };
  }

  // Fallback for non-IPC errors (e.g. timeout, network)
  if (error instanceof Error) {
    return { type: 'error', message: error.message };
  }

  if (typeof error === 'string') {
    return { type: 'error', message: error };
  }

  return { type: 'error', message: 'An unexpected error occurred.' };
}
