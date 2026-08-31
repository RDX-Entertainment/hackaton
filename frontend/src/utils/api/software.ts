import { get, post } from './client';
import type { CreatedRecord, OwnedFile, SoftwareType } from './types';

export function listSoftware(): Promise<OwnedFile[]> {
  return get<OwnedFile[]>('/software');
}

export function createSoftware(
  name: string,
  softType: number,
  size: number,
  ram: number,
  serverId: number,
): Promise<CreatedRecord> {
  return post<CreatedRecord>('/software', {
    name,
    soft_type: softType,
    size,
    ram,
    server_id: serverId,
  });
}

export function listSoftwareTypes(): Promise<SoftwareType[]> {
  return get<SoftwareType[]>('/game/software-types');
}
