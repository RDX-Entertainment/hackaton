import { get, post, put } from './client';
import type { AuthenticatedSession, User } from './types';

export function login(username: string, password: string): Promise<AuthenticatedSession> {
  return post<AuthenticatedSession>('/auth/login', { username, password });
}

export function register(
  username: string,
  password: string,
  email?: string,
): Promise<AuthenticatedSession> {
  return post<AuthenticatedSession>('/auth/register', {
    username,
    password,
    // The server tells an absent e-mail apart from an empty one.
    email: email || undefined,
  });
}

export function me(): Promise<User> {
  return get<User>('/me');
}

export function setLanguage(lang: string): Promise<User> {
  return put<User>('/me/language', { lang });
}
