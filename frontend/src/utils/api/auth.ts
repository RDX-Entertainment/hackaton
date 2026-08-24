import { post } from './client';
import type { AuthenticatedSession } from './types';

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
