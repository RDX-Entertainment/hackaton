import { get, post } from './client';
import type { Clan, ClanRequest, ClanSummary, CreatedRecord } from './types';

/** Answers null when the player belongs to no clan. */
export function getMyClan(): Promise<Clan | null> {
  return get<Clan | null>('/clans/mine');
}

export function listClans(): Promise<ClanSummary[]> {
  return get<ClanSummary[]>('/clans');
}

export function createClan(
  name: string,
  nick: string,
  description: string,
): Promise<CreatedRecord> {
  return post<CreatedRecord>('/clans', { name, nick, description });
}

export function requestToJoin(id: number, message: string): Promise<CreatedRecord> {
  return post<CreatedRecord>(`/clans/${id}/join`, { message });
}

/** Only visible to members whose auth level lets them administer the clan. */
export function listJoinRequests(): Promise<ClanRequest[]> {
  return get<ClanRequest[]>('/clans/requests');
}

export function acceptJoinRequest(id: number): Promise<void> {
  return post<void>(`/clans/requests/${id}/accept`);
}
