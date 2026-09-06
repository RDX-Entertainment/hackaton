/**
 * The session lives in localStorage so a reload does not log the player out.
 * Both the Redux slice and the HTTP client need it, and neither should own the
 * storage key, so it lives here.
 */
import { usarIdioma } from '../i18n';
import type { Session } from './api/types';

const STORAGE_KEY = 'legacy.session';

const EMPTY_SESSION: Session = { token: null, user: null };

export function readSession(): Session {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return EMPTY_SESSION;
  }

  try {
    const session = JSON.parse(raw) as Session;
    // El idioma se pone aqui y en `writeSession` porque son los dos unicos sitios por los
    // que pasa una sesion: al arrancar y al entrar. Asi ninguna pantalla tiene que acordarse.
    usarIdioma(session.user?.lang);
    return session;
  } catch {
    // A corrupted entry is worth less than no entry: signing in again rewrites it.
    return EMPTY_SESSION;
  }
}

export function writeSession(session: Session): void {
  usarIdioma(session.user?.lang);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(session));
}

export function readToken(): string | null {
  return readSession().token;
}
