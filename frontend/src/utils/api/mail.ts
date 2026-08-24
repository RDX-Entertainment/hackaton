import { del, get, post } from './client';
import type { CreatedRecord, UnknownRow } from './types';

export function listMail(): Promise<UnknownRow[]> {
  return get<UnknownRow[]>('/mails');
}

/** Reading marks the message read as a side effect, so the list is stale after. */
export function readMail(id: number): Promise<UnknownRow> {
  return get<UnknownRow>(`/mails/${id}`);
}

export function sendMail(to: string, subject: string, text: string): Promise<CreatedRecord> {
  return post<CreatedRecord>('/mails', { to, subject, text });
}

export function removeMail(id: number): Promise<void> {
  return del<void>(`/mails/${id}`);
}
