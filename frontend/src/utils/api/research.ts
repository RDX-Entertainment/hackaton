import { get, post } from './client';
import type { UnknownRow } from './types';

export function listResearch(): Promise<UnknownRow[]> {
  return get<UnknownRow[]>('/research');
}

/** Researching queues work; the version only rises once the worker runs. */
export function startResearch(
  softwareId: number,
): Promise<{ process_id: number; software_id: number; seconds: number }> {
  return post('/research', { software_id: softwareId });
}
