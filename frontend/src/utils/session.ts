/**
 * The session lives in localStorage so a reload does not log the player out.
 * Both the Redux slice and the HTTP client need it, and neither should own the
 * storage key, so it lives here.
 */
import type { Session } from './api/types';

const STORAGE_KEY = 'legacy.session';

const EMPTY_SESSION: Session = { token: null, user: null };

export function readSession(): Session {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return EMPTY_SESSION;
  }

  try {
    return JSON.parse(raw) as Session;
  } catch {
    // A corrupted entry is worth less than no entry: signing in again rewrites it.
    return EMPTY_SESSION;
  }
}

export function writeSession(session: Session): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(session));
}

export function readToken(): string | null {
  return readSession().token;
}
