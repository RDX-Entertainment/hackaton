import { get, put } from './client';
import type { Profile, Settings } from './types';

export function getSettings(): Promise<Settings> {
  return get<Settings>('/settings');
}

export function saveSettings(
  email: string,
  currentPassword: string,
  newPassword?: string,
): Promise<void> {
  return put<void>('/settings', {
    email,
    current_password: currentPassword,
    // Omitted rather than empty: the server reads it as "leave the password alone".
    new_password: newPassword || undefined,
  });
}

export function getProfile(): Promise<Profile> {
  return get<Profile>('/profile');
}

/**
 * Leaving `web` out keeps the page as it is: saving a biography must not wipe
 * the site, and the server reads a missing field as "do not touch it".
 */
export function saveProfile(biography: string, web?: string): Promise<void> {
  return put<void>('/profile', { biography, web });
}
