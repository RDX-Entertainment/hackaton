import { del, get, post } from './client';
import type { CreatedRecord, ListEntry } from './types';

export function listCompromised(): Promise<ListEntry[]> {
  return get<ListEntry[]>('/lists');
}

export function addCompromised(
  ip: number,
  login: string,
  password: string,
): Promise<CreatedRecord> {
  return post<CreatedRecord>('/lists', { ip, login, password });
}

export function removeCompromised(id: number): Promise<void> {
  return del<void>(`/lists/${id}`);
}
