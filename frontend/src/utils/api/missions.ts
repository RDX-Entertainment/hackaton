import { get, post } from './client';
import type { MissionCommand, UnknownRow } from './types';

export function listMissions(): Promise<UnknownRow[]> {
  return get<UnknownRow[]>('/missions');
}

export function commandMission(id: number, action: MissionCommand): Promise<void> {
  return post<void>(`/missions/${id}`, { action });
}
