import { get, post } from './client';
import type { Mission, MissionCommand } from './types';

export function listMissions(): Promise<Mission[]> {
  return get<Mission[]>('/missions');
}

export function commandMission(id: number, action: MissionCommand): Promise<void> {
  return post<void>(`/missions/${id}`, { action });
}
